//! Unified Peer Types
//!
//! This module provides a unified `Peer` type that represents all known network
//! participants. It consolidates the previous separate `ContactInfo` and `PeerInfo`
//! types into a single, cohesive model.
//!
//! ## Design Rationale
//!
//! Previously, the system had two separate tracking mechanisms:
//! - `ContactInfo`: DID-based, mutual acceptance, profile info, no connection metrics
//! - `PeerInfo`: Endpoint-based, auto-discovered, connection metrics, no profile
//!
//! This caused UI confusion (separate sections) and duplicated concepts. The unified
//! `Peer` type uses `endpoint_id` as the primary key (required for network connectivity)
//! with optional DID identity and contact-specific details.
//!
//! ## Key Design Decisions
//!
//! 1. **endpoint_id as primary key** - Network connectivity requires this
//! 2. **Optional DID** - Discovered peers may not have verified identity yet
//! 3. **ContactDetails as Option** - A contact is just a peer with this field populated
//! 4. **Keep connection metrics** - Needed for reconnection backoff logic

use crate::invite::NodeAddrBytes;
use crate::types::contact::ProfileSnapshot;
use crate::types::RealmId;
use iroh::PublicKey;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// How a peer was discovered
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeerSource {
    /// Discovered through a specific realm's gossip topic
    FromRealm(RealmId),
    /// Added through an invite (not yet seen on gossip)
    FromInvite,
    /// Became a mutually accepted contact
    FromContact,
}

impl Default for PeerSource {
    fn default() -> Self {
        Self::FromInvite
    }
}

/// Connection status of a peer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PeerStatus {
    /// Currently connected (seen recently)
    Online,
    /// Not currently connected
    Offline,
    /// Status unknown (never attempted connection)
    #[default]
    Unknown,
}

impl std::fmt::Display for PeerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PeerStatus::Online => write!(f, "online"),
            PeerStatus::Offline => write!(f, "offline"),
            PeerStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// Contact-specific details (only for mutually accepted contacts)
///
/// When two peers mutually accept each other as contacts, they exchange
/// a dedicated gossip topic and encryption key for 1:1 communication.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContactDetails {
    /// Dedicated 1:1 gossip topic for this contact
    pub contact_topic: [u8; 32],
    /// Shared encryption key for messages
    pub contact_key: [u8; 32],
    /// Unix timestamp of mutual acceptance
    pub accepted_at: i64,
    /// Priority for auto-connect on startup
    pub is_favorite: bool,
}

impl ContactDetails {
    /// Create new contact details
    pub fn new(contact_topic: [u8; 32], contact_key: [u8; 32]) -> Self {
        Self {
            contact_topic,
            contact_key,
            accepted_at: chrono::Utc::now().timestamp(),
            is_favorite: false,
        }
    }

    /// Mark this contact as a favorite
    pub fn with_favorite(mut self, is_favorite: bool) -> Self {
        self.is_favorite = is_favorite;
        self
    }
}

/// Unified peer record - represents any known network participant
///
/// This struct consolidates the previous `ContactInfo` and `PeerInfo` types
/// into a single model. All peers have an endpoint_id (network identity),
/// with optional DID identity and contact details.
///
/// ## Example
///
/// ```rust
/// use syncengine_core::types::peer::{Peer, PeerSource, PeerStatus};
/// use iroh::SecretKey;
///
/// // Create a discovered peer
/// let secret = SecretKey::generate(&mut rand::rng());
/// let peer = Peer::new(secret.public(), PeerSource::FromRealm(Default::default()));
///
/// assert!(peer.did.is_none());          // No verified identity yet
/// assert!(peer.contact_info.is_none()); // Not a contact
/// assert!(peer.is_discovered());        // Just discovered via realm
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Peer {
    // === Core Identity (always present) ===
    /// Primary key - iroh public key (32 bytes)
    pub endpoint_id: [u8; 32],

    // === Optional DID Identity ===
    /// Set when identity is verified through contact exchange
    pub did: Option<String>,

    // === Profile Information ===
    /// Profile snapshot from contact exchange or gossip
    pub profile: Option<ProfileSnapshot>,
    /// User-set nickname as fallback display name
    pub nickname: Option<String>,

    // === Contact-Specific (only for mutually accepted) ===
    /// Present only for mutually accepted contacts
    pub contact_info: Option<ContactDetails>,

    // === Discovery & Connectivity ===
    /// How we discovered this peer
    pub source: PeerSource,
    /// List of realm IDs we share with this peer
    pub shared_realms: Vec<RealmId>,
    /// Last known network address
    pub node_addr: Option<NodeAddrBytes>,

    // === Status & Metrics ===
    /// Current connection status
    pub status: PeerStatus,
    /// When we last saw this peer (Unix timestamp)
    pub last_seen: u64,
    /// Total number of connection attempts
    #[serde(default)]
    pub connection_attempts: u32,
    /// Number of successful connections
    #[serde(default)]
    pub successful_connections: u32,
    /// Unix timestamp of last connection attempt
    #[serde(default)]
    pub last_attempt: u64,
}

impl Peer {
    /// Create a new peer with minimal information
    pub fn new(endpoint_id: PublicKey, source: PeerSource) -> Self {
        Self {
            endpoint_id: *endpoint_id.as_bytes(),
            did: None,
            profile: None,
            nickname: None,
            contact_info: None,
            source,
            shared_realms: Vec::new(),
            node_addr: None,
            status: PeerStatus::Unknown,
            last_seen: Self::current_timestamp(),
            connection_attempts: 0,
            successful_connections: 0,
            last_attempt: 0,
        }
    }

    /// Get current Unix timestamp
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Get the endpoint ID as a PublicKey
    pub fn public_key(&self) -> PublicKey {
        PublicKey::from_bytes(&self.endpoint_id).expect("stored endpoint_id should always be valid")
    }

    /// Check if this peer is a mutually accepted contact
    pub fn is_contact(&self) -> bool {
        self.contact_info.is_some()
    }

    /// Check if this peer is just discovered (not a contact)
    pub fn is_discovered(&self) -> bool {
        self.contact_info.is_none()
    }

    /// Get the display name for this peer (profile name > nickname > truncated endpoint)
    pub fn display_name(&self) -> String {
        if let Some(ref profile) = self.profile {
            return profile.display_name.clone();
        }
        if let Some(ref nickname) = self.nickname {
            return nickname.clone();
        }
        // Fallback: first 8 chars of hex endpoint_id
        format!("peer_{}", hex::encode(&self.endpoint_id[..4]))
    }

    /// Check if this peer has a verified DID identity
    pub fn has_verified_identity(&self) -> bool {
        self.did.is_some()
    }

    // === Builder methods ===

    /// Set the DID identity
    pub fn with_did(mut self, did: impl Into<String>) -> Self {
        self.did = Some(did.into());
        self
    }

    /// Set the profile snapshot
    pub fn with_profile(mut self, profile: ProfileSnapshot) -> Self {
        self.profile = Some(profile);
        self
    }

    /// Set the nickname
    pub fn with_nickname(mut self, nickname: impl Into<String>) -> Self {
        self.nickname = Some(nickname.into());
        self
    }

    /// Set contact details (promotes to contact status)
    pub fn with_contact_info(mut self, contact_info: ContactDetails) -> Self {
        self.contact_info = Some(contact_info);
        self.source = PeerSource::FromContact;
        self
    }

    /// Set the node address
    pub fn with_node_addr(mut self, node_addr: NodeAddrBytes) -> Self {
        self.node_addr = Some(node_addr);
        self
    }

    /// Set the status
    pub fn with_status(mut self, status: PeerStatus) -> Self {
        self.status = status;
        self
    }

    // === Mutation methods ===

    /// Update the last_seen timestamp to now
    pub fn touch(&mut self) {
        self.last_seen = Self::current_timestamp();
    }

    /// Add a realm to the shared_realms list if not already present
    pub fn add_realm(&mut self, realm_id: RealmId) {
        if !self.shared_realms.contains(&realm_id) {
            self.shared_realms.push(realm_id);
        }
    }

    /// Update profile information
    pub fn update_profile(&mut self, profile: ProfileSnapshot) {
        self.profile = Some(profile);
    }

    /// Set nickname
    pub fn set_nickname(&mut self, nickname: impl Into<String>) {
        self.nickname = Some(nickname.into());
    }

    /// Promote to contact with the given details
    pub fn promote_to_contact(&mut self, contact_info: ContactDetails) {
        self.contact_info = Some(contact_info);
        self.source = PeerSource::FromContact;
    }

    /// Toggle favorite status (only for contacts)
    pub fn toggle_favorite(&mut self) {
        if let Some(ref mut info) = self.contact_info {
            info.is_favorite = !info.is_favorite;
        }
    }

    /// Check if this contact is a favorite
    pub fn is_favorite(&self) -> bool {
        self.contact_info
            .as_ref()
            .map(|c| c.is_favorite)
            .unwrap_or(false)
    }

    // === Connection metrics ===

    /// Record a connection attempt
    pub fn record_attempt(&mut self) {
        self.connection_attempts += 1;
        self.last_attempt = Self::current_timestamp();
    }

    /// Record a successful connection
    pub fn record_success(&mut self) {
        self.successful_connections += 1;
        self.status = PeerStatus::Online;
        self.touch();
    }

    /// Record a connection failure
    pub fn record_failure(&mut self) {
        self.status = PeerStatus::Offline;
    }

    /// Calculate success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f64 {
        if self.connection_attempts == 0 {
            0.0
        } else {
            self.successful_connections as f64 / self.connection_attempts as f64
        }
    }

    /// Calculate the number of consecutive failures
    fn consecutive_failures(&self) -> u32 {
        if self.connection_attempts == 0 {
            return 0;
        }
        self.connection_attempts
            .saturating_sub(self.successful_connections)
    }

    /// Calculate Fibonacci backoff delay in seconds
    /// Sequence: 1min, 1min, 2min, 3min, 5min, 8min, 13min, 21min, 34min, 55min, capped at 60min
    pub fn backoff_delay(&self) -> u64 {
        let failures = self.consecutive_failures();
        let base_unit = 60u64; // 1 minute in seconds
        let max_delay = 3600u64; // 60 minutes in seconds

        let fib = Self::fibonacci(failures);
        let delay = fib.saturating_mul(base_unit);
        delay.min(max_delay)
    }

    /// Calculate the nth Fibonacci number efficiently (iterative)
    fn fibonacci(n: u32) -> u64 {
        match n {
            0 => 1,
            1 => 1,
            _ => {
                let mut a = 1u64;
                let mut b = 1u64;
                for _ in 2..=n {
                    let next = a.saturating_add(b);
                    a = b;
                    b = next;
                }
                b
            }
        }
    }

    /// Check if enough time has passed since last attempt to retry
    pub fn should_retry_now(&self) -> bool {
        if self.last_attempt == 0 {
            return true;
        }

        let now = Self::current_timestamp();
        let elapsed = now.saturating_sub(self.last_attempt);
        let required_delay = self.backoff_delay();

        elapsed >= required_delay
    }

    /// Check if contact was recently online (within 5 minutes)
    pub fn is_recently_active(&self) -> bool {
        let now = Self::current_timestamp();
        now.saturating_sub(self.last_seen) < 300 // 5 minutes
    }

    /// Mark as seen (for contact-compatible API)
    pub fn mark_seen(&mut self) {
        self.touch();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iroh::SecretKey;

    fn create_test_public_key() -> PublicKey {
        SecretKey::generate(&mut rand::rng()).public()
    }

    #[test]
    fn test_peer_creation() {
        let endpoint_id = create_test_public_key();
        let realm_id = RealmId::new();

        let peer = Peer::new(endpoint_id, PeerSource::FromRealm(realm_id));

        assert_eq!(peer.endpoint_id, *endpoint_id.as_bytes());
        assert_eq!(peer.status, PeerStatus::Unknown);
        assert!(peer.did.is_none());
        assert!(peer.profile.is_none());
        assert!(peer.nickname.is_none());
        assert!(peer.contact_info.is_none());
        assert!(peer.last_seen > 0);
    }

    #[test]
    fn test_peer_is_contact() {
        let endpoint_id = create_test_public_key();
        let mut peer = Peer::new(endpoint_id, PeerSource::FromInvite);

        assert!(!peer.is_contact());
        assert!(peer.is_discovered());

        // Promote to contact
        let contact_details = ContactDetails::new([1u8; 32], [2u8; 32]);
        peer.promote_to_contact(contact_details);

        assert!(peer.is_contact());
        assert!(!peer.is_discovered());
        assert_eq!(peer.source, PeerSource::FromContact);
    }

    #[test]
    fn test_peer_display_name() {
        let endpoint_id = create_test_public_key();
        let peer = Peer::new(endpoint_id, PeerSource::FromInvite);

        // Default: truncated endpoint
        let name = peer.display_name();
        assert!(name.starts_with("peer_"));

        // With nickname
        let peer = peer.with_nickname("Love");
        assert_eq!(peer.display_name(), "Love");

        // With profile (takes precedence)
        let profile = ProfileSnapshot {
            display_name: "Love Wonderland".to_string(),
            subtitle: None,
            avatar_blob_id: None,
            bio: String::new(),
        };
        let peer = peer.with_profile(profile);
        assert_eq!(peer.display_name(), "Love Wonderland");
    }

    #[test]
    fn test_peer_with_builders() {
        let endpoint_id = create_test_public_key();
        let contact_details = ContactDetails::new([1u8; 32], [2u8; 32]);

        let peer = Peer::new(endpoint_id, PeerSource::FromInvite)
            .with_did("did:sync:test123")
            .with_nickname("Joy")
            .with_contact_info(contact_details)
            .with_status(PeerStatus::Online);

        assert_eq!(peer.did, Some("did:sync:test123".to_string()));
        assert_eq!(peer.nickname, Some("Joy".to_string()));
        assert!(peer.is_contact());
        assert_eq!(peer.status, PeerStatus::Online);
    }

    #[test]
    fn test_peer_add_realm() {
        let endpoint_id = create_test_public_key();
        let mut peer = Peer::new(endpoint_id, PeerSource::FromInvite);

        let realm1 = RealmId::new();
        let realm2 = RealmId::new();

        peer.add_realm(realm1.clone());
        assert_eq!(peer.shared_realms.len(), 1);

        // Adding same realm again should not duplicate
        peer.add_realm(realm1.clone());
        assert_eq!(peer.shared_realms.len(), 1);

        peer.add_realm(realm2);
        assert_eq!(peer.shared_realms.len(), 2);
    }

    #[test]
    fn test_peer_favorite_toggle() {
        let endpoint_id = create_test_public_key();
        let contact_details = ContactDetails::new([1u8; 32], [2u8; 32]);
        let mut peer = Peer::new(endpoint_id, PeerSource::FromInvite)
            .with_contact_info(contact_details);

        assert!(!peer.is_favorite());

        peer.toggle_favorite();
        assert!(peer.is_favorite());

        peer.toggle_favorite();
        assert!(!peer.is_favorite());
    }

    #[test]
    fn test_peer_connection_metrics() {
        let endpoint_id = create_test_public_key();
        let mut peer = Peer::new(endpoint_id, PeerSource::FromInvite);

        assert_eq!(peer.connection_attempts, 0);
        assert_eq!(peer.successful_connections, 0);
        assert_eq!(peer.success_rate(), 0.0);

        peer.record_attempt();
        assert_eq!(peer.connection_attempts, 1);

        peer.record_success();
        assert_eq!(peer.successful_connections, 1);
        assert_eq!(peer.status, PeerStatus::Online);
        assert_eq!(peer.success_rate(), 1.0);

        peer.record_attempt();
        peer.record_failure();
        assert_eq!(peer.status, PeerStatus::Offline);
        assert_eq!(peer.success_rate(), 0.5);
    }

    #[test]
    fn test_peer_backoff_delay() {
        let endpoint_id = create_test_public_key();
        let mut peer = Peer::new(endpoint_id, PeerSource::FromInvite);

        // No failures
        assert_eq!(peer.backoff_delay(), 60); // 1 minute

        // Simulate failures
        peer.connection_attempts = 5;
        peer.successful_connections = 0;
        assert_eq!(peer.backoff_delay(), 480); // 8 minutes (F(5) = 8)

        // Cap at 60 minutes
        peer.connection_attempts = 20;
        assert_eq!(peer.backoff_delay(), 3600); // 60 minutes max
    }

    #[test]
    fn test_peer_should_retry_now() {
        let endpoint_id = create_test_public_key();
        let peer = Peer::new(endpoint_id, PeerSource::FromInvite);

        // Never attempted, can retry immediately
        assert!(peer.should_retry_now());
    }

    #[test]
    fn test_peer_recently_active() {
        let endpoint_id = create_test_public_key();
        let mut peer = Peer::new(endpoint_id, PeerSource::FromInvite);

        // Just created, should be recently active
        assert!(peer.is_recently_active());

        // Simulate old last_seen
        peer.last_seen = Peer::current_timestamp() - 600; // 10 minutes ago
        assert!(!peer.is_recently_active());
    }

    #[test]
    fn test_contact_details_creation() {
        let details = ContactDetails::new([1u8; 32], [2u8; 32]);

        assert_eq!(details.contact_topic, [1u8; 32]);
        assert_eq!(details.contact_key, [2u8; 32]);
        assert!(!details.is_favorite);
        assert!(details.accepted_at > 0);
    }

    #[test]
    fn test_contact_details_favorite() {
        let details = ContactDetails::new([1u8; 32], [2u8; 32]).with_favorite(true);
        assert!(details.is_favorite);
    }

    #[test]
    fn test_peer_status_display() {
        assert_eq!(PeerStatus::Online.to_string(), "online");
        assert_eq!(PeerStatus::Offline.to_string(), "offline");
        assert_eq!(PeerStatus::Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_peer_source_default() {
        assert_eq!(PeerSource::default(), PeerSource::FromInvite);
    }
}

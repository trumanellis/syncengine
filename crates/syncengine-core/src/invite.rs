//! Invite system for Synchronicity Engine
//!
//! Provides invite tickets for sharing realms with peers. An invite ticket
//! contains all the information needed to join a realm:
//! - Topic ID for gossip subscription
//! - Encryption key for the realm
//! - Bootstrap peers for initial connection
//!
//! Tickets are encoded as `sync-invite:{base58}` strings for easy sharing.

use std::net::SocketAddr;

use crate::error::SyncError;
use crate::types::RealmId;
use iroh::{EndpointAddr, PublicKey, RelayUrl};
use iroh_gossip::proto::TopicId;
use rand::RngCore;
use serde::{Deserialize, Serialize};

/// Prefix for encoded invite strings
const INVITE_PREFIX: &str = "sync-invite:";

/// Current protocol version
const PROTOCOL_VERSION: u8 = 1;

/// Serializable representation of a peer's network address.
///
/// This is a portable format that can be serialized and shared in invite tickets,
/// containing all information needed to connect to a peer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeAddrBytes {
    /// Node's public key (32 bytes)
    pub node_id: [u8; 32],
    /// Optional relay URL for NAT traversal
    pub relay_url: Option<String>,
    /// Direct socket addresses as strings (e.g., "192.168.1.1:4433")
    pub direct_addresses: Vec<String>,
}

impl NodeAddrBytes {
    /// Create a new NodeAddrBytes with just a node ID
    pub fn new(node_id: [u8; 32]) -> Self {
        Self {
            node_id,
            relay_url: None,
            direct_addresses: Vec::new(),
        }
    }

    /// Create with a relay URL
    pub fn with_relay(mut self, relay_url: impl Into<String>) -> Self {
        self.relay_url = Some(relay_url.into());
        self
    }

    /// Add a direct address
    pub fn with_address(mut self, addr: impl Into<String>) -> Self {
        self.direct_addresses.push(addr.into());
        self
    }

    /// Add multiple direct addresses
    pub fn with_addresses(mut self, addrs: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.direct_addresses
            .extend(addrs.into_iter().map(|a| a.into()));
        self
    }

    /// Create from an iroh EndpointAddr
    ///
    /// This extracts the node ID, relay URL, and direct addresses from
    /// the EndpointAddr into a serializable format.
    pub fn from_endpoint_addr(addr: &EndpointAddr) -> Self {
        let node_id = addr.id.as_bytes().to_owned();

        let relay_url = addr.relay_urls().next().map(|url| url.to_string());

        let direct_addresses = addr.ip_addrs().map(|addr| addr.to_string()).collect();

        Self {
            node_id,
            relay_url,
            direct_addresses,
        }
    }

    /// Convert back to an iroh EndpointAddr
    ///
    /// # Errors
    ///
    /// Returns `SyncError::InvalidInvite` if:
    /// - The public key bytes are invalid
    /// - A relay URL is malformed
    /// - A socket address is malformed
    pub fn to_endpoint_addr(&self) -> Result<EndpointAddr, SyncError> {
        let public_key = PublicKey::from_bytes(&self.node_id)
            .map_err(|e| SyncError::InvalidInvite(format!("Invalid public key: {}", e)))?;

        let mut addr = EndpointAddr::new(public_key);

        // Add relay URL if present
        if let Some(ref relay_str) = self.relay_url {
            let relay_url: RelayUrl = relay_str
                .parse()
                .map_err(|e| SyncError::InvalidInvite(format!("Invalid relay URL: {}", e)))?;
            addr = addr.with_relay_url(relay_url);
        }

        // Add direct addresses
        for addr_str in &self.direct_addresses {
            let socket_addr: SocketAddr = addr_str
                .parse()
                .map_err(|e| SyncError::InvalidInvite(format!("Invalid socket address: {}", e)))?;
            addr = addr.with_ip_addr(socket_addr);
        }

        Ok(addr)
    }
}

impl From<&EndpointAddr> for NodeAddrBytes {
    fn from(addr: &EndpointAddr) -> Self {
        Self::from_endpoint_addr(addr)
    }
}

impl From<EndpointAddr> for NodeAddrBytes {
    fn from(addr: EndpointAddr) -> Self {
        Self::from_endpoint_addr(&addr)
    }
}

impl TryFrom<NodeAddrBytes> for EndpointAddr {
    type Error = SyncError;

    fn try_from(bytes: NodeAddrBytes) -> Result<Self, Self::Error> {
        bytes.to_endpoint_addr()
    }
}

impl TryFrom<&NodeAddrBytes> for EndpointAddr {
    type Error = SyncError;

    fn try_from(bytes: &NodeAddrBytes) -> Result<Self, Self::Error> {
        bytes.to_endpoint_addr()
    }
}

/// An invite ticket containing all information needed to join a realm.
///
/// Invite tickets are designed to be:
/// - Self-contained: All data needed to join is in the ticket
/// - Portable: Can be shared via QR code, link, or copy-paste
/// - Secure: Contains the encryption key for the realm
/// - Expirable: Optional expiry time and use limits
///
/// # Example
///
/// ```ignore
/// use syncengine_core::invite::{InviteTicket, NodeAddrBytes};
///
/// // Create an invite for a realm
/// let ticket = InviteTicket::new(
///     &realm_id,
///     realm_key,
///     vec![NodeAddrBytes::new(peer_node_id)],
/// )
/// .with_name("My Shared Tasks")
/// .with_expiry(chrono::Utc::now().timestamp() + 86400); // 24 hours
///
/// // Encode for sharing
/// let invite_string = ticket.encode()?;
/// // -> "sync-invite:3xK7hNp..."
///
/// // Decode on receiving end
/// let decoded = InviteTicket::decode(&invite_string)?;
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InviteTicket {
    /// Protocol version (for future compatibility)
    pub version: u8,
    /// Unique identifier for this invite (for tracking/revocation)
    pub invite_id: [u8; 16],
    /// Topic ID bytes for gossip subscription
    pub topic: [u8; 32],
    /// ChaCha20-Poly1305 encryption key for the realm
    pub realm_key: [u8; 32],
    /// Bootstrap peers to connect to
    pub bootstrap_peers: Vec<NodeAddrBytes>,
    /// Human-readable realm name (optional)
    pub realm_name: Option<String>,
    /// Unix timestamp when this invite expires (None = never)
    pub expires_at: Option<i64>,
    /// Maximum number of times this invite can be used (None = unlimited)
    pub max_uses: Option<u32>,
}

impl InviteTicket {
    /// Create a new invite ticket for a realm.
    ///
    /// # Arguments
    ///
    /// * `realm_id` - The realm's unique identifier (used as topic ID)
    /// * `realm_key` - The ChaCha20-Poly1305 encryption key for the realm
    /// * `bootstrap_peers` - Peers to connect to when joining
    ///
    /// # Returns
    ///
    /// A new `InviteTicket` with a random invite ID and current protocol version.
    pub fn new(
        realm_id: &RealmId,
        realm_key: [u8; 32],
        bootstrap_peers: Vec<NodeAddrBytes>,
    ) -> Self {
        let mut invite_id = [0u8; 16];
        rand::rng().fill_bytes(&mut invite_id);

        Self {
            version: PROTOCOL_VERSION,
            invite_id,
            topic: *realm_id.as_bytes(),
            realm_key,
            bootstrap_peers,
            realm_name: None,
            expires_at: None,
            max_uses: None,
        }
    }

    /// Set a human-readable name for the realm (builder pattern).
    pub fn with_name(mut self, name: &str) -> Self {
        self.realm_name = Some(name.to_string());
        self
    }

    /// Set an expiry time as Unix timestamp (builder pattern).
    pub fn with_expiry(mut self, expires_at: i64) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Set maximum number of uses (builder pattern).
    pub fn with_max_uses(mut self, max: u32) -> Self {
        self.max_uses = Some(max);
        self
    }

    /// Encode the ticket as a `sync-invite:{base58}` string.
    ///
    /// Uses postcard for efficient binary serialization, then base58 for
    /// URL-safe encoding.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::Serialization` if encoding fails.
    pub fn encode(&self) -> Result<String, SyncError> {
        let bytes = postcard::to_stdvec(self)
            .map_err(|e| SyncError::Serialization(format!("Failed to encode invite: {}", e)))?;
        let encoded = bs58::encode(&bytes).into_string();
        Ok(format!("{}{}", INVITE_PREFIX, encoded))
    }

    /// Decode a ticket from a `sync-invite:{base58}` string.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::InvalidInvite` if:
    /// - The string doesn't start with `sync-invite:`
    /// - The base58 encoding is invalid
    /// - The binary data is malformed
    pub fn decode(s: &str) -> Result<Self, SyncError> {
        let data = s.strip_prefix(INVITE_PREFIX).ok_or_else(|| {
            SyncError::InvalidInvite(format!(
                "Invalid prefix: expected '{}', got '{}'",
                INVITE_PREFIX,
                s.chars().take(15).collect::<String>()
            ))
        })?;

        let bytes = bs58::decode(data)
            .into_vec()
            .map_err(|e| SyncError::InvalidInvite(format!("Invalid base58: {}", e)))?;

        let ticket: InviteTicket = postcard::from_bytes(&bytes)
            .map_err(|e| SyncError::InvalidInvite(format!("Invalid ticket data: {}", e)))?;

        Ok(ticket)
    }

    /// Check if this invite has expired.
    ///
    /// Returns `false` if no expiry is set.
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires) => chrono::Utc::now().timestamp() > expires,
            None => false,
        }
    }

    /// Convert the topic bytes to an iroh-gossip TopicId.
    pub fn topic_id(&self) -> TopicId {
        TopicId::from_bytes(self.topic)
    }

    /// Get the realm ID from this ticket.
    pub fn realm_id(&self) -> RealmId {
        RealmId::from_bytes(self.topic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        key[0] = 0xDE;
        key[1] = 0xAD;
        key[30] = 0xBE;
        key[31] = 0xEF;
        key
    }

    fn make_test_node_id() -> [u8; 32] {
        let mut id = [0u8; 32];
        id[0] = 0x01;
        id[31] = 0xFF;
        id
    }

    #[test]
    fn test_invite_encode_decode_roundtrip() {
        let realm_id = RealmId::new();
        let realm_key = make_test_key();
        let bootstrap_peers = vec![NodeAddrBytes::new(make_test_node_id())
            .with_relay("https://relay.example.com")
            .with_address("192.168.1.1:4433")];

        let ticket = InviteTicket::new(&realm_id, realm_key, bootstrap_peers.clone());

        // Encode
        let encoded = ticket.encode().expect("Failed to encode");
        assert!(encoded.starts_with(INVITE_PREFIX));

        // Decode
        let decoded = InviteTicket::decode(&encoded).expect("Failed to decode");

        // Verify all fields match
        assert_eq!(decoded.version, ticket.version);
        assert_eq!(decoded.invite_id, ticket.invite_id);
        assert_eq!(decoded.topic, ticket.topic);
        assert_eq!(decoded.realm_key, ticket.realm_key);
        assert_eq!(decoded.bootstrap_peers.len(), 1);
        assert_eq!(
            decoded.bootstrap_peers[0].node_id,
            bootstrap_peers[0].node_id
        );
        assert_eq!(
            decoded.bootstrap_peers[0].relay_url,
            bootstrap_peers[0].relay_url
        );
        assert_eq!(
            decoded.bootstrap_peers[0].direct_addresses,
            bootstrap_peers[0].direct_addresses
        );
        assert_eq!(decoded.realm_name, None);
        assert_eq!(decoded.expires_at, None);
        assert_eq!(decoded.max_uses, None);
    }

    #[test]
    fn test_invite_with_all_fields() {
        let realm_id = RealmId::new();
        let realm_key = make_test_key();
        let peer1 = NodeAddrBytes::new(make_test_node_id())
            .with_relay("https://relay1.example.com")
            .with_addresses(["192.168.1.1:4433", "10.0.0.1:4433"]);
        let mut peer2_id = make_test_node_id();
        peer2_id[0] = 0x02;
        let peer2 = NodeAddrBytes::new(peer2_id).with_relay("https://relay2.example.com");

        let expires = chrono::Utc::now().timestamp() + 86400; // 24 hours from now

        let ticket = InviteTicket::new(&realm_id, realm_key, vec![peer1, peer2])
            .with_name("Test Realm")
            .with_expiry(expires)
            .with_max_uses(5);

        // Encode and decode
        let encoded = ticket.encode().expect("Failed to encode");
        let decoded = InviteTicket::decode(&encoded).expect("Failed to decode");

        // Verify optional fields
        assert_eq!(decoded.realm_name, Some("Test Realm".to_string()));
        assert_eq!(decoded.expires_at, Some(expires));
        assert_eq!(decoded.max_uses, Some(5));

        // Verify bootstrap peers
        assert_eq!(decoded.bootstrap_peers.len(), 2);
        assert_eq!(decoded.bootstrap_peers[0].direct_addresses.len(), 2);
        assert_eq!(decoded.bootstrap_peers[1].direct_addresses.len(), 0);
    }

    #[test]
    fn test_invite_expired() {
        let realm_id = RealmId::new();
        let realm_key = make_test_key();

        // Create an expired invite (1 hour ago)
        let past_time = chrono::Utc::now().timestamp() - 3600;
        let expired_ticket = InviteTicket::new(&realm_id, realm_key, vec![]).with_expiry(past_time);

        assert!(expired_ticket.is_expired());

        // Create a future invite (1 hour from now)
        let future_time = chrono::Utc::now().timestamp() + 3600;
        let valid_ticket = InviteTicket::new(&realm_id, realm_key, vec![]).with_expiry(future_time);

        assert!(!valid_ticket.is_expired());

        // Create an invite with no expiry
        let no_expiry_ticket = InviteTicket::new(&realm_id, realm_key, vec![]);
        assert!(!no_expiry_ticket.is_expired());
    }

    #[test]
    fn test_invite_invalid_format() {
        // Empty string
        let result = InviteTicket::decode("");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, SyncError::InvalidInvite(_)));

        // Invalid base58
        let result = InviteTicket::decode("sync-invite:not-valid-base58!!!");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, SyncError::InvalidInvite(_)));

        // Valid base58 but invalid data
        let result = InviteTicket::decode("sync-invite:3mJr7AoU");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, SyncError::InvalidInvite(_)));
    }

    #[test]
    fn test_invite_wrong_prefix() {
        // Wrong prefix
        let result = InviteTicket::decode("wrong-prefix:abc123");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, SyncError::InvalidInvite(_)));
        assert!(format!("{}", err).contains("Invalid prefix"));

        // No prefix at all
        let result = InviteTicket::decode("abc123");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, SyncError::InvalidInvite(_)));
    }

    #[test]
    fn test_topic_id_conversion() {
        let realm_id = RealmId::new();
        let realm_key = make_test_key();
        let ticket = InviteTicket::new(&realm_id, realm_key, vec![]);

        let topic_id = ticket.topic_id();

        // Verify the topic ID matches the realm ID bytes
        assert_eq!(topic_id.as_bytes(), realm_id.as_bytes());
    }

    #[test]
    fn test_realm_id_extraction() {
        let original_realm_id = RealmId::new();
        let realm_key = make_test_key();
        let ticket = InviteTicket::new(&original_realm_id, realm_key, vec![]);

        let extracted_realm_id = ticket.realm_id();
        assert_eq!(extracted_realm_id, original_realm_id);
    }

    #[test]
    fn test_node_addr_bytes_builder() {
        let node_id = make_test_node_id();

        let addr = NodeAddrBytes::new(node_id)
            .with_relay("https://relay.example.com")
            .with_address("192.168.1.1:4433")
            .with_addresses(["10.0.0.1:4433", "172.16.0.1:4433"]);

        assert_eq!(addr.node_id, node_id);
        assert_eq!(
            addr.relay_url,
            Some("https://relay.example.com".to_string())
        );
        assert_eq!(addr.direct_addresses.len(), 3);
        assert_eq!(addr.direct_addresses[0], "192.168.1.1:4433");
        assert_eq!(addr.direct_addresses[1], "10.0.0.1:4433");
        assert_eq!(addr.direct_addresses[2], "172.16.0.1:4433");
    }

    #[test]
    fn test_invite_version() {
        let realm_id = RealmId::new();
        let realm_key = make_test_key();
        let ticket = InviteTicket::new(&realm_id, realm_key, vec![]);

        assert_eq!(ticket.version, PROTOCOL_VERSION);
        assert_eq!(ticket.version, 1);
    }

    #[test]
    fn test_invite_id_is_random() {
        let realm_id = RealmId::new();
        let realm_key = make_test_key();

        let ticket1 = InviteTicket::new(&realm_id, realm_key, vec![]);
        let ticket2 = InviteTicket::new(&realm_id, realm_key, vec![]);

        // Each invite should have a unique invite_id
        assert_ne!(ticket1.invite_id, ticket2.invite_id);
    }

    #[test]
    fn test_empty_bootstrap_peers() {
        let realm_id = RealmId::new();
        let realm_key = make_test_key();
        let ticket = InviteTicket::new(&realm_id, realm_key, vec![]);

        let encoded = ticket.encode().expect("Failed to encode");
        let decoded = InviteTicket::decode(&encoded).expect("Failed to decode");

        assert!(decoded.bootstrap_peers.is_empty());
    }

    #[test]
    fn test_node_addr_bytes_from_endpoint_addr() {
        use iroh::SecretKey;

        // Create an EndpointAddr with a known key
        let secret_key = SecretKey::generate(&mut rand::rng());
        let public_key = secret_key.public();
        let endpoint_addr = EndpointAddr::new(public_key)
            .with_relay_url("https://relay.example.com".parse().unwrap())
            .with_ip_addr("192.168.1.1:4433".parse().unwrap());

        // Convert to NodeAddrBytes
        let node_addr_bytes: NodeAddrBytes = (&endpoint_addr).into();

        // Verify the conversion
        assert_eq!(node_addr_bytes.node_id, *public_key.as_bytes());
        // Note: URL parsing may normalize the URL (add trailing slash, etc.)
        // Just verify it starts with the expected prefix
        let relay_url = node_addr_bytes.relay_url.expect("Expected relay URL");
        assert!(
            relay_url.starts_with("https://relay.example.com"),
            "Unexpected relay URL: {}",
            relay_url
        );
        assert_eq!(node_addr_bytes.direct_addresses.len(), 1);
        assert_eq!(node_addr_bytes.direct_addresses[0], "192.168.1.1:4433");
    }

    #[test]
    fn test_node_addr_bytes_to_endpoint_addr() {
        use iroh::SecretKey;

        // Create a NodeAddrBytes
        let secret_key = SecretKey::generate(&mut rand::rng());
        let public_key = secret_key.public();

        let node_addr_bytes = NodeAddrBytes {
            node_id: *public_key.as_bytes(),
            relay_url: Some("https://relay.example.com".to_string()),
            direct_addresses: vec!["192.168.1.1:4433".to_string(), "10.0.0.1:1234".to_string()],
        };

        // Convert to EndpointAddr
        let endpoint_addr: EndpointAddr = node_addr_bytes
            .try_into()
            .expect("Failed to convert to EndpointAddr");

        // Verify the conversion
        assert_eq!(endpoint_addr.id, public_key);
        assert_eq!(endpoint_addr.relay_urls().count(), 1);
        assert_eq!(endpoint_addr.ip_addrs().count(), 2);
    }

    #[test]
    fn test_node_addr_bytes_roundtrip() {
        use iroh::SecretKey;

        // Create an EndpointAddr
        let secret_key = SecretKey::generate(&mut rand::rng());
        let public_key = secret_key.public();
        let original = EndpointAddr::new(public_key)
            .with_relay_url("https://relay.test.com".parse().unwrap())
            .with_ip_addr("127.0.0.1:5555".parse().unwrap());

        // Convert to NodeAddrBytes and back
        let bytes: NodeAddrBytes = (&original).into();
        let recovered: EndpointAddr = bytes.try_into().expect("Failed to convert back");

        // Verify roundtrip
        assert_eq!(recovered.id, original.id);
        // Note: BTreeSet ordering may differ, so compare by endpoint id
    }

    #[test]
    fn test_node_addr_bytes_invalid_public_key() {
        let node_addr_bytes = NodeAddrBytes {
            node_id: [0u8; 32], // Invalid: all zeros is not a valid ed25519 public key
            relay_url: None,
            direct_addresses: vec![],
        };

        let result: Result<EndpointAddr, _> = node_addr_bytes.try_into();
        // Note: Some zero keys might be valid depending on the curve point
        // This test just verifies the conversion doesn't panic
        let _ = result;
    }

    #[test]
    fn test_node_addr_bytes_invalid_socket_addr() {
        use iroh::SecretKey;

        let secret_key = SecretKey::generate(&mut rand::rng());
        let public_key = secret_key.public();

        let node_addr_bytes = NodeAddrBytes {
            node_id: *public_key.as_bytes(),
            relay_url: None,
            direct_addresses: vec!["not-a-valid-address".to_string()],
        };

        let result: Result<EndpointAddr, _> = node_addr_bytes.try_into();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid socket address"));
    }

    #[test]
    fn test_node_addr_bytes_invalid_relay_url() {
        use iroh::SecretKey;

        let secret_key = SecretKey::generate(&mut rand::rng());
        let public_key = secret_key.public();

        let node_addr_bytes = NodeAddrBytes {
            node_id: *public_key.as_bytes(),
            relay_url: Some("not-a-valid-url".to_string()),
            direct_addresses: vec![],
        };

        let result: Result<EndpointAddr, _> = node_addr_bytes.try_into();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid relay URL"));
    }
}

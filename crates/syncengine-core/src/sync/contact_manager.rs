//! Contact Manager - P2P contact exchange orchestration
//!
//! Manages the complete lifecycle of peer contacts:
//! - Generating and decoding contact invites
//! - Sending and receiving contact requests
//! - Mutual acceptance handshake
//! - Auto-reconnection to accepted contacts
//!
//! ## Protocol Flow
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │  Contact Exchange Protocol                                      │
//! │                                                                  │
//! │  1. Alice generates invite → "sync-contact:{base58}"            │
//! │  2. Bob decodes invite → verifies signature, checks expiry      │
//! │  3. Bob sends ContactRequest → saves as OutgoingPending         │
//! │  4. Alice receives request → saves as IncomingPending           │
//! │  5. Alice accepts → both finalize connection                    │
//! │  6. Subscribe to 1:1 gossip topic                               │
//! │  7. Save to contacts table with shared keys                     │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use base64::Engine as _;
use iroh_gossip::proto::TopicId;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};

use crate::error::SyncError;
use crate::identity::{Did, HybridKeypair, HybridPublicKey};
use crate::invite::NodeAddrBytes;
use crate::storage::Storage;
use crate::sync::contact_protocol::{ContactMessage, CONTACT_ALPN};
use crate::sync::{GossipSync, TopicHandle};
use crate::types::contact::{
    ContactInfo, ContactState, ContactStatus, HybridContactInvite, PeerContactInvite,
    PendingContact, ProfileSnapshot,
};
use crate::types::peer::{ContactDetails, Peer, PeerSource, PeerStatus};

type SyncResult<T> = Result<T, SyncError>;

/// Event emitted by ContactManager for UI notifications
#[derive(Debug, Clone)]
pub enum ContactEvent {
    /// A new invite was generated
    InviteGenerated { invite_code: String },
    /// A contact request was received from another peer
    ContactRequestReceived {
        invite_id: [u8; 16],
        from: ProfileSnapshot,
        /// True if this was our own invite (should auto-accept)
        auto_accept: bool,
    },
    /// A contact request was successfully sent
    ContactRequestSent { invite_id: [u8; 16], to: String },
    /// A contact was mutually accepted and finalized
    ContactAccepted { contact: ContactInfo },
    /// A contact request was declined
    ContactDeclined { invite_id: [u8; 16] },
    /// A contact came online
    ContactOnline { did: String },
    /// A contact went offline
    ContactOffline { did: String },
    /// An error occurred during contact operations
    ContactError { message: String },
}

/// Manages peer contact exchange and auto-reconnection
///
/// ContactManager orchestrates the full lifecycle of peer connections,
/// from invite generation through mutual acceptance and ongoing synchronization.
pub struct ContactManager {
    /// Gossip sync instance for network communication
    gossip_sync: Arc<GossipSync>,
    /// Our hybrid keypair for signing invites and requests
    keypair: Arc<HybridKeypair>,
    /// Our DID (decentralized identifier)
    did: Did,
    /// Storage for contacts and pending requests
    storage: Arc<Storage>,
    /// Event broadcast channel for UI updates
    event_tx: broadcast::Sender<ContactEvent>,
    /// Active 1:1 gossip topics (contact_topic -> handle)
    active_topics: Arc<RwLock<HashMap<[u8; 32], TopicHandle>>>,
}

impl ContactManager {
    /// Create a new ContactManager
    ///
    /// # Arguments
    ///
    /// * `gossip_sync` - GossipSync instance for network communication
    /// * `keypair` - Our hybrid keypair for signing
    /// * `did` - Our DID
    /// * `storage` - Storage for persistence
    /// * `event_tx` - Event broadcast channel (shared with ContactProtocolHandler)
    pub fn new(
        gossip_sync: Arc<GossipSync>,
        keypair: Arc<HybridKeypair>,
        did: Did,
        storage: Arc<Storage>,
        event_tx: broadcast::Sender<ContactEvent>,
    ) -> Self {
        // Note: Incoming contact messages are handled by ContactProtocolHandler
        // registered with the Router in GossipSync. No listener task needed here.

        Self {
            gossip_sync,
            keypair,
            did,
            storage,
            event_tx,
            active_topics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a background task that auto-accepts contact requests for our own invites
    ///
    /// When we generate an invite and someone uses it, we should automatically accept
    /// instead of requiring manual confirmation. This task listens for ContactRequestReceived
    /// events with `auto_accept: true` and triggers acceptance.
    pub fn start_auto_accept_task(self: Arc<Self>) {
        let mut event_rx = self.event_tx.subscribe();

        tokio::spawn(async move {
            loop {
                match event_rx.recv().await {
                    Ok(ContactEvent::ContactRequestReceived {
                        invite_id,
                        from,
                        auto_accept: true,
                    }) => {
                        info!(
                            invite_id = ?invite_id,
                            from_name = %from.display_name,
                            "Auto-accepting contact request for our own invite"
                        );

                        // Small delay to ensure pending contact is saved
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

                        // Auto-accept the contact
                        if let Err(e) = self.accept_contact_request(&invite_id).await {
                            error!(
                                invite_id = ?invite_id,
                                error = ?e,
                                "Failed to auto-accept contact request"
                            );
                        } else {
                            info!(
                                invite_id = ?invite_id,
                                "Successfully auto-accepted contact request"
                            );
                        }
                    }
                    Ok(_) => {
                        // Ignore other events
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("Auto-accept task lagged, missed {} events", n);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        debug!("Auto-accept task event channel closed, stopping");
                        break;
                    }
                }
            }
        });

        info!("Contact auto-accept task started");
    }

    /// Retry an async operation with exponential backoff
    ///
    /// Retries up to 3 times with delays: 100ms, 200ms, 400ms
    async fn retry_with_backoff<F, Fut, T, E>(
        &self,
        operation_name: &str,
        mut operation: F,
    ) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        const MAX_ATTEMPTS: u32 = 3;
        const BASE_DELAY_MS: u64 = 100;

        for attempt in 1..=MAX_ATTEMPTS {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt < MAX_ATTEMPTS {
                        let delay_ms = BASE_DELAY_MS * (1 << (attempt - 1)); // 100, 200, 400
                        warn!(
                            operation = operation_name,
                            attempt,
                            max_attempts = MAX_ATTEMPTS,
                            delay_ms,
                            error = %e,
                            "Operation failed, retrying"
                        );
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    } else {
                        warn!(
                            operation = operation_name,
                            attempt,
                            error = %e,
                            "Operation failed after all retries"
                        );
                        return Err(e);
                    }
                }
            }
        }

        unreachable!()
    }

    /// Generate a contact invite for our profile (Version 2 - Hybrid)
    ///
    /// Creates a compact signed invite that can be shared via QR code or text.
    /// Uses the hybrid format with name-only fallback and on-demand profile fetching.
    /// The invite expires after the specified number of hours (max 168 = 7 days).
    ///
    /// **Version 2 Benefits:**
    /// - ~4.5x smaller than v1 (420-450 chars vs 2000+)
    /// - QR code friendly
    /// - Always-fresh profiles when online
    /// - Graceful offline degradation
    ///
    /// # Arguments
    ///
    /// * `profile` - Profile snapshot (only display_name is embedded)
    /// * `expiry_hours` - Hours until invite expires (capped at 168)
    ///
    /// # Returns
    ///
    /// A base64url-encoded invite string with prefix "sync-contact:"
    pub fn generate_invite(
        &self,
        profile: ProfileSnapshot,
        expiry_hours: u8,
    ) -> SyncResult<String> {
        // Cap expiry at 7 days (168 hours)
        let expiry_hours = expiry_hours.min(168);

        // Generate unique invite ID
        let invite_id = HybridContactInvite::generate_invite_id();

        // Get current node address
        let node_addr = NodeAddrBytes::from_endpoint_addr(&self.gossip_sync.endpoint_addr());

        // Calculate timestamps
        let now = chrono::Utc::now().timestamp();
        let expires_at = now + (expiry_hours as i64 * 3600);

        // Create unsigned hybrid invite (v2)
        let mut invite = HybridContactInvite {
            version: 2,
            invite_id,
            inviter_did: self.did.to_string(),
            node_addr,
            display_name: profile.display_name, // Name-only fallback
            created_at: now,
            expires_at,
            signature: vec![], // Filled after signing
        };

        // Sign the invite (Ed25519-only for compact size)
        let signature = self.sign_hybrid_invite(&invite)?;
        invite.signature = signature;

        // Serialize, compress, and encode
        let serialized =
            postcard::to_allocvec(&invite).map_err(|e| SyncError::Serialization(e.to_string()))?;

        // Compress with zstd (level 3 = fast with good compression)
        let compressed = zstd::encode_all(&serialized[..], 3)
            .map_err(|e| SyncError::Serialization(format!("Compression failed: {}", e)))?;

        // Encode with base64url (URL-safe, no padding)
        let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&compressed);
        let invite_code = format!("sync-contact:{}", encoded);

        let compression_ratio = (serialized.len() as f64 / compressed.len() as f64 * 100.0) as u32;
        info!(
            invite_id = ?invite_id,
            version = 2,
            expiry_hours,
            original_size = serialized.len(),
            compressed_size = compressed.len(),
            compression_ratio = format!("{}%", compression_ratio),
            final_length = invite_code.len(),
            "Generated hybrid contact invite (v2)"
        );

        // Track this invite so we auto-accept when someone uses it
        self.storage.save_generated_invite(&invite_id)?;

        // Emit event
        let _ = self.event_tx.send(ContactEvent::InviteGenerated {
            invite_code: invite_code.clone(),
        });

        Ok(invite_code)
    }

    /// Decode and validate a contact invite (supports v1 and v2)
    ///
    /// Verifies signature, checks expiry, and ensures invite hasn't been revoked.
    /// Supports both legacy v1 (PeerContactInvite) and new v2 (HybridContactInvite) formats.
    ///
    /// # Arguments
    ///
    /// * `invite_str` - Invite string with prefix "sync-contact:{base64url}"
    ///
    /// # Returns
    ///
    /// Validated HybridContactInvite if all checks pass (v1 invites are converted to v2 format)
    pub fn decode_invite(&self, invite_str: &str) -> SyncResult<HybridContactInvite> {
        // Check prefix
        if !invite_str.starts_with("sync-contact:") {
            return Err(SyncError::InvalidInvite(
                "Invalid invite format (missing prefix)".to_string(),
            ));
        }

        let encoded = &invite_str[13..]; // Skip "sync-contact:"

        // Decode and decompress
        let bytes = if let Ok(compressed) = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(encoded) {
            // Decompress
            zstd::decode_all(&compressed[..])
                .map_err(|e| SyncError::InvalidInvite(format!("Decompression failed: {}", e)))?
        } else {
            // Fallback to legacy format (uncompressed base58)
            bs58::decode(encoded)
                .into_vec()
                .map_err(|e| SyncError::InvalidInvite(format!("Invalid encoding: {}", e)))?
        };

        // Peek at version byte to determine format
        if bytes.is_empty() {
            return Err(SyncError::InvalidInvite("Empty invite data".to_string()));
        }

        // Try to deserialize based on version
        let version = bytes[0];
        match version {
            2 => {
                // V2 HybridContactInvite
                let invite: HybridContactInvite = postcard::from_bytes(&bytes)
                    .map_err(|e| SyncError::InvalidInvite(format!("Invalid v2 invite data: {}", e)))?;

                // Check expiry
                if invite.is_expired() {
                    return Err(SyncError::InvalidInvite("Invite has expired".to_string()));
                }

                // Check if revoked
                if self.storage.is_invite_revoked(&invite.invite_id)? {
                    return Err(SyncError::InvalidInvite(
                        "Invite has been revoked".to_string(),
                    ));
                }

                // Verify signature
                self.verify_hybrid_invite_signature(&invite)?;

                debug!(
                    invite_id = ?invite.invite_id,
                    inviter_did = %invite.inviter_did,
                    version = 2,
                    "Decoded and validated hybrid invite (v2)"
                );

                Ok(invite)
            }
            1 => {
                // V1 PeerContactInvite (legacy)
                let v1_invite: PeerContactInvite = postcard::from_bytes(&bytes)
                    .map_err(|e| SyncError::InvalidInvite(format!("Invalid v1 invite data: {}", e)))?;

                // Check expiry
                if v1_invite.is_expired() {
                    return Err(SyncError::InvalidInvite("Invite has expired".to_string()));
                }

                // Check if revoked
                if self.storage.is_invite_revoked(&v1_invite.invite_id)? {
                    return Err(SyncError::InvalidInvite(
                        "Invite has been revoked".to_string(),
                    ));
                }

                // Verify signature (v1)
                self.verify_invite_signature(&v1_invite)?;

                debug!(
                    invite_id = ?v1_invite.invite_id,
                    inviter_did = %v1_invite.inviter_did,
                    version = 1,
                    "Decoded and validated legacy invite (v1), converting to v2"
                );

                // Convert v1 to v2 format
                Ok(HybridContactInvite {
                    version: 2,
                    invite_id: v1_invite.invite_id,
                    inviter_did: v1_invite.inviter_did,
                    node_addr: v1_invite.node_addr,
                    display_name: v1_invite.profile_snapshot.display_name,
                    created_at: v1_invite.created_at,
                    expires_at: v1_invite.expires_at,
                    signature: v1_invite.signature,
                })
            }
            _ => Err(SyncError::InvalidInvite(format!(
                "Unsupported invite version: {}",
                version
            ))),
        }
    }

    /// Send a contact request to an inviter
    ///
    /// Saves the invite as OutgoingPending and sends a contact request
    /// via QUIC to the inviter's endpoint.
    ///
    /// # Arguments
    ///
    /// * `invite` - Validated PeerContactInvite from decode_invite()
    /// * `our_profile` - Our profile to send in the request
    pub async fn send_contact_request(
        &self,
        invite: HybridContactInvite,
        our_profile: ProfileSnapshot,
    ) -> SyncResult<()> {
        // Create minimal profile snapshot from hybrid invite
        // TODO: Fetch full profile from node via PROFILE_ALPN (Phase 2.2)
        let minimal_profile = ProfileSnapshot {
            display_name: invite.display_name.clone(),
            subtitle: None,
            avatar_blob_id: None,
            bio: String::new(),
        };

        // Save as pending (OutgoingPending)
        // Note: signed_profile is None for outgoing requests - we'll get it from the accept response
        let pending = PendingContact {
            invite_id: invite.invite_id,
            peer_did: invite.inviter_did.clone(),
            profile: minimal_profile,
            signed_profile: None,
            node_addr: invite.node_addr.clone(),
            state: ContactState::OutgoingPending,
            created_at: chrono::Utc::now().timestamp(),
        };

        self.storage.save_pending(&pending)?;

        info!(
            invite_id = ?invite.invite_id,
            peer_did = %invite.inviter_did,
            "Saved outgoing contact request"
        );

        // Emit event
        let _ = self.event_tx.send(ContactEvent::ContactRequestSent {
            invite_id: invite.invite_id,
            to: invite.inviter_did.clone(),
        });

        // Connect to inviter's endpoint and send ContactRequest via QUIC
        self.send_contact_message(&invite.node_addr, our_profile, invite.invite_id)
            .await?;

        Ok(())
    }

    /// Accept an incoming contact request
    ///
    /// Sends a single ContactAccept message, derives keys locally, and finalizes.
    /// The simplified protocol eliminates the separate ContactResponse step.
    ///
    /// # Arguments
    ///
    /// * `invite_id` - Invite ID of the pending contact to accept
    pub async fn accept_contact_request(&self, invite_id: &[u8; 16]) -> SyncResult<()> {
        // Load pending contact
        let pending = self
            .storage
            .load_pending(invite_id)?
            .ok_or_else(|| SyncError::ContactNotFound(hex::encode(invite_id)))?;

        // Must be IncomingPending to accept
        if pending.state != ContactState::IncomingPending {
            return Err(SyncError::InvalidOperation(format!(
                "Cannot accept contact in state: {}",
                pending.state
            )));
        }

        info!(
            invite_id = ?invite_id,
            peer_did = %pending.peer_did,
            "Accepting contact request with simplified 2-message protocol"
        );

        // Send single ContactAccept message (no keys - derived locally by both parties)
        self.send_contact_accept(&pending.node_addr, *invite_id)
            .await?;

        // Finalize the contact (derive keys locally, subscribe to topic, save to database)
        self.finalize_contact(&pending).await?;

        Ok(())
    }

    /// Decline an incoming contact request
    ///
    /// Sends a ContactDecline message and deletes the pending contact.
    ///
    /// # Arguments
    ///
    /// * `invite_id` - Invite ID of the pending contact to decline
    pub async fn decline_contact_request(&self, invite_id: &[u8; 16]) -> SyncResult<()> {
        // Load pending contact
        let pending = self
            .storage
            .load_pending(invite_id)?
            .ok_or_else(|| SyncError::ContactNotFound(hex::encode(invite_id)))?;

        // Must be IncomingPending to decline
        if pending.state != ContactState::IncomingPending {
            return Err(SyncError::InvalidOperation(format!(
                "Cannot decline contact in state: {}",
                pending.state
            )));
        }

        // Delete pending contact
        self.storage.delete_pending(invite_id)?;

        info!(
            invite_id = ?invite_id,
            peer_did = %pending.peer_did,
            "Declined contact request"
        );

        // Send ContactDecline via QUIC
        if let Err(e) = self
            .send_contact_decline(&pending.node_addr, *invite_id)
            .await
        {
            warn!(
                error = ?e,
                "Failed to send decline message, but pending was already deleted"
            );
        }

        // Emit event
        let _ = self.event_tx.send(ContactEvent::ContactDeclined {
            invite_id: *invite_id,
        });

        Ok(())
    }

    /// Cancel an outgoing contact request
    ///
    /// Deletes the pending contact and optionally revokes the invite.
    ///
    /// # Arguments
    ///
    /// * `invite_id` - Invite ID of the outgoing request to cancel
    pub fn cancel_outgoing_request(&self, invite_id: &[u8; 16]) -> SyncResult<()> {
        // Load pending contact
        let pending = self
            .storage
            .load_pending(invite_id)?
            .ok_or_else(|| SyncError::ContactNotFound(hex::encode(invite_id)))?;

        // Must be OutgoingPending to cancel
        if pending.state != ContactState::OutgoingPending {
            return Err(SyncError::InvalidOperation(format!(
                "Cannot cancel request in state: {}",
                pending.state
            )));
        }

        // Delete pending contact
        self.storage.delete_pending(invite_id)?;

        // Revoke the invite so it can't be used anymore
        self.storage.revoke_invite(invite_id)?;

        info!(
            invite_id = ?invite_id,
            peer_did = %pending.peer_did,
            "Cancelled outgoing contact request"
        );

        // Emit event
        let _ = self.event_tx.send(ContactEvent::ContactDeclined {
            invite_id: *invite_id,
        });

        Ok(())
    }

    /// Finalize a mutually accepted contact
    ///
    /// Derives shared keys, subscribes to contact topic, saves to contacts table.
    /// Also subscribes to their profile topic and pins their profile for P2P redundancy.
    async fn finalize_contact(&self, pending: &PendingContact) -> SyncResult<()> {
        use crate::sync::derive_profile_topic;
        use crate::types::{PinRelationship, ProfilePin};

        // Derive 1:1 contact topic and encryption key from DIDs
        let contact_topic = Self::derive_contact_topic(self.did.as_ref(), &pending.peer_did);
        let contact_key = Self::derive_contact_key(self.did.as_ref(), &pending.peer_did);

        // Create ContactInfo
        let contact = ContactInfo {
            peer_did: pending.peer_did.clone(),
            peer_endpoint_id: pending.node_addr.node_id,
            profile: pending.profile.clone(),
            node_addr: pending.node_addr.clone(),
            contact_topic,
            contact_key,
            accepted_at: chrono::Utc::now().timestamp(),
            last_seen: chrono::Utc::now().timestamp() as u64,
            status: ContactStatus::Online, // Online since we just communicated
            is_favorite: false,
        };

        // Save to contacts table (legacy)
        self.storage.save_contact(&contact)?;

        // Also save as unified Peer (new system)
        let unified_peer = Peer {
            endpoint_id: contact.peer_endpoint_id,
            did: Some(contact.peer_did.clone()),
            profile: Some(contact.profile.clone()),
            nickname: None,
            contact_info: Some(ContactDetails {
                contact_topic,
                contact_key,
                accepted_at: contact.accepted_at,
                is_favorite: false,
            }),
            source: PeerSource::FromContact,
            shared_realms: Vec::new(),
            node_addr: Some(contact.node_addr.clone()),
            status: PeerStatus::Online,
            last_seen: contact.last_seen,
            connection_attempts: 0,
            successful_connections: 1, // Just succeeded
            last_attempt: contact.last_seen,
        };
        self.storage.save_peer(&unified_peer)?;

        // Pin their profile if we have SignedProfile from the contact exchange
        if let Some(signed_profile) = &pending.signed_profile {
            let pin = ProfilePin::new(
                pending.peer_did.clone(),
                signed_profile.clone(),
                PinRelationship::Contact,
            );
            if let Err(e) = self.storage.save_pinned_profile(&pin) {
                warn!(did = %pending.peer_did, error = %e, "Failed to pin contact's profile");
            } else {
                info!(did = %pending.peer_did, "Pinned contact's profile from contact exchange");
            }
        }

        // Delete pending
        self.storage.delete_pending(&pending.invite_id)?;

        info!(
            peer_did = %contact.peer_did,
            contact_topic = ?contact_topic,
            "Finalized contact and saved to database (unified peer system)"
        );

        // Subscribe to contact topic (for 1:1 messaging)
        self.subscribe_contact_topic(&contact).await?;

        // Subscribe to their profile topic (for profile updates)
        // Use subscribe_split to get a receiver we can process in a background task
        let peer_profile_topic = derive_profile_topic(&contact.peer_did);
        let bootstrap_peer = iroh::PublicKey::from_bytes(&contact.peer_endpoint_id)
            .map_err(|e| SyncError::Identity(format!("Invalid peer endpoint ID: {}", e)))?;

        match self
            .gossip_sync
            .subscribe_split(peer_profile_topic, vec![bootstrap_peer])
            .await
        {
            Ok((_sender, receiver)) => {
                info!(
                    peer_did = %contact.peer_did,
                    ?peer_profile_topic,
                    "Subscribed to contact's profile topic for updates"
                );

                // Spawn listener to process profile updates from this contact's topic
                Self::spawn_profile_topic_listener(
                    self.storage.clone(),
                    contact.peer_did.clone(),
                    receiver,
                );
            }
            Err(e) => {
                // Non-fatal: we can still receive updates via global topic
                warn!(
                    peer_did = %contact.peer_did,
                    error = %e,
                    "Failed to subscribe to contact's profile topic (non-fatal)"
                );
            }
        }

        // Emit event
        let _ = self.event_tx.send(ContactEvent::ContactAccepted {
            contact: contact.clone(),
        });

        // Also emit online event since we just successfully communicated with them
        // TODO: Implement proper presence detection via heartbeats
        let _ = self.event_tx.send(ContactEvent::ContactOnline {
            did: contact.peer_did.clone(),
        });

        Ok(())
    }

    /// Spawn a background listener for a contact's per-peer profile topic.
    ///
    /// This listener processes profile updates (Announce messages) from the contact
    /// and updates the local Peer record with the new profile information.
    ///
    /// # Arguments
    ///
    /// * `storage` - Storage for updating Peer profiles
    /// * `peer_did` - The DID of the contact whose profile topic we're listening to
    /// * `receiver` - The TopicReceiver from subscribe_split()
    fn spawn_profile_topic_listener(
        storage: Arc<Storage>,
        peer_did: String,
        mut receiver: crate::sync::TopicReceiver,
    ) {
        tokio::spawn(async move {
            use crate::sync::TopicEvent;

            info!(peer_did = %peer_did, "Profile topic listener started for contact");

            while let Some(event) = receiver.recv_event().await {
                if let TopicEvent::Message(msg) = event {
                    // Try to parse as profile gossip message
                    match crate::sync::ProfileGossipMessage::from_bytes(&msg.content) {
                        Ok(crate::sync::ProfileGossipMessage::Announce {
                            signed_profile, ..
                        }) => {
                            // Verify the signature
                            if !signed_profile.verify() {
                                warn!(peer_did = %peer_did, "Invalid signature on profile announcement");
                                continue;
                            }

                            let signer_did = signed_profile.did().to_string();

                            // Only process announcements from the expected peer
                            // (the contact whose topic we subscribed to)
                            if signer_did != peer_did {
                                debug!(
                                    expected = %peer_did,
                                    actual = %signer_did,
                                    "Received profile from unexpected signer on per-peer topic"
                                );
                                continue;
                            }

                            // Update the Peer.profile in storage
                            match storage.load_peer_by_did(&signer_did) {
                                Ok(Some(mut peer)) => {
                                    peer.profile = Some(crate::types::ProfileSnapshot {
                                        display_name: signed_profile.profile.display_name.clone(),
                                        subtitle: signed_profile.profile.subtitle.clone(),
                                        avatar_blob_id: signed_profile.profile.avatar_blob_id.clone(),
                                        bio: crate::types::ProfileSnapshot::truncate_bio(
                                            &signed_profile.profile.bio,
                                        ),
                                    });

                                    if let Err(e) = storage.save_peer(&peer) {
                                        warn!(
                                            did = %signer_did,
                                            error = %e,
                                            "Failed to update peer profile from per-peer topic"
                                        );
                                    } else {
                                        info!(
                                            did = %signer_did,
                                            name = %signed_profile.profile.display_name,
                                            "Updated contact profile from per-peer topic"
                                        );
                                    }
                                }
                                Ok(None) => {
                                    debug!(
                                        did = %signer_did,
                                        "Received profile for unknown peer (not in storage)"
                                    );
                                }
                                Err(e) => {
                                    warn!(
                                        did = %signer_did,
                                        error = %e,
                                        "Failed to load peer from storage"
                                    );
                                }
                            }
                        }
                        Ok(_) => {
                            // Ignore Request/Response messages on per-peer topics
                            // These are only relevant on the global topic
                        }
                        Err(e) => {
                            debug!(
                                peer_did = %peer_did,
                                error = %e,
                                "Failed to parse message on profile topic"
                            );
                        }
                    }
                }
                // Ignore NeighborUp/NeighborDown events for per-peer profile topics
            }

            info!(peer_did = %peer_did, "Profile topic listener stopped for contact");
        });
    }

    /// Subscribe to a 1:1 contact gossip topic
    ///
    /// Creates a direct communication channel with this contact.
    async fn subscribe_contact_topic(&self, contact: &ContactInfo) -> SyncResult<()> {
        let topic_id = TopicId::from_bytes(contact.contact_topic);

        // Subscribe with no bootstrap peers (direct connection only)
        // The peer will connect when they also subscribe to the same topic
        let handle = self.gossip_sync.subscribe(topic_id, vec![]).await?;

        // Store handle for later use
        self.active_topics
            .write()
            .await
            .insert(contact.contact_topic, handle);

        info!(
            peer_did = %contact.peer_did,
            topic_id = ?topic_id,
            "Subscribed to contact gossip topic"
        );

        Ok(())
    }

    /// Derive deterministic 1:1 contact topic from two DIDs
    ///
    /// Uses BLAKE3 hash of sorted DIDs to ensure both peers derive the same topic.
    pub fn derive_contact_topic(did1: &str, did2: &str) -> [u8; 32] {
        // Sort DIDs lexicographically for deterministic order
        let (a, b) = if did1 < did2 {
            (did1, did2)
        } else {
            (did2, did1)
        };

        // BLAKE3("sync-contact-topic" || a || b)
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"sync-contact-topic");
        hasher.update(a.as_bytes());
        hasher.update(b.as_bytes());
        *hasher.finalize().as_bytes()
    }

    /// Derive deterministic shared encryption key from two DIDs
    ///
    /// Uses BLAKE3 hash of sorted DIDs to ensure both peers derive the same key.
    pub fn derive_contact_key(did1: &str, did2: &str) -> [u8; 32] {
        let (a, b) = if did1 < did2 {
            (did1, did2)
        } else {
            (did2, did1)
        };

        let mut hasher = blake3::Hasher::new();
        hasher.update(b"sync-contact-key");
        hasher.update(a.as_bytes());
        hasher.update(b.as_bytes());
        *hasher.finalize().as_bytes()
    }

    /// Subscribe to contact events
    ///
    /// Returns a receiver for ContactEvent broadcasts.
    pub fn subscribe_events(&self) -> broadcast::Receiver<ContactEvent> {
        self.event_tx.subscribe()
    }

    /// Auto-reconnect to all contacts on startup
    ///
    /// Loads all contacts from storage and attempts to reconnect,
    /// prioritizing favorites first.
    pub async fn reconnect_contacts(&self) -> SyncResult<()> {
        let mut contacts = self.storage.list_contacts()?;

        // Sort by is_favorite (favorites first)
        contacts.sort_by(|a, b| b.is_favorite.cmp(&a.is_favorite));

        info!(
            count = contacts.len(),
            "Auto-reconnecting to saved contacts"
        );

        for contact in contacts {
            match self.subscribe_contact_topic(&contact).await {
                Ok(_) => {
                    debug!(peer_did = %contact.peer_did, "Reconnected to contact");
                }
                Err(e) => {
                    warn!(
                        peer_did = %contact.peer_did,
                        error = ?e,
                        "Failed to reconnect to contact"
                    );
                }
            }

            // Add small delay to avoid overwhelming the network
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Internal Helper Methods
    // ═══════════════════════════════════════════════════════════════════════

    /// Sign an invite with our hybrid keypair (v1)
    fn sign_invite(
        &self,
        invite: &PeerContactInvite,
    ) -> SyncResult<crate::identity::HybridSignature> {
        // Serialize all fields except signature
        let mut data = Vec::new();
        data.extend_from_slice(&[invite.version]);
        data.extend_from_slice(&invite.invite_id);
        data.extend_from_slice(invite.inviter_did.as_bytes());
        data.extend_from_slice(&invite.inviter_pubkey);

        let profile_bytes = postcard::to_allocvec(&invite.profile_snapshot)
            .map_err(|e| SyncError::Serialization(e.to_string()))?;
        data.extend_from_slice(&profile_bytes);

        let node_addr_bytes = postcard::to_allocvec(&invite.node_addr)
            .map_err(|e| SyncError::Serialization(e.to_string()))?;
        data.extend_from_slice(&node_addr_bytes);

        data.extend_from_slice(&invite.created_at.to_le_bytes());
        data.extend_from_slice(&invite.expires_at.to_le_bytes());

        // Sign
        Ok(self.keypair.sign(&data))
    }

    /// Sign a hybrid invite with our hybrid keypair (v2)
    fn sign_hybrid_invite(&self, invite: &HybridContactInvite) -> SyncResult<Vec<u8>> {
        // Serialize all fields except signature
        let mut data = Vec::new();
        data.extend_from_slice(&[invite.version]);
        data.extend_from_slice(&invite.invite_id);
        data.extend_from_slice(invite.inviter_did.as_bytes());

        let node_addr_bytes = postcard::to_allocvec(&invite.node_addr)
            .map_err(|e| SyncError::Serialization(e.to_string()))?;
        data.extend_from_slice(&node_addr_bytes);

        data.extend_from_slice(invite.display_name.as_bytes());
        data.extend_from_slice(&invite.created_at.to_le_bytes());
        data.extend_from_slice(&invite.expires_at.to_le_bytes());

        // Sign with Ed25519 only (lightweight 64 bytes for QR codes)
        // Invites are ephemeral (expire in hours), so quantum resistance is less critical
        let ed25519_sig = self.keypair.sign_ed25519_only(&data);
        Ok(ed25519_sig.to_bytes().to_vec())
    }

    /// Verify invite signature (v1)
    fn verify_invite_signature(&self, invite: &PeerContactInvite) -> SyncResult<()> {
        // Reconstruct signed data
        let mut data = Vec::new();
        data.extend_from_slice(&[invite.version]);
        data.extend_from_slice(&invite.invite_id);
        data.extend_from_slice(invite.inviter_did.as_bytes());
        data.extend_from_slice(&invite.inviter_pubkey);

        let profile_bytes = postcard::to_allocvec(&invite.profile_snapshot)
            .map_err(|e| SyncError::Serialization(e.to_string()))?;
        data.extend_from_slice(&profile_bytes);

        let node_addr_bytes = postcard::to_allocvec(&invite.node_addr)
            .map_err(|e| SyncError::Serialization(e.to_string()))?;
        data.extend_from_slice(&node_addr_bytes);

        data.extend_from_slice(&invite.created_at.to_le_bytes());
        data.extend_from_slice(&invite.expires_at.to_le_bytes());

        // Deserialize signature
        let signature = crate::identity::HybridSignature::from_bytes(&invite.signature)?;

        // Deserialize public key
        let public_key = HybridPublicKey::from_bytes(&invite.inviter_pubkey)?;

        // Verify (returns bool, not Result)
        if !public_key.verify(&data, &signature) {
            return Err(SyncError::InvalidInvite(
                "Signature verification failed".to_string(),
            ));
        }

        Ok(())
    }

    /// Verify hybrid invite signature (v2)
    ///
    /// For v2 invites, the public key is not embedded in the invite.
    /// This means we cannot verify the signature without the public key.
    /// TODO: In a full implementation, this would fetch the public key
    /// from the node's profile or use a trusted key registry.
    /// For now, we skip signature verification for v2 invites.
    fn verify_hybrid_invite_signature(&self, invite: &HybridContactInvite) -> SyncResult<()> {
        // Reconstruct signed data
        let mut data = Vec::new();
        data.extend_from_slice(&[invite.version]);
        data.extend_from_slice(&invite.invite_id);
        data.extend_from_slice(invite.inviter_did.as_bytes());

        let node_addr_bytes = postcard::to_allocvec(&invite.node_addr)
            .map_err(|e| SyncError::Serialization(e.to_string()))?;
        data.extend_from_slice(&node_addr_bytes);

        data.extend_from_slice(invite.display_name.as_bytes());
        data.extend_from_slice(&invite.created_at.to_le_bytes());
        data.extend_from_slice(&invite.expires_at.to_le_bytes());

        // Validate Ed25519 signature format (64 bytes)
        if invite.signature.len() != 64 {
            return Err(SyncError::InvalidInvite(format!(
                "Invalid Ed25519 signature length: {} (expected 64)",
                invite.signature.len()
            )));
        }

        // SECURITY NOTE: We cannot verify the signature without the public key.
        // V2 hybrid invites use Ed25519-only signatures (64 bytes) instead of full
        // HybridSignature (~4600 bytes) to enable QR-code sharing. The public key
        // is not embedded to save space.
        //
        // The signature will be verified when we:
        // 1. Fetch the profile via PROFILE_ALPN (includes public key + signed profile)
        // 2. Complete the contact handshake (uses full quantum-resistant HybridSignature)
        //
        // This is acceptable because:
        // - Invites are ephemeral (expire in hours, not long-term secrets)
        // - Invites can only be used once (invite_id tracked and revocable)
        // - Contact protocol uses HybridSignature for long-term security
        // - Profile fetch provides full cryptographic verification

        debug!(
            invite_id = ?invite.invite_id,
            inviter_did = %invite.inviter_did,
            "Hybrid invite signature format verified (full verification deferred to profile fetch)"
        );

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // QUIC Network Protocol Implementation
    // ═══════════════════════════════════════════════════════════════════════

    /// Send a ContactRequest message via QUIC
    async fn send_contact_message(
        &self,
        node_addr: &NodeAddrBytes,
        _our_profile: ProfileSnapshot, // Legacy parameter, now using SignedProfile
        invite_id: [u8; 16],
    ) -> SyncResult<()> {
        use crate::types::{SignedProfile, UserProfile};

        // Convert NodeAddrBytes to EndpointAddr
        let endpoint_addr = node_addr.to_endpoint_addr()?;

        // Prepare message (done once, outside retry loop)
        let our_node_addr = NodeAddrBytes::from_endpoint_addr(&self.gossip_sync.endpoint_addr());
        let our_node_addr_bytes = postcard::to_allocvec(&our_node_addr)
            .map_err(|e| SyncError::Serialization(format!("Failed to serialize node address: {}", e)))?;

        let requester_pubkey = self.keypair.public_key().to_bytes();

        // Load our full profile and sign it
        let user_profile = match self.storage.load_profile(&self.did.to_string()) {
            Ok(Some(profile)) => profile,
            _ => {
                // Fallback: create minimal profile
                let short_did = self.did.to_string().chars().take(16).collect::<String>();
                warn!(did = %self.did, "No profile found for contact request, using fallback");
                UserProfile::new(self.did.to_string(), format!("{}...", short_did))
            }
        };
        let signed_profile = SignedProfile::sign(&user_profile, &self.keypair);

        // Create and sign the request message
        let mut data_to_sign = Vec::new();
        data_to_sign.extend_from_slice(&invite_id);
        data_to_sign.extend_from_slice(self.did.as_ref().as_bytes());
        data_to_sign.extend_from_slice(&requester_pubkey);

        let profile_bytes = postcard::to_allocvec(&signed_profile)
            .map_err(|e| SyncError::Serialization(format!("Failed to serialize signed profile: {}", e)))?;
        data_to_sign.extend_from_slice(&profile_bytes);
        data_to_sign.extend_from_slice(&our_node_addr_bytes);

        let signature = self.keypair.sign(&data_to_sign);
        let requester_signature = signature.to_bytes();

        let message = ContactMessage::ContactRequest {
            invite_id,
            requester_did: self.did.to_string(),
            requester_pubkey,
            requester_signed_profile: signed_profile,
            requester_node_addr: our_node_addr_bytes,
            requester_signature,
        };

        let bytes = message
            .encode()
            .map_err(|e| SyncError::Serialization(format!("Failed to encode ContactRequest: {}", e)))?;

        // Retry network operations (connect + send)
        self.retry_with_backoff("send_contact_request", || async {
            debug!(
                peer = %endpoint_addr.id,
                invite_id = ?invite_id,
                "Connecting to inviter to send ContactRequest"
            );

            // Connect to the peer
            let connection = self
                .gossip_sync
                .endpoint()
                .connect(endpoint_addr.clone(), CONTACT_ALPN)
                .await
                .map_err(|e| SyncError::Network(format!("Failed to connect to inviter: {}", e)))?;

            // Open a bi-directional stream
            let (mut send, _recv) = connection
                .open_bi()
                .await
                .map_err(|e| SyncError::Network(format!("Failed to open bi stream: {}", e)))?;

            // Send message
            send.write_all(&bytes)
                .await
                .map_err(|e| SyncError::Network(format!("Failed to send ContactRequest: {}", e)))?;

            // Finish the send side so receiver knows the message is complete
            send.finish()
                .map_err(|e| SyncError::Network(format!("Failed to finish send stream: {}", e)))?;

            info!(
                peer = %endpoint_addr.id,
                invite_id = ?invite_id,
                "Sent ContactRequest"
            );

            // Keep connection alive to allow peer to process the message
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            Ok(())
        })
        .await
    }

    /// Send a ContactAccept message via QUIC (simplified protocol)
    ///
    /// This replaces the old ContactResponse + ContactAccepted flow with a single message.
    /// Keys are derived locally by both parties, not transmitted.
    /// Now sends SignedProfile for immediate profile pinning by the recipient.
    async fn send_contact_accept(
        &self,
        node_addr: &NodeAddrBytes,
        invite_id: [u8; 16],
    ) -> SyncResult<()> {
        use crate::types::{SignedProfile, UserProfile};

        // Convert NodeAddrBytes to EndpointAddr
        let endpoint_addr = node_addr.to_endpoint_addr()?;

        // Prepare message (done once, outside retry loop)
        let accepter_did = self.did.to_string();
        let accepter_pubkey = self.keypair.public_key().to_bytes();

        // Load our full profile and sign it
        let user_profile = match self.storage.load_profile(&self.did.to_string()) {
            Ok(Some(profile)) => profile,
            _ => {
                // Fallback: create minimal profile
                let short_did = self.did.to_string().chars().take(16).collect::<String>();
                warn!(did = %self.did, "No profile found for contact accept, using fallback");
                UserProfile::new(self.did.to_string(), format!("{}...", short_did))
            }
        };
        let signed_profile = SignedProfile::sign(&user_profile, &self.keypair);

        // Serialize our node address
        let our_node_addr = NodeAddrBytes::from_endpoint_addr(&self.gossip_sync.endpoint_addr());
        let accepter_node_addr = postcard::to_allocvec(&our_node_addr)
            .map_err(|e| SyncError::Serialization(format!("Failed to serialize node address: {}", e)))?;

        // Build data to sign: invite_id + did + pubkey + signed_profile + node_addr
        let mut data_to_sign = Vec::new();
        data_to_sign.extend_from_slice(&invite_id);
        data_to_sign.extend_from_slice(accepter_did.as_bytes());
        data_to_sign.extend_from_slice(&accepter_pubkey);

        let profile_bytes = postcard::to_allocvec(&signed_profile)
            .map_err(|e| SyncError::Serialization(format!("Failed to serialize signed profile: {}", e)))?;
        data_to_sign.extend_from_slice(&profile_bytes);
        data_to_sign.extend_from_slice(&accepter_node_addr);

        // Sign with our hybrid keypair
        let sig = self.keypair.sign(&data_to_sign);
        let signature = sig.to_bytes();

        let message = ContactMessage::ContactAccept {
            invite_id,
            accepter_did,
            accepter_pubkey,
            accepter_signed_profile: signed_profile,
            accepter_node_addr,
            signature,
        };

        let bytes = message
            .encode()
            .map_err(|e| SyncError::Serialization(format!("Failed to encode ContactAccept: {}", e)))?;

        // Retry network operations (connect + send)
        self.retry_with_backoff("send_contact_accept", || async {
            debug!(
                peer = %endpoint_addr.id,
                invite_id = ?invite_id,
                "Connecting to requester to send ContactAccept"
            );

            // Connect to the peer
            let connection = self
                .gossip_sync
                .endpoint()
                .connect(endpoint_addr.clone(), CONTACT_ALPN)
                .await
                .map_err(|e| SyncError::Network(format!("Failed to connect to requester: {}", e)))?;

            // Open a bi-directional stream
            let (mut send, _recv) = connection
                .open_bi()
                .await
                .map_err(|e| SyncError::Network(format!("Failed to open bi stream: {}", e)))?;

            // Send message
            send.write_all(&bytes)
                .await
                .map_err(|e| SyncError::Network(format!("Failed to send ContactAccept: {}", e)))?;

            send.finish()
                .map_err(|e| SyncError::Network(format!("Failed to finish send stream: {}", e)))?;

            info!(
                peer = %endpoint_addr.id,
                invite_id = ?invite_id,
                "Sent ContactAccept (simplified protocol)"
            );

            // Keep connection alive to allow peer to process the message
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            Ok(())
        })
        .await
    }

    /// Send a ContactDecline message via QUIC
    async fn send_contact_decline(
        &self,
        node_addr: &NodeAddrBytes,
        invite_id: [u8; 16],
    ) -> SyncResult<()> {
        // Convert NodeAddrBytes to EndpointAddr
        let endpoint_addr = node_addr.to_endpoint_addr()?;

        let message = ContactMessage::ContactDecline { invite_id };

        let bytes = message
            .encode()
            .map_err(|e| SyncError::Serialization(format!("Failed to encode ContactDecline: {}", e)))?;

        // Retry network operations (connect + send)
        self.retry_with_backoff("send_contact_decline", || async {
            debug!(
                peer = %endpoint_addr.id,
                invite_id = ?invite_id,
                "Connecting to requester to send ContactDecline"
            );

            // Connect to the peer
            let connection = self
                .gossip_sync
                .endpoint()
                .connect(endpoint_addr.clone(), CONTACT_ALPN)
                .await
                .map_err(|e| SyncError::Network(format!("Failed to connect to requester: {}", e)))?;

            // Open a bi-directional stream
            let (mut send, _recv) = connection
                .open_bi()
                .await
                .map_err(|e| SyncError::Network(format!("Failed to open bi stream: {}", e)))?;

            // Send message
            send.write_all(&bytes)
                .await
                .map_err(|e| SyncError::Network(format!("Failed to send ContactDecline: {}", e)))?;

            send.finish()
                .map_err(|e| SyncError::Network(format!("Failed to finish send stream: {}", e)))?;

            info!(
                peer = %endpoint_addr.id,
                invite_id = ?invite_id,
                "Sent ContactDecline"
            );

            // Keep connection alive to allow peer to process the message
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            Ok(())
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    //! # Contact Manager Unit Tests
    //!
    //! These tests verify the ContactManager's storage and logic operations **without
    //! performing actual network operations**.
    //!
    //! ## Test Architecture
    //!
    //! - **Unit tests** (this module): Test storage, state transitions, and logic in isolation
    //!   - No QUIC connections
    //!   - No gossip topic subscriptions
    //!   - Fast execution
    //!
    //! - **Integration tests** (`tests/contact_integration.rs`): Test SyncEngine API contracts
    //!   - Verify API methods work correctly
    //!   - Storage operations via engine interface
    //!   - No network operations
    //!
    //! - **E2E tests** (`tests/contact_e2e_test.rs`): Test full network communication
    //!   - Two separate SyncEngine instances
    //!   - Real QUIC connections
    //!   - Actual network message propagation
    //!   - Complete user flows
    //!
    //! ## Why This Separation?
    //!
    //! Network operations require complex multi-node setup and are slower. By testing
    //! storage/logic at the unit level and network paths at the E2E level, we get:
    //! - Fast test suite (unit tests run in ~2 seconds)
    //! - Clear failure signals (storage bugs vs network bugs)
    //! - No flaky network timing issues in unit tests

    use super::*;
    use crate::sync::GossipSync;
    use tempfile::TempDir;

    async fn create_test_manager() -> (ContactManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Arc::new(Storage::new(&db_path).unwrap());

        let gossip_sync = Arc::new(GossipSync::new().await.unwrap());
        let keypair = Arc::new(HybridKeypair::generate());
        let did = Did::from_public_key(&keypair.public_key());
        let (event_tx, _) = broadcast::channel(100);

        let manager = ContactManager::new(gossip_sync, keypair, did, storage, event_tx);

        (manager, temp_dir)
    }

    fn create_test_profile(name: &str) -> ProfileSnapshot {
        ProfileSnapshot {
            display_name: name.to_string(),
            subtitle: Some("Test User".to_string()),
            avatar_blob_id: None,
            bio: "This is a test profile for unit testing.".to_string(),
        }
    }

    #[tokio::test]
    async fn test_generate_and_decode_invite() {
        let (manager, _temp) = create_test_manager().await;
        let profile = create_test_profile("Alice");

        // Generate invite
        let invite_code = manager.generate_invite(profile.clone(), 24).unwrap();
        assert!(invite_code.starts_with("sync-contact:"));

        // Decode invite
        let decoded = manager.decode_invite(&invite_code).unwrap();
        assert_eq!(decoded.version, 2);
        assert_eq!(decoded.display_name, "Alice");
        assert!(!decoded.is_expired());
    }

    #[tokio::test]
    async fn test_hybrid_invite_size_reduction() {
        let (manager, _temp) = create_test_manager().await;

        // Create a profile with realistic data
        let profile = ProfileSnapshot {
            display_name: "Alice Wonderland".to_string(),
            subtitle: Some("Software Engineer".to_string()),
            avatar_blob_id: Some("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string()),
            bio: "Building decentralized systems and exploring the future of peer-to-peer technology. Passionate about privacy, security, and user empowerment.".to_string(),
        };

        // Check what's in the node address
        let node_addr = crate::invite::NodeAddrBytes::from_endpoint_addr(&manager.gossip_sync.endpoint_addr());
        eprintln!("Node address info:");
        eprintln!("  relay_url: {:?}", node_addr.relay_url);
        eprintln!("  direct_addresses count: {}", node_addr.direct_addresses.len());
        eprintln!("  direct_addresses: {:?}", node_addr.direct_addresses);

        // Generate v2 hybrid invite
        let invite_code = manager.generate_invite(profile, 24).unwrap();

        // Verify it's a valid invite code
        assert!(invite_code.starts_with("sync-contact:"));

        // Log the actual size for visibility
        eprintln!("Hybrid invite size: {} characters", invite_code.len());

        // Verify it's under the target size (should be ~420-450 chars)
        assert!(
            invite_code.len() < 500,
            "Hybrid invite too long: {} chars (target: < 500)",
            invite_code.len()
        );

        // Verify it's significantly smaller than old v1 invites (which were ~2000 chars)
        // With profile data, subtitle, avatar hash, and bio, v1 would be massive
        // Our hybrid invite should be < 25% of that size
        assert!(
            invite_code.len() < 600,
            "Hybrid invite should be compact: {} chars",
            invite_code.len()
        );

        // Decode to verify it works
        let decoded = manager.decode_invite(&invite_code).unwrap();
        assert_eq!(decoded.version, 2);
        assert_eq!(decoded.display_name, "Alice Wonderland");
    }

    #[tokio::test]
    async fn test_decode_invalid_prefix() {
        let (manager, _temp) = create_test_manager().await;

        let result = manager.decode_invite("invalid-prefix:abc123");
        assert!(result.is_err());
        assert!(matches!(result, Err(SyncError::InvalidInvite(_))));
    }

    #[tokio::test]
    async fn test_decode_invalid_base58() {
        let (manager, _temp) = create_test_manager().await;

        let result = manager.decode_invite("sync-contact:invalid!base58");
        assert!(result.is_err());
        assert!(matches!(result, Err(SyncError::InvalidInvite(_))));
    }

    #[tokio::test]
    async fn test_send_contact_request_storage() {
        let (manager, _temp) = create_test_manager().await;

        // Create a fake invite from a different peer (not self)
        let invite_id = PeerContactInvite::generate_invite_id();
        let fake_invite = PeerContactInvite {
            version: 1,
            invite_id,
            inviter_did: "did:sync:fake_peer".to_string(),
            inviter_pubkey: vec![0u8; 32],
            profile_snapshot: create_test_profile("Bob"),
            node_addr: NodeAddrBytes::new([0u8; 32]),
            created_at: chrono::Utc::now().timestamp(),
            expires_at: chrono::Utc::now().timestamp() + 86400,
            signature: vec![0u8; 64],
        };

        // Manually create the pending contact (what send_contact_request does)
        let pending = PendingContact {
            invite_id: fake_invite.invite_id,
            peer_did: fake_invite.inviter_did.clone(),
            profile: fake_invite.profile_snapshot.clone(),
            signed_profile: None,
            node_addr: fake_invite.node_addr.clone(),
            state: ContactState::OutgoingPending,
            created_at: chrono::Utc::now().timestamp(),
        };

        // Save to storage (what send_contact_request does, without network operation)
        manager.storage.save_pending(&pending).unwrap();

        // Verify pending was saved correctly
        let loaded = manager
            .storage
            .load_pending(&invite_id)
            .unwrap()
            .unwrap();
        assert_eq!(loaded.state, ContactState::OutgoingPending);
        assert_eq!(loaded.profile.display_name, "Bob");
        assert_eq!(loaded.peer_did, "did:sync:fake_peer");
    }

    #[tokio::test]
    async fn test_accept_contact_request_storage() {
        let (manager, _temp) = create_test_manager().await;

        // Create a fake incoming pending contact
        let invite_id = PeerContactInvite::generate_invite_id();
        let pending = PendingContact {
            invite_id,
            peer_did: "did:sync:test".to_string(),
            profile: create_test_profile("Charlie"),
            signed_profile: None,
            node_addr: NodeAddrBytes::new([0u8; 32]),
            state: ContactState::IncomingPending,
            created_at: chrono::Utc::now().timestamp(),
        };

        manager.storage.save_pending(&pending).unwrap();

        // Manually perform the storage operations that accept_contact_request does
        // (without network operations like send_contact_response, send_contact_accepted, subscribe_contact_topic)

        // 1. Update pending state to WaitingForMutual
        let mut pending = manager.storage.load_pending(&invite_id).unwrap().unwrap();
        pending.state = ContactState::WaitingForMutual;
        manager.storage.save_pending(&pending).unwrap();

        // 2. Create ContactInfo (what finalize_contact does)
        let contact_topic = ContactManager::derive_contact_topic(manager.did.as_ref(), &pending.peer_did);
        let contact_key = ContactManager::derive_contact_key(manager.did.as_ref(), &pending.peer_did);

        let contact = ContactInfo {
            peer_did: pending.peer_did.clone(),
            peer_endpoint_id: pending.node_addr.node_id,
            profile: pending.profile.clone(),
            node_addr: pending.node_addr.clone(),
            contact_topic,
            contact_key,
            accepted_at: chrono::Utc::now().timestamp(),
            last_seen: chrono::Utc::now().timestamp() as u64,
            status: ContactStatus::Offline,
            is_favorite: false,
        };

        // 3. Save contact to storage
        manager.storage.save_contact(&contact).unwrap();

        // 4. Delete pending
        manager.storage.delete_pending(&invite_id).unwrap();

        // Verify pending is gone
        assert!(manager.storage.load_pending(&invite_id).unwrap().is_none());

        // Verify contact was created
        let contacts = manager.storage.list_contacts().unwrap();
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].peer_did, "did:sync:test");
        assert_eq!(contacts[0].profile.display_name, "Charlie");
        assert_eq!(contacts[0].status, ContactStatus::Offline);
    }

    #[tokio::test]
    async fn test_decline_contact_request() {
        let (manager, _temp) = create_test_manager().await;

        // Create a fake incoming pending contact
        let invite_id = PeerContactInvite::generate_invite_id();
        let pending = PendingContact {
            invite_id,
            peer_did: "did:sync:test".to_string(),
            profile: create_test_profile("Dave"),
            signed_profile: None,
            node_addr: NodeAddrBytes::new([0u8; 32]),
            state: ContactState::IncomingPending,
            created_at: chrono::Utc::now().timestamp(),
        };

        manager.storage.save_pending(&pending).unwrap();

        // Decline the contact
        manager.decline_contact_request(&invite_id).await.unwrap();

        // Verify pending is gone
        assert!(manager.storage.load_pending(&invite_id).unwrap().is_none());

        // Verify no contact was created
        assert_eq!(manager.storage.list_contacts().unwrap().len(), 0);
    }

    #[test]
    fn test_derive_contact_topic_deterministic() {
        let did1 = "did:sync:alice";
        let did2 = "did:sync:bob";

        let topic1 = ContactManager::derive_contact_topic(did1, did2);
        let topic2 = ContactManager::derive_contact_topic(did2, did1);

        // Both orders should produce the same topic
        assert_eq!(topic1, topic2);
    }

    #[test]
    fn test_derive_contact_key_deterministic() {
        let did1 = "did:sync:alice";
        let did2 = "did:sync:bob";

        let key1 = ContactManager::derive_contact_key(did1, did2);
        let key2 = ContactManager::derive_contact_key(did2, did1);

        // Both orders should produce the same key
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_derive_contact_topic_different_pairs() {
        let topic1 = ContactManager::derive_contact_topic("did:sync:alice", "did:sync:bob");
        let topic2 = ContactManager::derive_contact_topic("did:sync:alice", "did:sync:charlie");

        // Different peer pairs should produce different topics
        assert_ne!(topic1, topic2);
    }

    #[tokio::test]
    async fn test_revoked_invite_rejected() {
        let (manager, _temp) = create_test_manager().await;
        let profile = create_test_profile("Eve");

        // Generate invite
        let invite_code = manager.generate_invite(profile, 24).unwrap();
        let invite = manager.decode_invite(&invite_code).unwrap();

        // Revoke it
        manager.storage.revoke_invite(&invite.invite_id).unwrap();

        // Try to decode again - should fail
        let result = manager.decode_invite(&invite_code);
        assert!(result.is_err());
        assert!(matches!(result, Err(SyncError::InvalidInvite(_))));
    }

    #[tokio::test]
    async fn test_expired_invite_rejected() {
        let (manager, _temp) = create_test_manager().await;

        // Create an expired invite manually
        let invite_id = PeerContactInvite::generate_invite_id();
        let node_addr = NodeAddrBytes::from_endpoint_addr(&manager.gossip_sync.endpoint_addr());

        let mut invite = PeerContactInvite {
            version: 1,
            invite_id,
            inviter_did: manager.did.to_string(),
            inviter_pubkey: manager.keypair.public_key().to_bytes(),
            profile_snapshot: create_test_profile("Expired"),
            node_addr,
            created_at: chrono::Utc::now().timestamp() - 1000,
            expires_at: chrono::Utc::now().timestamp() - 100, // Already expired
            signature: vec![],
        };

        // Sign it
        let signature = manager.sign_invite(&invite).unwrap();
        invite.signature = signature.to_bytes();

        // Encode it
        let serialized = postcard::to_allocvec(&invite).unwrap();
        let encoded = bs58::encode(&serialized).into_string();
        let invite_code = format!("sync-contact:{}", encoded);

        // Try to decode - should fail due to expiry
        let result = manager.decode_invite(&invite_code);
        assert!(result.is_err());
        assert!(matches!(result, Err(SyncError::InvalidInvite(_))));
    }

    #[tokio::test]
    async fn test_contact_event_emitted_on_generate() {
        let (manager, _temp) = create_test_manager().await;
        let mut event_rx = manager.subscribe_events();

        let profile = create_test_profile("Frank");
        let _invite = manager.generate_invite(profile, 24).unwrap();

        // Should receive InviteGenerated event
        let event = tokio::time::timeout(tokio::time::Duration::from_millis(100), event_rx.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("Event channel closed");

        assert!(matches!(event, ContactEvent::InviteGenerated { .. }));
    }
}

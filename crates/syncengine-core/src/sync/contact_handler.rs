//! Contact protocol handler for Router integration
//!
//! This provides a protocol handler that can be registered with the iroh Router
//! to handle CONTACT_ALPN connections. It processes contact protocol messages
//! and updates storage directly.

use std::sync::Arc;

use iroh::discovery::static_provider::StaticProvider;
use iroh::endpoint::Connection;
use iroh::protocol::ProtocolHandler;
use iroh_gossip::net::Gossip;
use iroh_gossip::TopicId;
use tokio::sync::broadcast;
use tracing::{debug, error, info};

use crate::error::SyncError;
use crate::invite::NodeAddrBytes;
use crate::storage::Storage;
use crate::sync::contact_protocol::{ContactMessage, CONTACT_ALPN};
use crate::sync::{ActiveContactTopics, ContactEvent};
use crate::types::contact::{ContactState, PendingContact, ProfileSnapshot};
use crate::types::peer::{ContactDetails, Peer, PeerSource, PeerStatus};

/// Protocol handler for contact exchange
///
/// This is registered with the Router to handle incoming CONTACT_ALPN connections.
/// It processes contact protocol messages directly, keeping the connection alive
/// throughout the message exchange.
#[derive(Clone)]
pub struct ContactProtocolHandler {
    storage: Arc<Storage>,
    event_tx: broadcast::Sender<ContactEvent>,
    gossip: Gossip,
    /// Static discovery for adding peer addresses before gossip subscription
    static_discovery: StaticProvider,
    /// Our local DID for key derivation in the simplified protocol
    local_did: String,
    /// Shared active topics map for contact topic senders.
    /// When we receive a ContactAccept, we subscribe to the contact topic and
    /// add the sender here so ContactManager can use it for sending messages.
    active_topics: ActiveContactTopics,
}

impl std::fmt::Debug for ContactProtocolHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContactProtocolHandler")
            .field("storage", &"<Storage>")
            .field("event_tx", &"<Sender<ContactEvent>>")
            .field("gossip", &"<Gossip>")
            .field("static_discovery", &"<StaticProvider>")
            .field("local_did", &self.local_did)
            .field("active_topics", &"<ActiveContactTopics>")
            .finish()
    }
}

impl ContactProtocolHandler {
    /// Create a new contact protocol handler
    ///
    /// The `local_did` is required for the simplified protocol to derive
    /// contact keys locally without transmitting them.
    /// The `static_discovery` is used to add peer addresses before gossip subscription.
    /// The `active_topics` is the shared map where we add senders when receiving ContactAccept,
    /// so ContactManager can use them for sending messages to contacts.
    pub fn new(
        storage: Arc<Storage>,
        event_tx: broadcast::Sender<ContactEvent>,
        gossip: Gossip,
        static_discovery: StaticProvider,
        local_did: String,
        active_topics: ActiveContactTopics,
    ) -> Self {
        Self {
            storage,
            event_tx,
            gossip,
            static_discovery,
            local_did,
            active_topics,
        }
    }

    /// Get the ALPN identifier for this protocol
    pub const fn alpn() -> &'static [u8] {
        CONTACT_ALPN
    }

    /// Handle a routed contact connection
    ///
    /// This processes the contact protocol message and updates storage.
    async fn handle_connection(
        connection: Connection,
        storage: Arc<Storage>,
        event_tx: broadcast::Sender<ContactEvent>,
        gossip: Gossip,
        static_discovery: StaticProvider,
        local_did: String,
        active_topics: ActiveContactTopics,
    ) -> Result<(), SyncError> {
        let remote_id = connection.remote_id();
        debug!(?remote_id, "Handling routed contact connection");

        // Accept a bi-directional stream
        let (_send, mut recv) = connection
            .accept_bi()
            .await
            .map_err(|e| SyncError::Network(format!("Failed to accept bi stream: {}", e)))?;

        // Read the message bytes
        let bytes = recv
            .read_to_end(1024 * 1024) // 1MB max
            .await
            .map_err(|e| SyncError::Network(format!("Failed to read message: {}", e)))?;

        // Decode the message
        let message = ContactMessage::decode(&bytes).map_err(|e| {
            SyncError::Serialization(format!("Failed to decode contact message: {}", e))
        })?;

        debug!(?message, "Received contact message");

        // Handle the message based on variant
        match message {
            ContactMessage::ContactRequest {
                invite_id,
                requester_did,
                requester_pubkey,
                requester_signed_profile,
                requester_node_addr,
                requester_encryption_keys,
                requester_signature,
            } => {
                // Verify signature
                use crate::identity::{Did, HybridPublicKey, HybridSignature};

                // First, verify the SignedProfile's own signature (profile authenticity)
                if !requester_signed_profile.verify() {
                    return Err(SyncError::Identity(
                        "Requester's SignedProfile signature verification failed".to_string(),
                    ));
                }

                // Deserialize public key
                let pubkey = HybridPublicKey::from_bytes(&requester_pubkey)
                    .map_err(|e| SyncError::Identity(format!("Invalid public key: {}", e)))?;

                // Verify DID matches public key
                let expected_did = Did::from_public_key(&pubkey);
                let requester_did_parsed = Did::parse(&requester_did)
                    .map_err(|e| SyncError::Identity(format!("Invalid DID: {}", e)))?;

                if expected_did != requester_did_parsed {
                    return Err(SyncError::Identity(format!(
                        "DID mismatch: expected {} but got {}",
                        expected_did, requester_did
                    )));
                }

                // Rebuild signed data to verify message signature
                let mut data_to_verify = Vec::new();
                data_to_verify.extend_from_slice(&invite_id);
                data_to_verify.extend_from_slice(requester_did.as_bytes());
                data_to_verify.extend_from_slice(&requester_pubkey);

                let profile_bytes = postcard::to_allocvec(&requester_signed_profile)
                    .map_err(|e| SyncError::Serialization(format!("Failed to serialize signed profile: {}", e)))?;
                data_to_verify.extend_from_slice(&profile_bytes);
                data_to_verify.extend_from_slice(&requester_node_addr);
                // Include encryption keys in signature verification if present
                if let Some(ref enc_keys) = requester_encryption_keys {
                    data_to_verify.extend_from_slice(enc_keys);
                }

                // Verify message signature
                let signature = HybridSignature::from_bytes(&requester_signature)
                    .map_err(|e| SyncError::Identity(format!("Invalid signature: {}", e)))?;

                if !pubkey.verify(&data_to_verify, &signature) {
                    return Err(SyncError::Identity("Message signature verification failed".to_string()));
                }

                debug!(
                    invite_id = ?invite_id,
                    requester_did = %requester_did,
                    "Signatures verified successfully (profile + message)"
                );

                // Deserialize node address
                let node_addr: NodeAddrBytes = postcard::from_bytes(&requester_node_addr)
                    .map_err(|e| {
                        SyncError::Serialization(format!("Invalid node address: {}", e))
                    })?;

                // Check if this is an invite we generated (should auto-accept)
                let is_our_invite = storage.is_our_generated_invite(&invite_id).unwrap_or(false);

                if is_our_invite {
                    // Clean up the generated invite record since it's being used
                    let _ = storage.delete_generated_invite(&invite_id);
                    info!(
                        invite_id = ?invite_id,
                        requester_did = %requester_did,
                        "Received request for our own invite - will auto-accept"
                    );
                }

                // Extract ProfileSnapshot from SignedProfile for PendingContact
                let profile_snapshot = ProfileSnapshot {
                    display_name: requester_signed_profile.profile.display_name.clone(),
                    subtitle: requester_signed_profile.profile.subtitle.clone(),
                    avatar_blob_id: requester_signed_profile.profile.avatar_blob_id.clone(),
                    bio: ProfileSnapshot::truncate_bio(&requester_signed_profile.profile.bio),
                };

                // Save as IncomingPending (with SignedProfile for later pinning)
                let pending = PendingContact {
                    invite_id,
                    peer_did: requester_did.clone(),
                    profile: profile_snapshot.clone(),
                    signed_profile: Some(requester_signed_profile),
                    node_addr,
                    state: ContactState::IncomingPending,
                    created_at: chrono::Utc::now().timestamp(),
                    encryption_keys: requester_encryption_keys,
                };

                storage.save_pending(&pending)?;

                info!(
                    invite_id = ?invite_id,
                    requester_did = %requester_did,
                    auto_accept = is_our_invite,
                    "Received contact request, saved as IncomingPending"
                );

                // Emit event (with auto_accept flag if it's our invite)
                let _ = event_tx.send(ContactEvent::ContactRequestReceived {
                    invite_id,
                    from: profile_snapshot,
                    auto_accept: is_our_invite,
                });
            }

            ContactMessage::ContactAccept {
                invite_id,
                accepter_did,
                accepter_pubkey,
                accepter_signed_profile,
                accepter_node_addr,
                accepter_encryption_keys,
                signature,
            } => {
                // Verify signature
                use crate::identity::{Did, HybridPublicKey, HybridSignature};
                use crate::sync::contact_protocol::{derive_contact_key, derive_contact_topic};
                use crate::sync::derive_profile_topic;
                use crate::types::{PinRelationship, ProfilePin};

                // First, verify the SignedProfile's own signature (profile authenticity)
                if !accepter_signed_profile.verify() {
                    return Err(SyncError::Identity(
                        "Accepter's SignedProfile signature verification failed".to_string(),
                    ));
                }

                // Deserialize public key
                let pubkey = HybridPublicKey::from_bytes(&accepter_pubkey)
                    .map_err(|e| SyncError::Identity(format!("Invalid public key: {}", e)))?;

                // Verify DID matches public key
                let expected_did = Did::from_public_key(&pubkey);
                let accepter_did_parsed = Did::parse(&accepter_did)
                    .map_err(|e| SyncError::Identity(format!("Invalid DID: {}", e)))?;

                if expected_did != accepter_did_parsed {
                    return Err(SyncError::Identity(format!(
                        "DID mismatch in ContactAccept: expected {} but got {}",
                        expected_did, accepter_did
                    )));
                }

                // Rebuild signed data to verify message signature
                let mut data_to_verify = Vec::new();
                data_to_verify.extend_from_slice(&invite_id);
                data_to_verify.extend_from_slice(accepter_did.as_bytes());
                data_to_verify.extend_from_slice(&accepter_pubkey);

                let profile_bytes = postcard::to_allocvec(&accepter_signed_profile)
                    .map_err(|e| SyncError::Serialization(format!("Failed to serialize signed profile: {}", e)))?;
                data_to_verify.extend_from_slice(&profile_bytes);
                data_to_verify.extend_from_slice(&accepter_node_addr);
                // Include encryption keys in signature verification if present
                if let Some(ref enc_keys) = accepter_encryption_keys {
                    data_to_verify.extend_from_slice(enc_keys);
                }

                // Verify message signature
                let sig = HybridSignature::from_bytes(&signature)
                    .map_err(|e| SyncError::Identity(format!("Invalid signature: {}", e)))?;

                if !pubkey.verify(&data_to_verify, &sig) {
                    return Err(SyncError::Identity("ContactAccept message signature verification failed".to_string()));
                }

                debug!(
                    invite_id = ?invite_id,
                    accepter_did = %accepter_did,
                    "ContactAccept signatures verified (profile + message), finalizing contact"
                );

                // Load and verify pending is in OutgoingPending state
                if let Ok(Some(pending)) = storage.load_pending(&invite_id) {
                    if pending.state == ContactState::OutgoingPending {
                        // Deserialize accepter's node address
                        let node_addr: NodeAddrBytes = postcard::from_bytes(&accepter_node_addr)
                            .map_err(|e| {
                                SyncError::Serialization(format!("Invalid node address: {}", e))
                            })?;

                        // Derive keys locally from DIDs (no transmission needed!)
                        // Use local_did (our DID) and accepter_did (peer's DID)
                        let contact_topic = derive_contact_topic(&local_did, &accepter_did);
                        let contact_key = derive_contact_key(&local_did, &accepter_did);

                        // Extract ProfileSnapshot from SignedProfile for ContactInfo
                        let profile_snapshot = ProfileSnapshot {
                            display_name: accepter_signed_profile.profile.display_name.clone(),
                            subtitle: accepter_signed_profile.profile.subtitle.clone(),
                            avatar_blob_id: accepter_signed_profile.profile.avatar_blob_id.clone(),
                            bio: accepter_signed_profile.profile.bio.clone(),
                        };

                        // Create ContactInfo and save to contacts table
                        use crate::types::contact::{ContactInfo, ContactStatus};

                        let contact = ContactInfo {
                            peer_did: accepter_did.clone(),
                            peer_endpoint_id: node_addr.node_id,
                            profile: profile_snapshot.clone(),
                            node_addr: node_addr.clone(),
                            contact_topic,
                            contact_key,
                            accepted_at: chrono::Utc::now().timestamp(),
                            last_seen: chrono::Utc::now().timestamp() as u64,
                            status: ContactStatus::Online, // Online since we just received their message
                            is_favorite: false,
                            encryption_keys: accepter_encryption_keys.clone(),
                        };

                        // Save to legacy contacts table
                        storage.save_contact(&contact)?;

                        // Also save to unified peers table (new system)
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
                        storage.save_peer(&unified_peer)?;

                        // Pin the accepter's profile immediately (per-peer profile topics)
                        let pin = ProfilePin::new(
                            accepter_did.clone(),
                            accepter_signed_profile.clone(),
                            PinRelationship::Contact,
                        );
                        if let Err(e) = storage.save_pinned_profile(&pin) {
                            error!(did = %accepter_did, error = %e, "Failed to pin contact's profile");
                        } else {
                            info!(did = %accepter_did, "Pinned contact's profile from ContactAccept");
                        }

                        storage.delete_pending(&invite_id)?;

                        info!(
                            invite_id = ?invite_id,
                            peer_did = %contact.peer_did,
                            "Contact accepted and finalized with simplified protocol (keys derived locally)"
                        );

                        // Subscribe to the contact gossip topic WITH the peer as bootstrap
                        // CRITICAL: Add peer to static discovery FIRST so gossip can connect!
                        // Without this, the bootstrap peer lookup fails and subscription closes.
                        let contact_topic_id = TopicId::from_bytes(contact.contact_topic);
                        let bootstrap_peer = iroh::PublicKey::from_bytes(&node_addr.node_id)
                            .expect("node_id should be valid PublicKey bytes");

                        // Add peer address to static discovery for reachability
                        if let Ok(endpoint_addr) = node_addr.to_endpoint_addr() {
                            static_discovery.add_endpoint_info(endpoint_addr);
                            info!(
                                peer_did = %contact.peer_did,
                                "Added peer to static discovery before contact topic subscription"
                            );
                        }

                        match gossip.subscribe(contact_topic_id, vec![bootstrap_peer]).await {
                            Ok(topic) => {
                                info!(
                                    peer_did = %contact.peer_did,
                                    ?contact_topic_id,
                                    "Subscribed to contact gossip topic"
                                );

                                // Split the topic to get sender for active_topics
                                // CRITICAL: Add sender to active_topics so ContactManager can use it for SENDING.
                                // The receiver is dropped here - ContactManager will create its own subscription
                                // when it receives the ContactAccepted event, which will handle RECEIVING.
                                //
                                // This separation is important because iroh-gossip allows multiple subscriptions
                                // to the same topic, each getting its own receiver. We need the sender here
                                // for immediate sending capability, while ContactManager's listener handles
                                // all message processing (Announce AND Packet messages).
                                let (sender, receiver) = topic.split();

                                // Wrap sender in TopicSender and add to active_topics
                                let topic_sender = crate::sync::TopicSender::from_raw(sender, contact_topic_id);
                                {
                                    let mut topics = active_topics.write().await;
                                    topics.insert(contact.contact_topic, topic_sender);
                                    info!(
                                        peer_did = %contact.peer_did,
                                        topic_bytes = ?contact.contact_topic,
                                        "Added contact topic sender to active_topics (handler)"
                                    );
                                }

                                // Drop receiver - ContactManager will create its own subscription
                                // when it receives the ContactAccepted event via start_contact_subscription_task().
                                // This prevents the bug where handler's listener consumed messages meant
                                // for ContactManager's listener (which processes Packet messages).
                                drop(receiver);
                                debug!(
                                    peer_did = %contact.peer_did,
                                    "Dropped handler receiver - ContactManager will subscribe for receiving"
                                );
                            }
                            Err(e) => {
                                error!(
                                    peer_did = %contact.peer_did,
                                    ?contact_topic_id,
                                    error = ?e,
                                    "Failed to subscribe to contact topic (non-fatal)"
                                );
                            }
                        }

                        // Subscribe to the contact's profile topic (per-peer profile updates)
                        let peer_profile_topic = derive_profile_topic(&accepter_did);
                        let bootstrap_peer = iroh::PublicKey::from_bytes(&node_addr.node_id)
                            .expect("node_id should be valid PublicKey bytes");
                        match gossip.subscribe(peer_profile_topic, vec![bootstrap_peer]).await {
                            Ok(_) => {
                                info!(
                                    peer_did = %accepter_did,
                                    ?peer_profile_topic,
                                    "Subscribed to contact's profile topic"
                                );
                            }
                            Err(e) => {
                                error!(
                                    peer_did = %accepter_did,
                                    ?peer_profile_topic,
                                    error = ?e,
                                    "Failed to subscribe to contact's profile topic (non-fatal)"
                                );
                            }
                        }

                        let _ = event_tx.send(ContactEvent::ContactAccepted {
                            contact: contact.clone(),
                        });

                        // Also emit online event since we just received their message
                        let _ = event_tx.send(ContactEvent::ContactOnline {
                            did: contact.peer_did.clone(),
                        });
                    } else {
                        debug!(
                            invite_id = ?invite_id,
                            state = ?pending.state,
                            "Received ContactAccept but not in OutgoingPending state"
                        );
                    }
                }
            }

            ContactMessage::ContactDecline { invite_id } => {
                debug!(invite_id = ?invite_id, "Received ContactDecline");

                // Delete pending and emit event
                storage.delete_pending(&invite_id)?;
                info!(invite_id = ?invite_id, "Contact request was declined");

                let _ = event_tx.send(ContactEvent::ContactDeclined { invite_id });
            }
        }

        Ok(())
    }
}

impl ProtocolHandler for ContactProtocolHandler {
    fn accept(
        &self,
        conn: Connection,
    ) -> impl std::future::Future<Output = Result<(), iroh::protocol::AcceptError>> + Send {
        let storage = self.storage.clone();
        let event_tx = self.event_tx.clone();
        let gossip = self.gossip.clone();
        let static_discovery = self.static_discovery.clone();
        let local_did = self.local_did.clone();
        let active_topics = self.active_topics.clone();

        async move {
            debug!(peer = %conn.remote_id(), "Router accepting contact connection");

            // Process the connection fully before returning
            if let Err(e) = Self::handle_connection(conn, storage, event_tx, gossip, static_discovery, local_did, active_topics).await {
                error!(error = ?e, "Failed to handle contact connection");
                return Err(iroh::protocol::AcceptError::from_err(e));
            }

            Ok(())
        }
    }
}

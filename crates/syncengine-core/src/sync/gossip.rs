//! Gossip-based P2P networking using iroh-gossip
//!
//! Provides multi-peer broadcast synchronization for realms.

use std::sync::Arc;

use iroh::discovery::static_provider::StaticProvider;
use iroh::protocol::Router;
use iroh::{Endpoint, EndpointAddr, PublicKey, SecretKey};
use iroh_gossip::net::{Gossip, GOSSIP_ALPN};
use iroh_gossip::proto::TopicId;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::blobs::{BlobManager, BlobProtocolHandler};
use crate::error::{SyncError, SyncResult};
use crate::identity::{Did, HybridKeypair};
use crate::invite::{InviteTicket, NodeAddrBytes};
use crate::sync::contact_handler::ContactProtocolHandler;
use crate::sync::contact_protocol::CONTACT_ALPN;
use crate::sync::profile_protocol::{ProfileProtocolHandler, PROFILE_ALPN};
use crate::types::RealmId;

/// Message received from a gossip topic
#[derive(Debug, Clone)]
pub struct GossipMessage {
    /// The sender's public key
    pub from: PublicKey,
    /// The raw message content
    pub content: Vec<u8>,
}

/// Event from a gossip topic (message or neighbor change)
#[derive(Debug)]
pub enum TopicEvent {
    /// A message was received from a peer
    Message(GossipMessage),
    /// A neighbor joined the topic
    NeighborUp(PublicKey),
    /// A neighbor left the topic
    NeighborDown(PublicKey),
}

/// Handle to a subscribed gossip topic for sending messages
///
/// The sender can be cloned and shared across threads.
/// The receiver is returned separately by subscribe() for direct polling.
#[derive(Clone)]
pub struct TopicSender {
    sender: Arc<Mutex<iroh_gossip::api::GossipSender>>,
    topic_id: TopicId,
}

/// Handle to receive messages from a gossip topic
///
/// This should be polled directly by a single task - not shared via Arc<Mutex<...>>.
pub struct TopicReceiver {
    receiver: iroh_gossip::api::GossipReceiver,
    topic_id: TopicId,
}

impl TopicSender {
    /// Broadcast a message to all peers on this topic
    pub async fn broadcast(&self, msg: impl Into<Vec<u8>>) -> SyncResult<()> {
        let data: Vec<u8> = msg.into();
        debug!(topic = ?self.topic_id, len = data.len(), "Broadcasting message");

        self.sender
            .lock()
            .await
            .broadcast(data.into())
            .await
            .map_err(|e| SyncError::Gossip(format!("Failed to broadcast: {}", e)))?;

        Ok(())
    }

    /// Get the topic ID
    pub fn topic_id(&self) -> TopicId {
        self.topic_id
    }
}

impl TopicReceiver {
    /// Receive the next event from the topic (message or neighbor change)
    ///
    /// This should be called from a single task that owns the receiver.
    /// Returns None if the topic subscription is closed.
    pub async fn recv_event(&mut self) -> Option<TopicEvent> {
        use iroh_gossip::api::Event;
        use n0_future::StreamExt;

        loop {
            match self.receiver.try_next().await {
                Ok(Some(event)) => {
                    match event {
                        Event::Received(msg) => {
                            debug!(topic = ?self.topic_id, from = ?msg.delivered_from, "Received message");
                            return Some(TopicEvent::Message(GossipMessage {
                                from: msg.delivered_from,
                                content: msg.content.to_vec(),
                            }));
                        }
                        Event::NeighborUp(peer) => {
                            info!(topic = ?self.topic_id, ?peer, "Neighbor joined");
                            return Some(TopicEvent::NeighborUp(peer));
                        }
                        Event::NeighborDown(peer) => {
                            info!(topic = ?self.topic_id, ?peer, "Neighbor left");
                            return Some(TopicEvent::NeighborDown(peer));
                        }
                        Event::Lagged => {
                            warn!(topic = ?self.topic_id, "Lagged behind on topic");
                            // Continue waiting for actual events
                        }
                    }
                }
                Ok(None) => {
                    debug!(topic = ?self.topic_id, "Topic subscription closed");
                    return None;
                }
                Err(e) => {
                    warn!(topic = ?self.topic_id, error = ?e, "Error receiving from topic");
                    return None;
                }
            }
        }
    }

    /// Check if we have joined the topic swarm
    pub fn is_joined(&self) -> bool {
        self.receiver.is_joined()
    }

    /// Wait until we are connected to at least one peer on this topic.
    ///
    /// This is important for ensuring the gossip mesh is formed before
    /// starting to receive messages. Without waiting, the receiver loop
    /// may exit immediately because there are no connected peers.
    ///
    /// # Returns
    ///
    /// Ok(()) when connected to at least one peer, or Err if the connection fails.
    pub async fn joined(&mut self) -> SyncResult<()> {
        self.receiver
            .joined()
            .await
            .map_err(|e| SyncError::Gossip(format!("Failed to join topic swarm: {}", e)))
    }

    /// Get the topic ID
    pub fn topic_id(&self) -> TopicId {
        self.topic_id
    }
}

/// Legacy handle to a subscribed gossip topic (for compatibility)
///
/// Allows sending and receiving messages on a specific topic.
pub struct TopicHandle {
    sender: Arc<Mutex<iroh_gossip::api::GossipSender>>,
    receiver: Arc<Mutex<iroh_gossip::api::GossipReceiver>>,
    topic_id: TopicId,
}

impl TopicHandle {
    /// Broadcast a message to all peers on this topic
    pub async fn broadcast(&self, msg: impl Into<Vec<u8>>) -> SyncResult<()> {
        let data: Vec<u8> = msg.into();
        debug!(topic = ?self.topic_id, len = data.len(), "Broadcasting message");

        self.sender
            .lock()
            .await
            .broadcast(data.into())
            .await
            .map_err(|e| SyncError::Gossip(format!("Failed to broadcast: {}", e)))?;

        Ok(())
    }

    /// Receive the next message from peers
    ///
    /// Returns None if the topic subscription is closed.
    pub async fn recv(&self) -> Option<GossipMessage> {
        loop {
            match self.recv_event().await {
                Some(TopicEvent::Message(msg)) => return Some(msg),
                Some(TopicEvent::NeighborUp(_) | TopicEvent::NeighborDown(_)) => {
                    // Skip neighbor events, continue waiting for messages
                    continue;
                }
                None => return None,
            }
        }
    }

    /// Receive the next event from the topic (message or neighbor change)
    ///
    /// Returns all events including NeighborUp/NeighborDown, not just messages.
    /// This is useful for listeners that need to track peer connections.
    ///
    /// Returns None if the topic subscription is closed.
    pub async fn recv_event(&self) -> Option<TopicEvent> {
        use iroh_gossip::api::Event;
        use n0_future::StreamExt;

        let mut receiver = self.receiver.lock().await;

        loop {
            match receiver.try_next().await {
                Ok(Some(event)) => {
                    match event {
                        Event::Received(msg) => {
                            debug!(topic = ?self.topic_id, from = ?msg.delivered_from, "Received message");
                            return Some(TopicEvent::Message(GossipMessage {
                                from: msg.delivered_from,
                                content: msg.content.to_vec(),
                            }));
                        }
                        Event::NeighborUp(peer) => {
                            info!(topic = ?self.topic_id, ?peer, "Neighbor joined");
                            return Some(TopicEvent::NeighborUp(peer));
                        }
                        Event::NeighborDown(peer) => {
                            info!(topic = ?self.topic_id, ?peer, "Neighbor left");
                            return Some(TopicEvent::NeighborDown(peer));
                        }
                        Event::Lagged => {
                            warn!(topic = ?self.topic_id, "Lagged behind on topic");
                            // Continue waiting for actual events
                        }
                    }
                }
                Ok(None) => {
                    debug!(topic = ?self.topic_id, "Topic subscription closed");
                    return None;
                }
                Err(e) => {
                    warn!(topic = ?self.topic_id, error = ?e, "Error receiving from topic");
                    return None;
                }
            }
        }
    }

    /// Check if we have joined the topic swarm
    pub async fn is_joined(&self) -> bool {
        self.receiver.lock().await.is_joined()
    }

    /// Wait for a neighbor to join the topic
    ///
    /// This blocks until a NeighborUp event is received or the timeout expires.
    /// Returns true if a neighbor was found, false if timeout occurred.
    /// Any messages received while waiting are discarded (they should be
    /// handled by the main listener which runs in parallel).
    pub async fn wait_for_neighbor(&self, timeout: std::time::Duration) -> bool {
        use iroh_gossip::api::Event;
        use n0_future::StreamExt;

        let result = tokio::time::timeout(timeout, async {
            let mut receiver = self.receiver.lock().await;
            loop {
                match receiver.try_next().await {
                    Ok(Some(event)) => {
                        match event {
                            Event::NeighborUp(peer) => {
                                info!(topic = ?self.topic_id, ?peer, "Neighbor found while waiting");
                                return true;
                            }
                            Event::Received(_) => {
                                // Message received while waiting - continue waiting
                                // The main listener will handle messages
                                debug!(topic = ?self.topic_id, "Message received while waiting for neighbor");
                            }
                            Event::NeighborDown(_) | Event::Lagged => {
                                // Continue waiting
                            }
                        }
                    }
                    Ok(None) => {
                        debug!(topic = ?self.topic_id, "Topic closed while waiting for neighbor");
                        return false;
                    }
                    Err(e) => {
                        warn!(topic = ?self.topic_id, error = ?e, "Error while waiting for neighbor");
                        return false;
                    }
                }
            }
        })
        .await;

        match result {
            Ok(found) => found,
            Err(_) => {
                debug!(topic = ?self.topic_id, "Timeout waiting for neighbor");
                false
            }
        }
    }

    /// Get the topic ID this handle is subscribed to
    pub fn topic_id(&self) -> TopicId {
        self.topic_id
    }
}

/// Main gossip synchronization engine
///
/// Manages an iroh endpoint with gossip protocol support,
/// allowing subscription to multiple topics for multi-realm sync.
#[derive(Debug)]
pub struct GossipSync {
    endpoint: Endpoint,
    gossip: Gossip,
    router: Router,
    /// Static discovery provider for adding out-of-band peer addresses
    static_provider: StaticProvider,
    #[allow(dead_code)]
    secret_key: SecretKey,
}

impl GossipSync {
    /// Create a new gossip sync instance
    ///
    /// Spawns an iroh endpoint with gossip protocol support.
    /// The endpoint will be reachable by other peers.
    pub async fn new() -> SyncResult<Self> {
        Self::with_secret_key(None, None, None, None).await
    }

    /// Create a new gossip sync instance with a specific secret key
    ///
    /// Useful for persistent identity across restarts.
    /// If storage and event_tx are provided, contact protocol handler will be registered.
    /// The local_did is required for the simplified contact protocol's local key derivation.
    /// If profile handler deps are provided, profile protocol handler will be registered.
    /// If blob manager is provided, blob protocol handler will be registered for P2P image transfer.
    pub async fn with_secret_key(
        secret_key: Option<SecretKey>,
        contact_handler_deps: Option<(Arc<crate::storage::Storage>, tokio::sync::broadcast::Sender<crate::sync::ContactEvent>, String)>,
        profile_handler_deps: Option<(Arc<crate::storage::Storage>, Arc<HybridKeypair>, Did)>,
        blob_manager: Option<&BlobManager>,
    ) -> SyncResult<Self> {
        let secret_key = secret_key.unwrap_or_else(|| SecretKey::generate(&mut rand::rng()));

        // Create static provider for out-of-band peer addresses
        let static_provider = StaticProvider::new();

        // Build the list of ALPNs to support
        let mut alpns = vec![
            GOSSIP_ALPN.to_vec(),
            CONTACT_ALPN.to_vec(),
            PROFILE_ALPN.to_vec(),
        ];
        if blob_manager.is_some() {
            alpns.push(iroh_blobs::ALPN.to_vec());
        }

        // Build the endpoint with static discovery
        // Support gossip (realm sync), contact exchange, profile serving, and blob protocols
        let endpoint = Endpoint::builder()
            .secret_key(secret_key.clone())
            .alpns(alpns)
            .discovery(static_provider.clone())
            .bind()
            .await
            .map_err(|e| SyncError::Network(format!("Failed to bind endpoint: {}", e)))?;

        let endpoint_id = endpoint.id();
        info!(%endpoint_id, "Endpoint bound");

        // Spawn gossip protocol handler with increased message size limit
        // Default is 4KB, but Automerge documents + envelope overhead can exceed this.
        // Use 1MB to support larger documents with many tasks.
        const MAX_MESSAGE_SIZE: usize = 1024 * 1024; // 1MB
        let gossip = Gossip::builder()
            .max_message_size(MAX_MESSAGE_SIZE)
            .spawn(endpoint.clone());
        info!(max_message_size = MAX_MESSAGE_SIZE, "Gossip spawned");

        // Build router - register contact, profile, and blob protocols if dependencies provided
        let mut router_builder = Router::builder(endpoint.clone()).accept(GOSSIP_ALPN, gossip.clone());

        if let Some((storage, event_tx, local_did)) = contact_handler_deps {
            let contact_handler = ContactProtocolHandler::new(
                storage,
                event_tx,
                gossip.clone(),
                static_provider.clone(),
                local_did,
            );
            router_builder = router_builder.accept(CONTACT_ALPN, contact_handler);
            info!("Contact protocol handler registered");
        }

        if let Some((storage, keypair, did)) = profile_handler_deps {
            let profile_handler = ProfileProtocolHandler::new(storage, keypair, did);
            router_builder = router_builder.accept(PROFILE_ALPN, profile_handler);
            info!("Profile protocol handler registered");
        }

        if let Some(manager) = blob_manager {
            let blob_handler = BlobProtocolHandler::from_manager(manager);
            router_builder = router_builder.accept(BlobProtocolHandler::alpn(), blob_handler.protocol());
            info!("Blob protocol handler registered for P2P image transfer");
        }

        let router = router_builder.spawn();
        info!("Router spawned");

        Ok(Self {
            endpoint,
            gossip,
            router,
            static_provider,
            secret_key,
        })
    }

    /// Get this node's endpoint ID
    ///
    /// This is the public identifier other peers use to connect.
    pub fn endpoint_id(&self) -> iroh::EndpointId {
        self.endpoint.id()
    }

    /// Get this node's public key
    pub fn public_key(&self) -> PublicKey {
        self.endpoint.id()
    }

    /// Add a peer's address to the static discovery provider
    ///
    /// This makes the peer's address known to iroh for faster connection
    /// establishment, without relying on DNS-based discovery.
    pub fn add_peer_addr(&self, endpoint_addr: EndpointAddr) {
        info!(
            peer = %endpoint_addr.id,
            addrs = endpoint_addr.addrs.len(),
            "Adding peer address to static discovery"
        );
        self.static_provider.add_endpoint_info(endpoint_addr);
    }

    /// Subscribe to a gossip topic (returns split sender/receiver)
    ///
    /// This is the preferred method for new code. The receiver should be polled
    /// directly by a single task - not wrapped in Arc<Mutex<...>>.
    ///
    /// # Arguments
    /// * `topic_id` - The topic to subscribe to (typically derived from a realm ID)
    /// * `bootstrap_peers` - Initial peers to connect to (can be empty for first node)
    pub async fn subscribe_split(
        &self,
        topic_id: TopicId,
        bootstrap_peers: Vec<iroh::EndpointId>,
    ) -> SyncResult<(TopicSender, TopicReceiver)> {
        info!(
            ?topic_id,
            peer_count = bootstrap_peers.len(),
            "Subscribing to topic (split)"
        );

        let gossip_topic = self
            .gossip
            .subscribe(topic_id, bootstrap_peers)
            .await
            .map_err(|e| SyncError::Gossip(format!("Failed to subscribe: {}", e)))?;

        let (sender, receiver) = gossip_topic.split();

        Ok((
            TopicSender {
                sender: Arc::new(Mutex::new(sender)),
                topic_id,
            },
            TopicReceiver { receiver, topic_id },
        ))
    }

    /// Subscribe to a gossip topic (legacy API with Arc<Mutex<...>> wrapped receiver)
    ///
    /// # Arguments
    /// * `topic_id` - The topic to subscribe to (typically derived from a realm ID)
    /// * `bootstrap_peers` - Initial peers to connect to (can be empty for first node)
    pub async fn subscribe(
        &self,
        topic_id: TopicId,
        bootstrap_peers: Vec<iroh::EndpointId>,
    ) -> SyncResult<TopicHandle> {
        info!(
            ?topic_id,
            peer_count = bootstrap_peers.len(),
            "Subscribing to topic"
        );

        let gossip_topic = self
            .gossip
            .subscribe(topic_id, bootstrap_peers)
            .await
            .map_err(|e| SyncError::Gossip(format!("Failed to subscribe: {}", e)))?;

        let (sender, receiver) = gossip_topic.split();

        Ok(TopicHandle {
            sender: Arc::new(Mutex::new(sender)),
            receiver: Arc::new(Mutex::new(receiver)),
            topic_id,
        })
    }

    /// Get a reference to the underlying endpoint
    ///
    /// Useful for advanced operations like direct peer connections.
    pub fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }

    /// Get this node's current EndpointAddr
    ///
    /// Returns the full addressing information including relay URLs and
    /// direct IP addresses that can be used by other peers to connect.
    pub fn endpoint_addr(&self) -> EndpointAddr {
        self.endpoint.addr()
    }

    /// Join a gossip topic using an invite ticket
    ///
    /// This extracts bootstrap peer information from the invite and subscribes
    /// to the topic. The invite's realm key can be used separately for
    /// encrypting/decrypting realm data.
    ///
    /// # Arguments
    ///
    /// * `invite` - An `InviteTicket` received from another peer
    ///
    /// # Returns
    ///
    /// A `TopicHandle` for sending and receiving messages on the topic.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::InvalidInvite` if the invite is expired or has invalid
    /// bootstrap peer addresses.
    /// Returns `SyncError::Gossip` if subscription fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let invite = InviteTicket::decode(&invite_string)?;
    /// let handle = gossip.join_via_invite(&invite).await?;
    ///
    /// // Now we can send/receive messages
    /// handle.broadcast(b"Hello!").await?;
    /// ```
    pub async fn join_via_invite(&self, invite: &InviteTicket) -> SyncResult<TopicHandle> {
        // Check expiration
        if invite.is_expired() {
            return Err(SyncError::InvalidInvite("Invite has expired".to_string()));
        }

        let topic_id = invite.topic_id();
        info!(
            ?topic_id,
            peers = invite.bootstrap_peers.len(),
            "Joining via invite"
        );

        // Convert bootstrap peers from NodeAddrBytes to EndpointAddr
        // and add them to the static discovery provider for immediate connection
        let mut bootstrap_ids = Vec::with_capacity(invite.bootstrap_peers.len());

        for peer_bytes in &invite.bootstrap_peers {
            // Convert to EndpointAddr
            let endpoint_addr = peer_bytes.to_endpoint_addr()?;

            debug!(
                peer = %endpoint_addr.id,
                relay = ?peer_bytes.relay_url,
                addrs = peer_bytes.direct_addresses.len(),
                "Adding bootstrap peer to static discovery"
            );

            // Add the full address to static discovery for immediate connection
            // This avoids relying on slow DNS-based discovery
            self.add_peer_addr(endpoint_addr.clone());

            bootstrap_ids.push(endpoint_addr.id);
        }

        // Subscribe to the topic with bootstrap peers
        // Gossip can now immediately connect using the addresses we added
        self.subscribe(topic_id, bootstrap_ids).await
    }

    /// Join a gossip topic using an invite ticket (split version)
    ///
    /// Returns a split (sender, receiver) pair. The receiver should be polled
    /// directly by a single task, not wrapped in Arc<Mutex<...>>.
    pub async fn join_via_invite_split(
        &self,
        invite: &InviteTicket,
    ) -> SyncResult<(TopicSender, TopicReceiver)> {
        // Check expiration
        if invite.is_expired() {
            return Err(SyncError::InvalidInvite("Invite has expired".to_string()));
        }

        let topic_id = invite.topic_id();
        info!(
            ?topic_id,
            peers = invite.bootstrap_peers.len(),
            "Joining via invite (split)"
        );

        // Convert bootstrap peers from NodeAddrBytes to EndpointAddr
        // and add them to the static discovery provider for immediate connection
        let mut bootstrap_ids = Vec::with_capacity(invite.bootstrap_peers.len());

        for peer_bytes in &invite.bootstrap_peers {
            // Convert to EndpointAddr
            let endpoint_addr = peer_bytes.to_endpoint_addr()?;

            debug!(
                peer = %endpoint_addr.id,
                relay = ?peer_bytes.relay_url,
                addrs = peer_bytes.direct_addresses.len(),
                "Adding bootstrap peer to static discovery"
            );

            // Add the full address to static discovery for immediate connection
            // This avoids relying on slow DNS-based discovery
            self.add_peer_addr(endpoint_addr.clone());

            bootstrap_ids.push(endpoint_addr.id);
        }

        // Subscribe to the topic with bootstrap peers using split API
        self.subscribe_split(topic_id, bootstrap_ids).await
    }

    /// Generate an invite ticket for a realm we're subscribed to
    ///
    /// Creates an invite that includes this node as a bootstrap peer,
    /// allowing other nodes to join the realm's gossip topic.
    ///
    /// # Arguments
    ///
    /// * `realm_id` - The realm to generate an invite for
    /// * `realm_key` - The ChaCha20-Poly1305 encryption key for the realm
    ///
    /// # Returns
    ///
    /// An `InviteTicket` that can be encoded and shared with other peers.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let invite = gossip.generate_invite(&realm_id, realm_key, Some("My Realm"))?;
    /// let invite_string = invite.encode()?;
    /// // Share invite_string via QR code, link, etc.
    /// ```
    pub fn generate_invite(
        &self,
        realm_id: &RealmId,
        realm_key: [u8; 32],
        realm_name: Option<&str>,
    ) -> SyncResult<InviteTicket> {
        // Get our current endpoint address
        let our_addr = self.endpoint_addr();

        // Convert to NodeAddrBytes for serialization
        let our_node_addr = NodeAddrBytes::from_endpoint_addr(&our_addr);

        info!(
            realm = %realm_id,
            node_id = ?our_addr.id,
            relay = ?our_node_addr.relay_url,
            addrs = our_node_addr.direct_addresses.len(),
            "Generating invite"
        );

        // Create the invite with us as the only bootstrap peer
        let mut invite = InviteTicket::new(realm_id, realm_key, vec![our_node_addr]);

        // Include realm name if provided
        if let Some(name) = realm_name {
            invite = invite.with_name(name);
        }

        Ok(invite)
    }

    /// Generate an invite with additional bootstrap peers
    ///
    /// Like `generate_invite`, but allows including additional known peers
    /// as bootstrap nodes for better connectivity.
    ///
    /// # Arguments
    ///
    /// * `realm_id` - The realm to generate an invite for
    /// * `realm_key` - The encryption key for the realm
    /// * `additional_peers` - Other peers to include as bootstrap nodes
    pub fn generate_invite_with_peers(
        &self,
        realm_id: &RealmId,
        realm_key: [u8; 32],
        additional_peers: Vec<NodeAddrBytes>,
    ) -> SyncResult<InviteTicket> {
        // Get our current endpoint address
        let our_addr = self.endpoint_addr();
        let our_node_addr = NodeAddrBytes::from_endpoint_addr(&our_addr);

        // Combine our address with additional peers
        let mut bootstrap_peers = vec![our_node_addr];
        bootstrap_peers.extend(additional_peers);

        info!(
            realm = %realm_id,
            peers = bootstrap_peers.len(),
            "Generating invite with peers"
        );

        let invite = InviteTicket::new(realm_id, realm_key, bootstrap_peers);

        Ok(invite)
    }

    /// Gracefully shutdown the gossip sync engine
    pub async fn shutdown(self) -> SyncResult<()> {
        info!("Shutting down gossip sync");

        // Shutdown the router first
        if let Err(e) = self.router.shutdown().await {
            warn!(error = ?e, "Failed to shutdown router cleanly");
        }

        // Close the endpoint
        self.endpoint.close().await;
        info!("Gossip sync shutdown complete");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gossip_sync_creates() {
        let result = GossipSync::new().await;
        assert!(result.is_ok(), "Failed to create GossipSync: {:?}", result);

        let gossip = result.unwrap();
        // Verify we have a valid endpoint ID
        let id = gossip.endpoint_id();
        assert!(!id.to_string().is_empty());

        // Clean shutdown
        let shutdown_result = gossip.shutdown().await;
        assert!(shutdown_result.is_ok());
    }

    #[tokio::test]
    async fn test_gossip_sync_with_secret_key() {
        let secret_key = SecretKey::generate(&mut rand::rng());
        let expected_public = secret_key.public();

        let gossip = GossipSync::with_secret_key(Some(secret_key), None, None, None)
            .await
            .expect("Failed to create GossipSync with secret key");

        // Verify the public key matches
        assert_eq!(gossip.public_key(), expected_public);

        gossip.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_subscribe_to_topic() {
        let gossip = GossipSync::new()
            .await
            .expect("Failed to create GossipSync");

        // Create a random topic
        let topic_id = TopicId::from_bytes(rand::random());

        // Subscribe without bootstrap peers (we're the first node)
        let handle = gossip
            .subscribe(topic_id, vec![])
            .await
            .expect("Failed to subscribe to topic");

        // Verify the topic ID matches
        assert_eq!(handle.topic_id(), topic_id);

        gossip.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_broadcast_on_topic() {
        let gossip = GossipSync::new()
            .await
            .expect("Failed to create GossipSync");

        let topic_id = TopicId::from_bytes(rand::random());
        let handle = gossip
            .subscribe(topic_id, vec![])
            .await
            .expect("Failed to subscribe to topic");

        // Broadcasting without peers should still succeed (message just won't go anywhere)
        let result = handle.broadcast(b"test message".to_vec()).await;
        assert!(result.is_ok());

        gossip.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_topic_id_from_realm() {
        // Verify we can create TopicId from our RealmId bytes
        use crate::types::RealmId;

        let realm = RealmId::new();
        let topic_id = TopicId::from_bytes(*realm.as_bytes());

        // Topic ID should be valid
        assert_eq!(topic_id.as_bytes(), realm.as_bytes());
    }

    #[tokio::test]
    async fn test_generate_invite_includes_self_as_bootstrap() {
        let gossip = GossipSync::new()
            .await
            .expect("Failed to create GossipSync");

        let realm_id = RealmId::new();
        let realm_key = [42u8; 32];

        // Generate an invite
        let invite = gossip
            .generate_invite(&realm_id, realm_key, None)
            .expect("Failed to generate invite");

        // Verify the invite contains our node as a bootstrap peer
        assert_eq!(invite.bootstrap_peers.len(), 1);

        // The bootstrap peer should have our node ID
        let our_public_key = gossip.public_key();
        assert_eq!(
            invite.bootstrap_peers[0].node_id,
            *our_public_key.as_bytes()
        );

        // The topic should match the realm
        assert_eq!(invite.topic, *realm_id.as_bytes());
        assert_eq!(invite.realm_key, realm_key);

        gossip.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_generate_invite_with_peers_includes_all_peers() {
        let gossip = GossipSync::new()
            .await
            .expect("Failed to create GossipSync");

        let realm_id = RealmId::new();
        let realm_key = [42u8; 32];

        // Create some additional peer addresses
        let additional_peer = NodeAddrBytes {
            node_id: [1u8; 32],
            relay_url: Some("https://other-relay.example.com".to_string()),
            direct_addresses: vec!["10.0.0.1:1234".to_string()],
        };

        // Generate an invite with additional peers
        let invite = gossip
            .generate_invite_with_peers(&realm_id, realm_key, vec![additional_peer.clone()])
            .expect("Failed to generate invite");

        // Verify we have both our node and the additional peer
        assert_eq!(invite.bootstrap_peers.len(), 2);

        // First peer should be us
        let our_public_key = gossip.public_key();
        assert_eq!(
            invite.bootstrap_peers[0].node_id,
            *our_public_key.as_bytes()
        );

        // Second peer should be the additional one
        assert_eq!(invite.bootstrap_peers[1].node_id, additional_peer.node_id);
        assert_eq!(
            invite.bootstrap_peers[1].relay_url,
            additional_peer.relay_url
        );

        gossip.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_join_via_invite_connects_to_topic() {
        // Create the host node
        let host = GossipSync::new()
            .await
            .expect("Failed to create host GossipSync");

        let realm_id = RealmId::new();
        let realm_key = [42u8; 32];

        // Host subscribes to the topic first
        let topic_id = TopicId::from_bytes(*realm_id.as_bytes());
        let _host_handle = host
            .subscribe(topic_id, vec![])
            .await
            .expect("Failed to subscribe to topic");

        // Generate an invite from the host
        let invite = host
            .generate_invite(&realm_id, realm_key, Some("Test Realm"))
            .expect("Failed to generate invite");

        // Create a joining node
        let joiner = GossipSync::new()
            .await
            .expect("Failed to create joiner GossipSync");

        // Join via the invite
        let joiner_handle = joiner
            .join_via_invite(&invite)
            .await
            .expect("Failed to join via invite");

        // Verify the joiner is subscribed to the correct topic
        assert_eq!(joiner_handle.topic_id(), topic_id);

        // Clean up
        joiner.shutdown().await.unwrap();
        host.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_join_via_invite_rejects_expired() {
        let gossip = GossipSync::new()
            .await
            .expect("Failed to create GossipSync");

        let realm_id = RealmId::new();
        let realm_key = [42u8; 32];

        // Create an expired invite
        let expired_invite = InviteTicket::new(
            &realm_id,
            realm_key,
            vec![NodeAddrBytes {
                node_id: *gossip.public_key().as_bytes(),
                relay_url: None,
                direct_addresses: vec![],
            }],
        )
        .with_expiry(0); // Unix epoch = already expired

        // Attempt to join should fail
        let result = gossip.join_via_invite(&expired_invite).await;
        assert!(result.is_err());

        // Use match to avoid needing Debug on TopicHandle
        match result {
            Err(SyncError::InvalidInvite(msg)) => {
                assert!(
                    msg.contains("expired"),
                    "Expected 'expired' in error message, got: {}",
                    msg
                );
            }
            Err(other) => panic!("Expected InvalidInvite error, got: {:?}", other),
            Ok(_) => panic!("Expected error, but got Ok"),
        }

        gossip.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_invite_roundtrip_encode_decode() {
        let gossip = GossipSync::new()
            .await
            .expect("Failed to create GossipSync");

        let realm_id = RealmId::new();
        let realm_key = [42u8; 32];

        // Generate an invite
        let invite = gossip
            .generate_invite(&realm_id, realm_key, Some("My Realm"))
            .expect("Failed to generate invite");

        // Encode and decode
        let encoded = invite.encode().expect("Failed to encode");
        let decoded = InviteTicket::decode(&encoded).expect("Failed to decode");

        // Verify the decoded invite matches
        assert_eq!(decoded.topic, invite.topic);
        assert_eq!(decoded.realm_key, invite.realm_key);
        assert_eq!(decoded.bootstrap_peers.len(), 1);
        assert_eq!(
            decoded.bootstrap_peers[0].node_id,
            invite.bootstrap_peers[0].node_id
        );

        gossip.shutdown().await.unwrap();
    }
}

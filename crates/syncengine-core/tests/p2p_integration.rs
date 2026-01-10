//! P2P Integration Tests
//!
//! These tests verify gossip-based sync between multiple nodes.
//!
//! ## Milestone Tests
//!
//! - Milestone 0.5: Verify iroh endpoint can be created
//! - Milestone 0.5b: Verify two endpoints can be created
//! - Milestone 1: Two nodes can exchange messages via gossip (gossip echo)
//! - Milestone 2: Bidirectional gossip communication
//! - Milestone 3: Three-node gossip propagation
//! - Milestone 4: Late joiner receives messages
//! - Milestone 2.0: Automerge document sync via gossip
//! - Milestone 2.5: Concurrent edits merge correctly

use std::collections::HashSet;
use std::time::Duration;

use bytes::Bytes;
use iroh::discovery::static_provider::StaticProvider;
use iroh::protocol::Router;
use iroh_gossip::api::{Event as GossipEvent, GossipReceiver};
use iroh_gossip::net::{Gossip, GOSSIP_ALPN};
use iroh_gossip::proto::TopicId;
use n0_future::StreamExt;

// ============================================================================
// Test Utilities
// ============================================================================

/// Generate a random topic ID for testing
fn random_topic() -> TopicId {
    TopicId::from_bytes(rand::random())
}

/// A test node that wraps endpoint, gossip, router, and discovery
struct TestNode {
    endpoint: iroh::Endpoint,
    gossip: Gossip,
    router: Router,
    discovery: StaticProvider,
}

impl TestNode {
    /// Create a new test node with fresh identity
    async fn new() -> anyhow::Result<Self> {
        let secret_key = iroh::SecretKey::generate(&mut rand::rng());
        Self::with_secret_key(secret_key).await
    }

    /// Create a test node with a specific secret key (for deterministic IDs)
    async fn with_secret_key(secret_key: iroh::SecretKey) -> anyhow::Result<Self> {
        // Create a static provider for manually adding peer addresses
        let discovery = StaticProvider::default();

        let endpoint = iroh::Endpoint::builder()
            .secret_key(secret_key)
            .alpns(vec![GOSSIP_ALPN.to_vec()])
            .discovery(discovery.clone())
            .bind()
            .await?;

        let gossip = Gossip::builder().spawn(endpoint.clone());

        let router = Router::builder(endpoint.clone())
            .accept(GOSSIP_ALPN, gossip.clone())
            .spawn();

        Ok(Self {
            endpoint,
            gossip,
            router,
            discovery,
        })
    }

    /// Get the endpoint ID (node ID) of this node
    fn id(&self) -> iroh::EndpointId {
        self.endpoint.id()
    }

    /// Get the full endpoint address (includes socket addresses)
    fn addr(&self) -> iroh::EndpointAddr {
        self.endpoint.addr()
    }

    /// Add another node's address to this node's discovery
    fn add_peer(&self, addr: iroh::EndpointAddr) {
        self.discovery.add_endpoint_info(addr);
    }

    /// Subscribe to a gossip topic with optional bootstrap peers
    async fn subscribe(
        &self,
        topic: TopicId,
        bootstrap: Vec<iroh::EndpointId>,
    ) -> anyhow::Result<iroh_gossip::api::GossipTopic> {
        Ok(self.gossip.subscribe(topic, bootstrap).await?)
    }

    /// Clean shutdown of the node
    async fn shutdown(self) {
        let _ = self.router.shutdown().await;
        self.endpoint.close().await;
    }
}

/// Wait for a NeighborUp event on the receiver, returning the updated receiver
/// and any message received during waiting
async fn wait_for_neighbor(
    mut receiver: GossipReceiver,
    timeout_secs: u64,
) -> anyhow::Result<(GossipReceiver, Option<Vec<u8>>)> {
    let mut message_received = None;

    let result = tokio::time::timeout(Duration::from_secs(timeout_secs), async {
        loop {
            match receiver.try_next().await? {
                Some(GossipEvent::NeighborUp(_)) => {
                    return anyhow::Ok(());
                }
                Some(GossipEvent::Received(msg)) => {
                    message_received = Some(msg.content.to_vec());
                }
                Some(_) => continue,
                None => anyhow::bail!("Stream ended"),
            }
        }
    })
    .await;

    match result {
        Ok(Ok(())) => Ok((receiver, message_received)),
        Ok(Err(e)) => Err(e),
        Err(_) => anyhow::bail!("Timeout waiting for neighbor"),
    }
}

/// Wait for a message on the receiver
async fn wait_for_message(
    mut receiver: GossipReceiver,
    timeout_secs: u64,
) -> anyhow::Result<(Vec<u8>, GossipReceiver)> {
    let result = tokio::time::timeout(Duration::from_secs(timeout_secs), async {
        loop {
            match receiver.try_next().await? {
                Some(GossipEvent::Received(msg)) => {
                    return anyhow::Ok(msg.content.to_vec());
                }
                Some(_) => continue, // Skip other events
                None => anyhow::bail!("Stream ended"),
            }
        }
    })
    .await;

    match result {
        Ok(Ok(msg)) => Ok((msg, receiver)),
        Ok(Err(e)) => Err(e),
        Err(_) => anyhow::bail!("Timeout waiting for message"),
    }
}

// ============================================================================
// Milestone 0.5 Tests - Endpoint Creation
// ============================================================================

/// Milestone 0.5: Verify iroh endpoint can be created
///
/// This is the most basic test - can we create an iroh endpoint
/// that can participate in the P2P network?
#[tokio::test]
async fn test_endpoint_binding() {
    // Create an iroh endpoint
    let endpoint = iroh::Endpoint::builder()
        .bind()
        .await
        .expect("Failed to bind endpoint");

    // Get endpoint ID (node ID)
    let endpoint_id = endpoint.id();
    println!("Endpoint ID: {}", endpoint_id);

    // Verify we got a valid endpoint ID (not empty)
    assert!(!endpoint_id.to_string().is_empty());

    // Clean shutdown
    endpoint.close().await;
}

/// Milestone 0.5b: Verify two endpoints can be created
///
/// This tests that we can create multiple independent nodes,
/// which is the foundation for P2P communication.
#[tokio::test]
async fn test_two_endpoints() {
    let endpoint_a = iroh::Endpoint::builder()
        .bind()
        .await
        .expect("Failed to bind endpoint A");

    let endpoint_b = iroh::Endpoint::builder()
        .bind()
        .await
        .expect("Failed to bind endpoint B");

    // Endpoint IDs should be different
    assert_ne!(endpoint_a.id(), endpoint_b.id());

    println!("Endpoint A: {}", endpoint_a.id());
    println!("Endpoint B: {}", endpoint_b.id());

    // Clean shutdown
    endpoint_a.close().await;
    endpoint_b.close().await;
}

/// Test that endpoints can be created with a specific secret key
///
/// Using a secret key ensures deterministic endpoint IDs for testing.
#[tokio::test]
async fn test_endpoint_with_secret_key() {
    // Generate a random secret key
    let secret_key = iroh::SecretKey::generate(&mut rand::rng());
    let expected_public = secret_key.public();

    let endpoint = iroh::Endpoint::builder()
        .secret_key(secret_key)
        .bind()
        .await
        .expect("Failed to bind endpoint");

    // The endpoint ID should match the public key from our secret
    println!("Endpoint ID: {}", endpoint.id());
    println!("Expected public: {}", expected_public);

    // Verify the endpoint ID is derived from our secret key
    assert_eq!(endpoint.id().to_string(), expected_public.to_string());

    endpoint.close().await;
}

/// Test endpoint creation with timeout
///
/// Ensures endpoint binding completes in reasonable time.
#[tokio::test]
async fn test_endpoint_creation_timeout() {
    let result = tokio::time::timeout(Duration::from_secs(10), async {
        iroh::Endpoint::builder().bind().await
    })
    .await;

    assert!(result.is_ok(), "Endpoint creation timed out");
    let endpoint = result.unwrap().expect("Failed to bind endpoint");
    endpoint.close().await;
}

/// Test that TestNode helper works correctly
#[tokio::test]
async fn test_node_creation() {
    let node = TestNode::new().await.expect("Failed to create test node");
    assert!(!node.id().to_string().is_empty());
    node.shutdown().await;
}

// ============================================================================
// Milestone 1 Tests - Gossip Echo
// ============================================================================

/// Milestone 1: Two nodes can exchange messages via gossip
///
/// This is the foundation of P2P sync - verify that two nodes can:
/// 1. Subscribe to the same gossip topic
/// 2. One node sends a message
/// 3. The other node receives it
#[tokio::test]
async fn test_gossip_echo() {
    // Initialize tracing for debugging (ok if already initialized)
    let _ = tracing_subscriber::fmt::try_init();

    // 1. Create two test nodes
    let node_a = TestNode::new().await.expect("Failed to create node A");
    let node_b = TestNode::new().await.expect("Failed to create node B");

    println!("Node A ID: {}", node_a.id());
    println!("Node B ID: {}", node_b.id());

    // 2. Share address information between nodes (required for local P2P)
    // Node B needs to know how to reach Node A
    node_b.add_peer(node_a.addr());
    // Node A needs to know how to reach Node B (for bidirectional comms)
    node_a.add_peer(node_b.addr());

    // 3. Generate a random topic for this test
    let topic = random_topic();
    println!("Topic: {:?}", topic);

    // 4. Node A subscribes to topic with no bootstrap peers (it's the first node)
    let topic_a = node_a
        .subscribe(topic, vec![])
        .await
        .expect("Node A failed to subscribe");
    let (_sender_a, receiver_a) = topic_a.split();

    // 5. Node B subscribes to topic with node A as bootstrap peer
    let topic_b = node_b
        .subscribe(topic, vec![node_a.id()])
        .await
        .expect("Node B failed to subscribe");
    let (sender_b, receiver_b) = topic_b.split();

    // 6. Wait for neighbors to connect (both sides)
    let (receiver_a, early_msg_a) = wait_for_neighbor(receiver_a, 10)
        .await
        .expect("Node A: no neighbor");
    let (receiver_b, _) = wait_for_neighbor(receiver_b, 10)
        .await
        .expect("Node B: no neighbor");

    // 7. Node B broadcasts a message
    let test_message = b"hello from B";
    sender_b
        .broadcast(Bytes::from_static(test_message))
        .await
        .expect("Failed to broadcast message");

    // 8. Node A should receive the message
    // Check if we already got it while waiting for neighbor
    let received = if let Some(msg) = early_msg_a {
        msg
    } else {
        let (msg, _) = wait_for_message(receiver_a, 5)
            .await
            .expect("Node A didn't receive message");
        msg
    };

    // 9. Assert the message was received correctly
    assert_eq!(received, test_message.to_vec(), "Message content mismatch");

    // Clean shutdown
    // Drain receiver_b to prevent warnings
    drop(receiver_b);
    node_a.shutdown().await;
    node_b.shutdown().await;
}

// ============================================================================
// Milestone 2 Tests - Bidirectional Gossip
// ============================================================================

/// Milestone 2: Both nodes can send and receive messages
///
/// Verifies full bidirectional communication works in the gossip mesh.
#[tokio::test]
async fn test_gossip_bidirectional() {
    let _ = tracing_subscriber::fmt::try_init();

    // Create two test nodes
    let node_a = TestNode::new().await.expect("Failed to create node A");
    let node_b = TestNode::new().await.expect("Failed to create node B");

    // Share addresses between nodes
    node_b.add_peer(node_a.addr());
    node_a.add_peer(node_b.addr());

    let topic = random_topic();

    // Both nodes subscribe to the same topic
    let topic_a = node_a
        .subscribe(topic, vec![])
        .await
        .expect("Node A failed to subscribe");
    let (sender_a, receiver_a) = topic_a.split();

    let topic_b = node_b
        .subscribe(topic, vec![node_a.id()])
        .await
        .expect("Node B failed to subscribe");
    let (sender_b, receiver_b) = topic_b.split();

    // Wait for both to have neighbors
    let (receiver_a, _) = wait_for_neighbor(receiver_a, 10)
        .await
        .expect("Node A: no neighbor");
    let (receiver_b, _) = wait_for_neighbor(receiver_b, 10)
        .await
        .expect("Node B: no neighbor");

    // Node A sends "from A"
    let msg_from_a = b"from A";
    sender_a
        .broadcast(Bytes::from_static(msg_from_a))
        .await
        .expect("Failed to send from A");

    // Node B should receive it
    let (received_at_b, receiver_b) = wait_for_message(receiver_b, 5)
        .await
        .expect("Node B didn't receive message from A");
    assert_eq!(
        received_at_b,
        msg_from_a.to_vec(),
        "Node B received wrong message"
    );

    // Node B sends "from B"
    let msg_from_b = b"from B";
    sender_b
        .broadcast(Bytes::from_static(msg_from_b))
        .await
        .expect("Failed to send from B");

    // Node A should receive it
    let (received_at_a, _) = wait_for_message(receiver_a, 5)
        .await
        .expect("Node A didn't receive message from B");
    assert_eq!(
        received_at_a,
        msg_from_b.to_vec(),
        "Node A received wrong message"
    );

    // Clean shutdown
    drop(receiver_b);
    node_a.shutdown().await;
    node_b.shutdown().await;
}

// ============================================================================
// Milestone 3 Tests - Three Node Propagation
// ============================================================================

/// Milestone 3: Messages propagate to all nodes in the swarm
///
/// Tests that gossip properly propagates messages through a multi-node mesh:
/// - Three nodes subscribe to the same topic
/// - A message from one node reaches all other nodes
#[tokio::test]
async fn test_three_node_propagation() {
    let _ = tracing_subscriber::fmt::try_init();

    // Create three test nodes
    let node_a = TestNode::new().await.expect("Failed to create node A");
    let node_b = TestNode::new().await.expect("Failed to create node B");
    let node_c = TestNode::new().await.expect("Failed to create node C");

    println!("Node A: {}", node_a.id());
    println!("Node B: {}", node_b.id());
    println!("Node C: {}", node_c.id());

    // Share addresses between all nodes
    node_b.add_peer(node_a.addr());
    node_c.add_peer(node_a.addr());
    node_a.add_peer(node_b.addr());
    node_a.add_peer(node_c.addr());
    node_b.add_peer(node_c.addr());
    node_c.add_peer(node_b.addr());

    let topic = random_topic();

    // Node A subscribes first (no bootstrap)
    let topic_a = node_a
        .subscribe(topic, vec![])
        .await
        .expect("Node A failed to subscribe");
    let (sender_a, receiver_a) = topic_a.split();

    // Node B bootstraps from A
    let topic_b = node_b
        .subscribe(topic, vec![node_a.id()])
        .await
        .expect("Node B failed to subscribe");
    let (_sender_b, receiver_b) = topic_b.split();

    // Node C bootstraps from A (could also bootstrap from B)
    let topic_c = node_c
        .subscribe(topic, vec![node_a.id()])
        .await
        .expect("Node C failed to subscribe");
    let (sender_c, receiver_c) = topic_c.split();

    // Wait for all nodes to have neighbors
    let (receiver_a, _) = wait_for_neighbor(receiver_a, 10)
        .await
        .expect("Node A: no neighbor");
    let (receiver_b, _) = wait_for_neighbor(receiver_b, 10)
        .await
        .expect("Node B: no neighbor");
    let (receiver_c, _) = wait_for_neighbor(receiver_c, 10)
        .await
        .expect("Node C: no neighbor");

    // Small additional delay for mesh to fully form
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Node A broadcasts "from A"
    let msg_from_a = b"message from A";
    sender_a
        .broadcast(Bytes::from_static(msg_from_a))
        .await
        .expect("Failed to send from A");

    // Both B and C should receive the message (concurrently)
    let receive_b = wait_for_message(receiver_b, 5);
    let receive_c = wait_for_message(receiver_c, 5);

    let (result_b, result_c) = tokio::join!(receive_b, receive_c);

    let (msg_b, _receiver_b) = result_b.expect("Node B didn't receive message");
    let (msg_c, receiver_c) = result_c.expect("Node C didn't receive message");

    assert_eq!(msg_b, msg_from_a.to_vec(), "Node B received wrong message");
    assert_eq!(msg_c, msg_from_a.to_vec(), "Node C received wrong message");

    // Node C broadcasts "from C"
    let msg_from_c = b"message from C";
    sender_c
        .broadcast(Bytes::from_static(msg_from_c))
        .await
        .expect("Failed to send from C");

    // Node A should receive the message from C
    let (msg_at_a, _) = wait_for_message(receiver_a, 5)
        .await
        .expect("Node A didn't receive message from C");

    assert_eq!(
        msg_at_a,
        msg_from_c.to_vec(),
        "Node A received wrong message from C"
    );

    // Clean shutdown
    drop(receiver_c);
    node_a.shutdown().await;
    node_b.shutdown().await;
    node_c.shutdown().await;
}

// ============================================================================
// Milestone 4 Tests - Late Joiner
// ============================================================================

/// Milestone 4: Node joining later can participate in the swarm
///
/// Tests that a node that joins after the initial setup can:
/// - Connect to the existing swarm
/// - Receive new messages sent after it joins
#[tokio::test]
async fn test_late_joiner() {
    let _ = tracing_subscriber::fmt::try_init();

    // Create initial nodes A and B
    let node_a = TestNode::new().await.expect("Failed to create node A");
    let node_b = TestNode::new().await.expect("Failed to create node B");

    // Share addresses between nodes
    node_b.add_peer(node_a.addr());
    node_a.add_peer(node_b.addr());

    let topic = random_topic();

    // A and B subscribe to the topic
    let topic_a = node_a
        .subscribe(topic, vec![])
        .await
        .expect("Node A failed to subscribe");
    let (sender_a, receiver_a) = topic_a.split();

    let topic_b = node_b
        .subscribe(topic, vec![node_a.id()])
        .await
        .expect("Node B failed to subscribe");
    let (_sender_b, receiver_b) = topic_b.split();

    // Wait for A and B to connect
    let (receiver_a, _) = wait_for_neighbor(receiver_a, 10)
        .await
        .expect("Node A: no neighbor");
    let (_receiver_b, _) = wait_for_neighbor(receiver_b, 10)
        .await
        .expect("Node B: no neighbor");

    // Exchange some messages between A and B (establishing the swarm)
    sender_a
        .broadcast(Bytes::from_static(b"initial message"))
        .await
        .expect("Failed to send initial message");

    // Small delay for the message to propagate
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Now create node C (the late joiner)
    let node_c = TestNode::new().await.expect("Failed to create node C");
    println!("Late joiner C: {}", node_c.id());

    // Share addresses with the late joiner
    node_c.add_peer(node_a.addr());
    node_a.add_peer(node_c.addr());

    // C joins the topic, bootstrapping from A
    let topic_c = node_c
        .subscribe(topic, vec![node_a.id()])
        .await
        .expect("Node C failed to subscribe");
    let (_sender_c, receiver_c) = topic_c.split();

    // Wait for C to connect
    let (receiver_c, _) = wait_for_neighbor(receiver_c, 10)
        .await
        .expect("Late joiner C: no neighbor");

    // Node A sends a NEW message (after C joined)
    let new_message = b"message after C joined";
    sender_a
        .broadcast(Bytes::from_static(new_message))
        .await
        .expect("Failed to send new message");

    // Node C should receive the new message (proves C joined the swarm)
    let (received_at_c, _) = wait_for_message(receiver_c, 5)
        .await
        .expect("Late joiner C didn't receive message");

    assert_eq!(
        received_at_c,
        new_message.to_vec(),
        "Late joiner C received wrong message"
    );

    // Clean shutdown
    drop(receiver_a);
    node_a.shutdown().await;
    node_b.shutdown().await;
    node_c.shutdown().await;
}

/// Test that late joiner can also send messages
#[tokio::test]
async fn test_late_joiner_can_send() {
    let _ = tracing_subscriber::fmt::try_init();

    // Create initial node A
    let node_a = TestNode::new().await.expect("Failed to create node A");

    let topic = random_topic();

    // A subscribes to the topic
    let topic_a = node_a
        .subscribe(topic, vec![])
        .await
        .expect("Node A failed to subscribe");
    let (_sender_a, receiver_a) = topic_a.split();

    // Now create late joiner C
    let node_c = TestNode::new().await.expect("Failed to create node C");

    // Share addresses
    node_c.add_peer(node_a.addr());
    node_a.add_peer(node_c.addr());

    let topic_c = node_c
        .subscribe(topic, vec![node_a.id()])
        .await
        .expect("Node C failed to subscribe");
    let (sender_c, receiver_c) = topic_c.split();

    // Wait for both to have neighbors
    let (receiver_a, _) = wait_for_neighbor(receiver_a, 10)
        .await
        .expect("Node A: no neighbor");
    let (_receiver_c, _) = wait_for_neighbor(receiver_c, 10)
        .await
        .expect("Node C: no neighbor");

    // Late joiner C sends a message
    let msg_from_c = b"hello from late joiner C";
    sender_c
        .broadcast(Bytes::from_static(msg_from_c))
        .await
        .expect("Failed to send from C");

    // Node A should receive C's message
    let (received_at_a, _) = wait_for_message(receiver_a, 5)
        .await
        .expect("Node A didn't receive message from C");

    assert_eq!(
        received_at_a,
        msg_from_c.to_vec(),
        "Node A received wrong message from late joiner C"
    );

    // Clean shutdown
    node_a.shutdown().await;
    node_c.shutdown().await;
}

// ============================================================================
// Additional Integration Tests
// ============================================================================

/// Test multiple messages in sequence
#[tokio::test]
async fn test_multiple_messages_sequence() {
    let _ = tracing_subscriber::fmt::try_init();

    let node_a = TestNode::new().await.expect("Failed to create node A");
    let node_b = TestNode::new().await.expect("Failed to create node B");

    // Share addresses
    node_b.add_peer(node_a.addr());
    node_a.add_peer(node_b.addr());

    let topic = random_topic();

    let topic_a = node_a.subscribe(topic, vec![]).await.unwrap();
    let (sender_a, receiver_a) = topic_a.split();

    let topic_b = node_b.subscribe(topic, vec![node_a.id()]).await.unwrap();
    let (_sender_b, receiver_b) = topic_b.split();

    // Wait for neighbors
    let (_receiver_a, _) = wait_for_neighbor(receiver_a, 10)
        .await
        .expect("Node A: no neighbor");
    let (mut receiver_b, _) = wait_for_neighbor(receiver_b, 10)
        .await
        .expect("Node B: no neighbor");

    // Send multiple messages
    let messages: Vec<&[u8]> = vec![b"message 1", b"message 2", b"message 3"];

    for msg in &messages {
        sender_a
            .broadcast(Bytes::copy_from_slice(*msg))
            .await
            .expect("Failed to broadcast");
        // Small delay between messages
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Receive all messages
    let mut received = Vec::new();
    let receive_result = tokio::time::timeout(Duration::from_secs(10), async {
        while received.len() < messages.len() {
            match receiver_b.try_next().await {
                Ok(Some(GossipEvent::Received(msg))) => {
                    received.push(msg.content.to_vec());
                }
                Ok(Some(_)) => continue,
                Ok(None) => break,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    })
    .await;

    assert!(
        receive_result.is_ok() && receive_result.unwrap().is_ok(),
        "Timed out receiving all messages"
    );
    assert_eq!(
        received.len(),
        messages.len(),
        "Did not receive all messages"
    );

    // Verify all messages were received (order may vary in gossip)
    for msg in &messages {
        assert!(
            received.contains(&msg.to_vec()),
            "Missing message: {:?}",
            msg
        );
    }

    node_a.shutdown().await;
    node_b.shutdown().await;
}

/// Test that nodes on different topics stay isolated
#[tokio::test]
async fn test_topic_isolation() {
    let _ = tracing_subscriber::fmt::try_init();

    let node_a = TestNode::new().await.expect("Failed to create node A");
    let node_b = TestNode::new().await.expect("Failed to create node B");
    let node_c = TestNode::new().await.expect("Failed to create node C");

    // Share addresses (A <-> B, but C is on different topic)
    node_b.add_peer(node_a.addr());
    node_a.add_peer(node_b.addr());

    // Two different topics
    let topic_1 = random_topic();
    let topic_2 = random_topic();

    // A and B subscribe to topic_1
    let topic_a = node_a.subscribe(topic_1, vec![]).await.unwrap();
    let (sender_a, receiver_a) = topic_a.split();

    let topic_b = node_b.subscribe(topic_1, vec![node_a.id()]).await.unwrap();
    let (_sender_b, receiver_b) = topic_b.split();

    // C subscribes to topic_2 (different topic!)
    // Note: C won't connect to A because they're on different topics
    let topic_c = node_c.subscribe(topic_2, vec![]).await.unwrap();
    let (_sender_c, mut receiver_c) = topic_c.split();

    // Wait for A and B to connect on topic_1
    let (_receiver_a, _) = wait_for_neighbor(receiver_a, 10)
        .await
        .expect("Node A: no neighbor");
    let (receiver_b, _) = wait_for_neighbor(receiver_b, 10)
        .await
        .expect("Node B: no neighbor");

    // A sends on topic_1
    sender_a
        .broadcast(Bytes::from_static(b"message on topic 1"))
        .await
        .expect("Failed to broadcast");

    // B should receive it (same topic)
    let (received_at_b, _) = wait_for_message(receiver_b, 5)
        .await
        .expect("Node B should receive message");
    assert_eq!(received_at_b, b"message on topic 1".to_vec());

    // C should NOT receive it (different topic) - expect timeout
    let received_at_c = tokio::time::timeout(Duration::from_millis(500), async {
        loop {
            match receiver_c.try_next().await {
                Ok(Some(GossipEvent::Received(msg))) => {
                    return Some(msg.content.to_vec());
                }
                Ok(Some(_)) => continue,
                Ok(None) => return None,
                Err(_) => return None,
            }
        }
    })
    .await;

    // C should timeout (no message on different topic)
    assert!(
        received_at_c.is_err() || received_at_c.unwrap().is_none(),
        "C should NOT receive message from different topic"
    );

    node_a.shutdown().await;
    node_b.shutdown().await;
    node_c.shutdown().await;
}

// ============================================================================
// Milestone 2.0+ Tests - Automerge Document Sync
// ============================================================================
//
// These tests verify that Automerge documents can be synchronized via gossip.
// They depend on RealmDoc, SyncMessage, and WireMessage being implemented.
//
// TODO: Remove #[ignore] once the following are implemented in syncengine-core:
//   - RealmDoc (Automerge document wrapper)
//   - SyncMessage (sync protocol messages)
//   - WireMessage (signed/versioned wire format)
// ============================================================================

use syncengine_core::{RealmId, RealmInfo};

/// Helper module for Automerge sync test types
///
/// These are placeholder implementations that will be replaced by
/// actual implementations in syncengine-core.
mod automerge_sync_helpers {
    use automerge::{transaction::Transactable, ObjType, ReadDoc, ROOT};
    use serde::{Deserialize, Serialize};
    use syncengine_core::RealmId;

    /// Task stored in a RealmDoc
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RealmTask {
        pub id: String,
        pub title: String,
        pub completed: bool,
    }

    /// Placeholder RealmDoc - wraps an Automerge document
    ///
    /// TODO: Replace with actual implementation from syncengine-core
    pub struct RealmDoc {
        doc: automerge::AutoCommit,
    }

    impl RealmDoc {
        /// Create a new empty document
        pub fn new() -> Self {
            Self {
                doc: automerge::AutoCommit::new(),
            }
        }

        /// Load document from saved bytes
        pub fn load(bytes: &[u8]) -> Result<Self, automerge::AutomergeError> {
            let doc = automerge::AutoCommit::load(bytes)?;
            Ok(Self { doc })
        }

        /// Save document to bytes
        pub fn save(&mut self) -> Vec<u8> {
            self.doc.save()
        }

        /// Add a task and return its ID
        pub fn add_task(&mut self, title: &str) -> Result<String, automerge::AutomergeError> {
            // Get or create tasks list
            let tasks_obj = match self.doc.get(ROOT, "tasks")? {
                Some((automerge::Value::Object(ObjType::List), id)) => id,
                _ => self.doc.put_object(ROOT, "tasks", ObjType::List)?,
            };

            // Create task object
            let task_id = ulid::Ulid::new().to_string();
            let task_idx = self.doc.length(&tasks_obj);
            let task_obj = self.doc.insert_object(&tasks_obj, task_idx, ObjType::Map)?;

            self.doc.put(&task_obj, "id", task_id.clone())?;
            self.doc.put(&task_obj, "title", title)?;
            self.doc.put(&task_obj, "completed", false)?;

            Ok(task_id)
        }

        /// List all tasks
        pub fn list_tasks(&self) -> Result<Vec<RealmTask>, automerge::AutomergeError> {
            let tasks_obj = match self.doc.get(ROOT, "tasks")? {
                Some((automerge::Value::Object(ObjType::List), id)) => id,
                _ => return Ok(vec![]),
            };

            let mut tasks = Vec::new();
            for i in 0..self.doc.length(&tasks_obj) {
                if let Some((automerge::Value::Object(ObjType::Map), task_obj)) =
                    self.doc.get(&tasks_obj, i)?
                {
                    let id = match self.doc.get(&task_obj, "id")? {
                        Some((automerge::Value::Scalar(s), _)) => {
                            s.into_owned().into_string().unwrap_or_default()
                        }
                        _ => continue,
                    };
                    let title = match self.doc.get(&task_obj, "title")? {
                        Some((automerge::Value::Scalar(s), _)) => {
                            s.into_owned().into_string().unwrap_or_default()
                        }
                        _ => continue,
                    };
                    let completed = match self.doc.get(&task_obj, "completed")? {
                        Some((automerge::Value::Scalar(s), _)) => s.to_bool().unwrap_or(false),
                        _ => false,
                    };

                    tasks.push(RealmTask {
                        id,
                        title,
                        completed,
                    });
                }
            }

            Ok(tasks)
        }

        /// Merge another document into this one
        pub fn merge(&mut self, other: &mut RealmDoc) -> Result<(), automerge::AutomergeError> {
            self.doc.merge(&mut other.doc)?;
            Ok(())
        }

        /// Generate sync message for incremental sync
        pub fn generate_sync_message(&mut self) -> Vec<u8> {
            // For simplicity, just save the full doc
            // A real implementation would use Automerge's sync protocol
            self.save()
        }

        /// Apply sync message for incremental sync
        pub fn apply_sync_message(&mut self, data: &[u8]) -> Result<(), automerge::AutomergeError> {
            // For simplicity, merge the full doc
            // A real implementation would use Automerge's sync protocol
            let mut other = Self::load(data)?;
            self.merge(&mut other)
        }
    }

    /// Sync protocol message types
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum SyncMessage {
        /// Full document sync response
        SyncResponse {
            realm_id: RealmId,
            document: Vec<u8>,
        },
        /// Incremental changes
        Changes { realm_id: RealmId, data: Vec<u8> },
    }

    /// Wire message wrapper with timestamp (for signing/versioning)
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WireMessage {
        pub timestamp: u64,
        pub message: SyncMessage,
    }

    impl WireMessage {
        /// Create a new wire message with current timestamp
        pub fn new(message: SyncMessage) -> Self {
            Self {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                message,
            }
        }

        /// Encode to bytes
        pub fn encode(&self) -> Result<Vec<u8>, postcard::Error> {
            postcard::to_stdvec(self)
        }

        /// Decode from bytes
        pub fn decode(bytes: &[u8]) -> Result<Self, postcard::Error> {
            postcard::from_bytes(bytes)
        }

        /// Consume and return inner message
        pub fn into_inner(self) -> SyncMessage {
            self.message
        }
    }
}

use automerge_sync_helpers::{RealmDoc, SyncMessage, WireMessage};

/// Helper to create a topic from a realm ID
fn realm_topic(realm_id: &RealmId) -> TopicId {
    TopicId::from_bytes(*realm_id.as_bytes())
}

/// Milestone 2.0: Two nodes sync an Automerge document via gossip
///
/// This test verifies that:
/// 1. Node A creates a document with a task
/// 2. Node A broadcasts the document via gossip
/// 3. Node B receives and loads the document
/// 4. Node B can read the task from the synced document
#[tokio::test]
async fn test_automerge_gossip_sync() {
    let _ = tracing_subscriber::fmt::try_init();

    // Create two nodes
    let node_a = TestNode::new().await.expect("Failed to create node A");
    let node_b = TestNode::new().await.expect("Failed to create node B");

    // Share addresses
    node_a.add_peer(node_b.addr());
    node_b.add_peer(node_a.addr());

    // Create a realm
    let realm_id = RealmId::new();
    let topic = realm_topic(&realm_id);

    // Subscribe to topic
    let topic_a = node_a
        .subscribe(topic, vec![])
        .await
        .expect("Node A failed to subscribe");
    let topic_b = node_b
        .subscribe(topic, vec![node_a.id()])
        .await
        .expect("Node B failed to subscribe");

    // Wait for neighbors
    let (sender_a, receiver_a) = topic_a.split();
    let (sender_b, receiver_b) = topic_b.split();

    let (receiver_a, _) = wait_for_neighbor(receiver_a, 10)
        .await
        .expect("Node A: no neighbor");
    let (mut receiver_b, _) = wait_for_neighbor(receiver_b, 10)
        .await
        .expect("Node B: no neighbor");

    // Node A creates document with task
    let mut doc_a = RealmDoc::new();
    let _task_id = doc_a
        .add_task("Sync this task")
        .expect("Failed to add task");

    // Node A broadcasts changes
    let changes = doc_a.save();
    let msg = WireMessage::new(SyncMessage::SyncResponse {
        realm_id: realm_id.clone(),
        document: changes,
    });
    sender_a
        .broadcast(Bytes::from(msg.encode().expect("Failed to encode")))
        .await
        .expect("Failed to broadcast");

    // Node B receives and applies
    let received = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match receiver_b.try_next().await {
                Ok(Some(GossipEvent::Received(msg))) => return Ok(msg),
                Ok(Some(_)) => continue,
                Ok(None) => return Err(anyhow::anyhow!("Stream ended")),
                Err(e) => return Err(anyhow::anyhow!("Error: {}", e)),
            }
        }
    })
    .await
    .expect("Timeout waiting for message")
    .expect("Failed to receive message");

    let wire_msg = WireMessage::decode(&received.content).expect("Failed to decode");

    match wire_msg.into_inner() {
        SyncMessage::SyncResponse { document, .. } => {
            let doc_b = RealmDoc::load(&document).expect("Failed to load document");
            let tasks = doc_b.list_tasks().expect("Failed to list tasks");
            assert_eq!(tasks.len(), 1, "Should have 1 task");
            assert_eq!(tasks[0].title, "Sync this task", "Task title mismatch");
        }
        _ => panic!("Expected SyncResponse"),
    }

    // Cleanup
    drop(sender_a);
    drop(sender_b);
    drop(receiver_a);
    drop(receiver_b);
    node_a.shutdown().await;
    node_b.shutdown().await;
}

/// Milestone 2.5: Concurrent edits merge correctly
///
/// This test verifies that:
/// 1. Node A creates a document and B receives it (common ancestor)
/// 2. Both nodes make concurrent edits (offline scenario)
/// 3. After re-syncing, both have all tasks merged via CRDT
/// 4. The merge is deterministic (same result on both sides)
#[tokio::test]
async fn test_concurrent_edits_sync() {
    let _ = tracing_subscriber::fmt::try_init();

    let node_a = TestNode::new().await.expect("Failed to create node A");
    let node_b = TestNode::new().await.expect("Failed to create node B");

    node_a.add_peer(node_b.addr());
    node_b.add_peer(node_a.addr());

    let realm_id = RealmId::new();
    let topic = realm_topic(&realm_id);

    let topic_a = node_a
        .subscribe(topic, vec![])
        .await
        .expect("Node A failed to subscribe");
    let topic_b = node_b
        .subscribe(topic, vec![node_a.id()])
        .await
        .expect("Node B failed to subscribe");

    let (sender_a, receiver_a) = topic_a.split();
    let (sender_b, receiver_b) = topic_b.split();

    let (mut receiver_a, _) = wait_for_neighbor(receiver_a, 10)
        .await
        .expect("Node A: no neighbor");
    let (mut receiver_b, _) = wait_for_neighbor(receiver_b, 10)
        .await
        .expect("Node B: no neighbor");

    // Step 1: Node A creates the document with an initial task
    let mut doc_a = RealmDoc::new();
    doc_a
        .add_task("Base task")
        .expect("Failed to add base task");

    // A sends initial document to B
    let initial_bytes = doc_a.save();
    let msg_init = WireMessage::new(SyncMessage::SyncResponse {
        realm_id: realm_id.clone(),
        document: initial_bytes.clone(),
    });
    sender_a
        .broadcast(Bytes::from(msg_init.encode().expect("encode")))
        .await
        .expect("broadcast initial");

    // B receives and loads (now B has a common ancestor with A)
    let recv_init = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match receiver_b.try_next().await {
                Ok(Some(GossipEvent::Received(msg))) => return Ok(msg),
                Ok(Some(_)) => continue,
                Ok(None) => return Err(anyhow::anyhow!("Stream ended")),
                Err(e) => return Err(anyhow::anyhow!("Error: {}", e)),
            }
        }
    })
    .await
    .expect("Timeout initial")
    .expect("Receive initial");

    let wire_init = WireMessage::decode(&recv_init.content).expect("decode initial");
    let mut doc_b = match wire_init.into_inner() {
        SyncMessage::SyncResponse { document, .. } => RealmDoc::load(&document).expect("load B"),
        _ => panic!("Expected SyncResponse"),
    };

    // Verify B has the base task
    assert_eq!(
        doc_b.list_tasks().expect("list B init").len(),
        1,
        "B should have base task"
    );

    // Step 2: Concurrent edits (simulating both offline, making changes)
    doc_a.add_task("Task from A").expect("Failed to add task A");
    doc_b.add_task("Task from B").expect("Failed to add task B");

    // Step 3: Exchange documents to merge concurrent changes
    let bytes_a = doc_a.save();
    let bytes_b = doc_b.save();

    // A sends to B
    let msg_a = WireMessage::new(SyncMessage::SyncResponse {
        realm_id: realm_id.clone(),
        document: bytes_a.clone(),
    });
    sender_a
        .broadcast(Bytes::from(msg_a.encode().expect("encode")))
        .await
        .expect("broadcast A");

    // B sends to A
    let msg_b = WireMessage::new(SyncMessage::SyncResponse {
        realm_id: realm_id.clone(),
        document: bytes_b.clone(),
    });
    sender_b
        .broadcast(Bytes::from(msg_b.encode().expect("encode")))
        .await
        .expect("broadcast B");

    // Give messages time to propagate
    tokio::time::sleep(Duration::from_millis(100)).await;

    // A receives B's document and merges
    let recv_a = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match receiver_a.try_next().await {
                Ok(Some(GossipEvent::Received(msg))) => return Ok(msg),
                Ok(Some(_)) => continue,
                Ok(None) => return Err(anyhow::anyhow!("Stream ended")),
                Err(e) => return Err(anyhow::anyhow!("Error: {}", e)),
            }
        }
    })
    .await
    .expect("Timeout A")
    .expect("Receive A");

    let wire_a = WireMessage::decode(&recv_a.content).expect("decode A");
    if let SyncMessage::SyncResponse { document, .. } = wire_a.into_inner() {
        let mut other = RealmDoc::load(&document).expect("load A");
        doc_a.merge(&mut other).expect("merge A");
    }

    // B receives A's document and merges
    let recv_b = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match receiver_b.try_next().await {
                Ok(Some(GossipEvent::Received(msg))) => return Ok(msg),
                Ok(Some(_)) => continue,
                Ok(None) => return Err(anyhow::anyhow!("Stream ended")),
                Err(e) => return Err(anyhow::anyhow!("Error: {}", e)),
            }
        }
    })
    .await
    .expect("Timeout B")
    .expect("Receive B");

    let wire_b = WireMessage::decode(&recv_b.content).expect("decode B");
    if let SyncMessage::SyncResponse { document, .. } = wire_b.into_inner() {
        let mut other = RealmDoc::load(&document).expect("load B");
        doc_b.merge(&mut other).expect("merge B");
    }

    // Step 4: Both should have all 3 tasks (base + concurrent edits)
    let tasks_a = doc_a.list_tasks().expect("list A");
    let tasks_b = doc_b.list_tasks().expect("list B");

    assert_eq!(tasks_a.len(), 3, "Doc A should have 3 tasks (base + A + B)");
    assert_eq!(tasks_b.len(), 3, "Doc B should have 3 tasks (base + A + B)");

    // Same content (CRDT determinism)
    let titles_a: HashSet<_> = tasks_a.iter().map(|t| &t.title).collect();
    let titles_b: HashSet<_> = tasks_b.iter().map(|t| &t.title).collect();
    assert_eq!(titles_a, titles_b, "Task titles should match");

    // Verify all expected tasks are present
    assert!(titles_a.contains(&"Base task".to_string()));
    assert!(titles_a.contains(&"Task from A".to_string()));
    assert!(titles_a.contains(&"Task from B".to_string()));

    // Cleanup
    drop(sender_a);
    drop(sender_b);
    drop(receiver_a);
    drop(receiver_b);
    node_a.shutdown().await;
    node_b.shutdown().await;
}

/// Test incremental sync (changes only)
///
/// This test verifies that:
/// 1. A creates an initial document and syncs to B
/// 2. A adds another task
/// 3. A sends only the incremental changes
/// 4. B applies the changes and has both tasks
#[tokio::test]
async fn test_incremental_sync() {
    let _ = tracing_subscriber::fmt::try_init();

    let node_a = TestNode::new().await.expect("Failed to create node A");
    let node_b = TestNode::new().await.expect("Failed to create node B");

    node_a.add_peer(node_b.addr());
    node_b.add_peer(node_a.addr());

    let realm_id = RealmId::new();
    let topic = realm_topic(&realm_id);

    let topic_a = node_a
        .subscribe(topic, vec![])
        .await
        .expect("Node A failed to subscribe");
    let topic_b = node_b
        .subscribe(topic, vec![node_a.id()])
        .await
        .expect("Node B failed to subscribe");

    let (sender_a, receiver_a) = topic_a.split();
    let (_sender_b, receiver_b) = topic_b.split();

    let (_receiver_a, _) = wait_for_neighbor(receiver_a, 10)
        .await
        .expect("Node A: no neighbor");
    let (mut receiver_b, _) = wait_for_neighbor(receiver_b, 10)
        .await
        .expect("Node B: no neighbor");

    // A creates initial document
    let mut doc_a = RealmDoc::new();
    doc_a
        .add_task("Initial task")
        .expect("Failed to add initial task");

    // Send full document first
    let full = doc_a.save();
    let msg = WireMessage::new(SyncMessage::SyncResponse {
        realm_id: realm_id.clone(),
        document: full,
    });
    sender_a
        .broadcast(Bytes::from(msg.encode().expect("encode")))
        .await
        .expect("broadcast");

    // B receives and loads
    let recv = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match receiver_b.try_next().await {
                Ok(Some(GossipEvent::Received(msg))) => return Ok(msg),
                Ok(Some(_)) => continue,
                Ok(None) => return Err(anyhow::anyhow!("Stream ended")),
                Err(e) => return Err(anyhow::anyhow!("Error: {}", e)),
            }
        }
    })
    .await
    .expect("Timeout")
    .expect("Receive");

    let wire = WireMessage::decode(&recv.content).expect("decode");
    let mut doc_b = match wire.into_inner() {
        SyncMessage::SyncResponse { document, .. } => RealmDoc::load(&document).expect("load"),
        _ => panic!("Expected SyncResponse"),
    };

    assert_eq!(
        doc_b.list_tasks().expect("list").len(),
        1,
        "Should have 1 task"
    );

    // A adds another task
    doc_a
        .add_task("Second task")
        .expect("Failed to add second task");

    // Send only incremental changes (in this implementation, still full doc)
    let changes = doc_a.generate_sync_message();
    let msg = WireMessage::new(SyncMessage::Changes {
        realm_id: realm_id.clone(),
        data: changes,
    });
    sender_a
        .broadcast(Bytes::from(msg.encode().expect("encode")))
        .await
        .expect("broadcast");

    // B receives and applies incremental
    let recv = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match receiver_b.try_next().await {
                Ok(Some(GossipEvent::Received(msg))) => return Ok(msg),
                Ok(Some(_)) => continue,
                Ok(None) => return Err(anyhow::anyhow!("Stream ended")),
                Err(e) => return Err(anyhow::anyhow!("Error: {}", e)),
            }
        }
    })
    .await
    .expect("Timeout")
    .expect("Receive");

    let wire = WireMessage::decode(&recv.content).expect("decode");
    match wire.into_inner() {
        SyncMessage::Changes { data, .. } => {
            doc_b.apply_sync_message(&data).expect("apply");
        }
        _ => panic!("Expected Changes"),
    }

    // B should now have both tasks
    assert_eq!(
        doc_b.list_tasks().expect("list").len(),
        2,
        "Should have 2 tasks"
    );

    node_a.shutdown().await;
    node_b.shutdown().await;
}

/// Test three-node document sync
///
/// This test verifies that:
/// 1. Three nodes join the same topic
/// 2. Node A creates a document with a task
/// 3. Nodes B and C both receive the document
#[tokio::test]
async fn test_three_node_document_sync() {
    let _ = tracing_subscriber::fmt::try_init();

    let node_a = TestNode::new().await.expect("Failed to create node A");
    let node_b = TestNode::new().await.expect("Failed to create node B");
    let node_c = TestNode::new().await.expect("Failed to create node C");

    // All nodes know about each other
    node_a.add_peer(node_b.addr());
    node_a.add_peer(node_c.addr());
    node_b.add_peer(node_a.addr());
    node_b.add_peer(node_c.addr());
    node_c.add_peer(node_a.addr());
    node_c.add_peer(node_b.addr());

    let realm_id = RealmId::new();
    let topic = realm_topic(&realm_id);

    let topic_a = node_a
        .subscribe(topic, vec![])
        .await
        .expect("Node A failed to subscribe");
    let topic_b = node_b
        .subscribe(topic, vec![node_a.id()])
        .await
        .expect("Node B failed to subscribe");
    let topic_c = node_c
        .subscribe(topic, vec![node_a.id()])
        .await
        .expect("Node C failed to subscribe");

    let (sender_a, receiver_a) = topic_a.split();
    let (_sender_b, receiver_b) = topic_b.split();
    let (_sender_c, receiver_c) = topic_c.split();

    let (_receiver_a, _) = wait_for_neighbor(receiver_a, 10)
        .await
        .expect("Node A: no neighbor");
    let (mut receiver_b, _) = wait_for_neighbor(receiver_b, 10)
        .await
        .expect("Node B: no neighbor");
    let (mut receiver_c, _) = wait_for_neighbor(receiver_c, 10)
        .await
        .expect("Node C: no neighbor");

    // Small delay for mesh to stabilize
    tokio::time::sleep(Duration::from_millis(100)).await;

    // A creates document
    let mut doc_a = RealmDoc::new();
    doc_a.add_task("From node A").expect("Failed to add task");

    let full = doc_a.save();
    let msg = WireMessage::new(SyncMessage::SyncResponse {
        realm_id: realm_id.clone(),
        document: full,
    });
    sender_a
        .broadcast(Bytes::from(msg.encode().expect("encode")))
        .await
        .expect("broadcast");

    // B and C should both receive
    let receive_b = async {
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                match receiver_b.try_next().await {
                    Ok(Some(GossipEvent::Received(msg))) => return Ok(msg),
                    Ok(Some(_)) => continue,
                    Ok(None) => return Err(anyhow::anyhow!("Stream ended")),
                    Err(e) => return Err(anyhow::anyhow!("Error: {}", e)),
                }
            }
        })
        .await
    };

    let receive_c = async {
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                match receiver_c.try_next().await {
                    Ok(Some(GossipEvent::Received(msg))) => return Ok(msg),
                    Ok(Some(_)) => continue,
                    Ok(None) => return Err(anyhow::anyhow!("Stream ended")),
                    Err(e) => return Err(anyhow::anyhow!("Error: {}", e)),
                }
            }
        })
        .await
    };

    let (result_b, result_c) = tokio::join!(receive_b, receive_c);

    let recv_b = result_b.expect("Timeout B").expect("Receive B");
    let recv_c = result_c.expect("Timeout C").expect("Receive C");

    let doc_b = match WireMessage::decode(&recv_b.content)
        .expect("decode B")
        .into_inner()
    {
        SyncMessage::SyncResponse { document, .. } => RealmDoc::load(&document).expect("load B"),
        _ => panic!("Expected SyncResponse"),
    };

    let doc_c = match WireMessage::decode(&recv_c.content)
        .expect("decode C")
        .into_inner()
    {
        SyncMessage::SyncResponse { document, .. } => RealmDoc::load(&document).expect("load C"),
        _ => panic!("Expected SyncResponse"),
    };

    assert_eq!(doc_b.list_tasks().expect("list B").len(), 1);
    assert_eq!(doc_c.list_tasks().expect("list C").len(), 1);

    node_a.shutdown().await;
    node_b.shutdown().await;
    node_c.shutdown().await;
}

// ============================================================================
// Milestone 5 Tests - Invite System Integration
// ============================================================================
//
// These tests verify the invite ticket system for realm sharing.
// Invite tickets encode:
// - Realm ID (gossip topic)
// - Encryption key
// - Bootstrap peers
// - Optional expiry
// ============================================================================

use syncengine_core::{InviteTicket, NodeAddrBytes, RealmCrypto};

/// Helper to convert EndpointAddr to NodeAddrBytes for invite tickets
fn endpoint_addr_to_node_addr_bytes(addr: &iroh::EndpointAddr) -> NodeAddrBytes {
    use iroh::TransportAddr;

    let direct_addrs: Vec<String> = addr
        .addrs
        .iter()
        .filter_map(|a| match a {
            TransportAddr::Ip(socket_addr) => Some(socket_addr.to_string()),
            TransportAddr::Relay(_) => None, // Handle relay separately
            _ => None,                       // Handle future variants
        })
        .collect();

    let relay_url: Option<String> = addr.addrs.iter().find_map(|a| match a {
        TransportAddr::Relay(url) => Some(url.to_string()),
        TransportAddr::Ip(_) => None,
        _ => None, // Handle future variants
    });

    let mut node_addr =
        NodeAddrBytes::new(addr.id.as_bytes().to_owned()).with_addresses(direct_addrs);

    if let Some(relay) = relay_url {
        node_addr = node_addr.with_relay(relay);
    }

    node_addr
}

/// Milestone 5.1: Test the complete invite-join flow
///
/// This test verifies that:
/// 1. Node A creates a realm and generates an invite
/// 2. Node B decodes the invite and joins the gossip topic using bootstrap peers
/// 3. Node B can receive messages from Node A on that topic
#[tokio::test]
async fn test_invite_join_flow() {
    let _ = tracing_subscriber::fmt::try_init();

    // Create two nodes
    let node_a = TestNode::new().await.expect("Failed to create node A");
    let node_b = TestNode::new().await.expect("Failed to create node B");

    println!("Node A ID: {}", node_a.id());
    println!("Node B ID: {}", node_b.id());

    // Node B needs to know how to reach Node A (via discovery)
    node_b.add_peer(node_a.addr());
    node_a.add_peer(node_b.addr());

    // Step 1: Node A creates a realm and generates an invite
    let realm_id = RealmId::new();
    let realm_key = RealmCrypto::generate_key();
    let bootstrap_peer = endpoint_addr_to_node_addr_bytes(&node_a.addr());

    let invite =
        InviteTicket::new(&realm_id, realm_key, vec![bootstrap_peer]).with_name("Test Realm");

    // Encode and verify it's valid
    let encoded = invite.encode().expect("Failed to encode invite");
    println!("Invite: {}", encoded);

    // Step 2: Node B decodes the invite
    let decoded = InviteTicket::decode(&encoded).expect("Failed to decode invite");
    assert_eq!(decoded.realm_id(), realm_id);
    assert_eq!(decoded.realm_key, realm_key);
    assert_eq!(decoded.realm_name, Some("Test Realm".to_string()));
    assert_eq!(decoded.bootstrap_peers.len(), 1);

    // Get topic and bootstrap info from invite
    let topic = decoded.topic_id();

    // Extract node ID from bootstrap peer
    let bootstrap_node_id = iroh::PublicKey::from_bytes(&decoded.bootstrap_peers[0].node_id)
        .expect("Invalid node ID in invite");

    // Step 3: Both nodes subscribe to the topic
    // Node A subscribes first (as the realm creator)
    let topic_a = node_a
        .subscribe(topic, vec![])
        .await
        .expect("Node A failed to subscribe");
    let (sender_a, receiver_a) = topic_a.split();

    // Node B joins using bootstrap peer from invite
    let topic_b = node_b
        .subscribe(topic, vec![bootstrap_node_id])
        .await
        .expect("Node B failed to subscribe");
    let (_sender_b, receiver_b) = topic_b.split();

    // Wait for neighbors
    let (receiver_a, _) = wait_for_neighbor(receiver_a, 10)
        .await
        .expect("Node A: no neighbor");
    let (receiver_b, _) = wait_for_neighbor(receiver_b, 10)
        .await
        .expect("Node B: no neighbor");

    // Step 4: Node A broadcasts a message
    let test_message = b"Welcome to the realm!";
    sender_a
        .broadcast(Bytes::from_static(test_message))
        .await
        .expect("Failed to broadcast message");

    // Step 5: Node B should receive the message
    let (received, _) = wait_for_message(receiver_b, 5)
        .await
        .expect("Node B didn't receive message");

    assert_eq!(received, test_message.to_vec(), "Message content mismatch");

    // Cleanup
    drop(receiver_a);
    node_a.shutdown().await;
    node_b.shutdown().await;
}

/// Milestone 5.2: Test invite with bootstrap peer encoding/decoding
///
/// This test verifies that:
/// 1. Create invite with Node A as bootstrap peer
/// 2. Verify the invite encodes/decodes the NodeAddrBytes correctly
/// 3. Node B should be able to connect using the bootstrap info
#[tokio::test]
async fn test_invite_with_bootstrap_peer() {
    let _ = tracing_subscriber::fmt::try_init();

    // Create a node to use as bootstrap peer
    let node_a = TestNode::new().await.expect("Failed to create node A");

    println!("Node A ID: {}", node_a.id());
    println!("Node A Addr: {:?}", node_a.addr());

    // Create invite with node A as bootstrap
    let realm_id = RealmId::new();
    let realm_key = RealmCrypto::generate_key();

    // Convert EndpointAddr to NodeAddrBytes using the helper
    let bootstrap_peer = endpoint_addr_to_node_addr_bytes(&node_a.addr());

    // Create and encode invite
    let invite = InviteTicket::new(&realm_id, realm_key, vec![bootstrap_peer.clone()])
        .with_name("Bootstrap Test Realm");

    let encoded = invite.encode().expect("Failed to encode invite");
    println!("Encoded invite length: {}", encoded.len());

    // Decode and verify
    let decoded = InviteTicket::decode(&encoded).expect("Failed to decode invite");

    // Verify bootstrap peer data survived roundtrip
    assert_eq!(decoded.bootstrap_peers.len(), 1);
    let decoded_peer = &decoded.bootstrap_peers[0];
    assert_eq!(decoded_peer.node_id, bootstrap_peer.node_id);
    assert_eq!(
        decoded_peer.direct_addresses,
        bootstrap_peer.direct_addresses
    );

    // Verify the node ID matches the original
    let decoded_node_id =
        iroh::PublicKey::from_bytes(&decoded_peer.node_id).expect("Invalid node ID");
    assert_eq!(decoded_node_id, node_a.id());

    // Verify we can use the decoded topic to subscribe
    let topic = decoded.topic_id();
    let _topic_a = node_a
        .subscribe(topic, vec![])
        .await
        .expect("Failed to subscribe with decoded topic");

    node_a.shutdown().await;
}

/// Milestone 5.3: Test invite expiry functionality
///
/// This test verifies that:
/// 1. Create an invite with expiry in the past
/// 2. Verify `is_expired()` returns true
/// 3. Create invite with future expiry, verify not expired
#[tokio::test]
async fn test_invite_expiry() {
    let _ = tracing_subscriber::fmt::try_init();

    let realm_id = RealmId::new();
    let realm_key = RealmCrypto::generate_key();

    // Test 1: Expired invite (expiry in the past)
    let past_time = chrono::Utc::now().timestamp() - 3600; // 1 hour ago
    let expired_invite = InviteTicket::new(&realm_id, realm_key, vec![]).with_expiry(past_time);

    assert!(
        expired_invite.is_expired(),
        "Invite with past expiry should be expired"
    );
    println!(
        "Expired invite (past): is_expired={}",
        expired_invite.is_expired()
    );

    // Encode/decode and verify expiry persists
    let encoded = expired_invite
        .encode()
        .expect("Failed to encode expired invite");
    let decoded = InviteTicket::decode(&encoded).expect("Failed to decode expired invite");
    assert!(
        decoded.is_expired(),
        "Decoded invite should still be expired"
    );
    assert_eq!(decoded.expires_at, Some(past_time));

    // Test 2: Valid invite (expiry in the future)
    let future_time = chrono::Utc::now().timestamp() + 3600; // 1 hour from now
    let valid_invite = InviteTicket::new(&realm_id, realm_key, vec![]).with_expiry(future_time);

    assert!(
        !valid_invite.is_expired(),
        "Invite with future expiry should not be expired"
    );
    println!(
        "Valid invite (future): is_expired={}",
        valid_invite.is_expired()
    );

    // Encode/decode and verify
    let encoded = valid_invite
        .encode()
        .expect("Failed to encode valid invite");
    let decoded = InviteTicket::decode(&encoded).expect("Failed to decode valid invite");
    assert!(
        !decoded.is_expired(),
        "Decoded invite should not be expired"
    );
    assert_eq!(decoded.expires_at, Some(future_time));

    // Test 3: Invite without expiry (never expires)
    let no_expiry_invite = InviteTicket::new(&realm_id, realm_key, vec![]);

    assert!(
        !no_expiry_invite.is_expired(),
        "Invite without expiry should not be expired"
    );
    assert!(no_expiry_invite.expires_at.is_none());
    println!(
        "No expiry invite: is_expired={}",
        no_expiry_invite.is_expired()
    );

    // Encode/decode and verify
    let encoded = no_expiry_invite
        .encode()
        .expect("Failed to encode no-expiry invite");
    let decoded = InviteTicket::decode(&encoded).expect("Failed to decode no-expiry invite");
    assert!(!decoded.is_expired());
    assert!(decoded.expires_at.is_none());
}

/// Milestone 5.4: Test invite realm key used for encryption
///
/// This test verifies that:
/// 1. Generate invite with realm key
/// 2. Verify the key can be used for encryption/decryption
/// 3. Use RealmCrypto to encrypt a message with the invite's realm_key
/// 4. Verify decryption works
#[tokio::test]
async fn test_invite_realm_key_used() {
    let _ = tracing_subscriber::fmt::try_init();

    let realm_id = RealmId::new();
    let realm_key = RealmCrypto::generate_key();

    // Create invite with the realm key
    let invite = InviteTicket::new(&realm_id, realm_key, vec![]).with_name("Encrypted Realm");

    // Encode and decode
    let encoded = invite.encode().expect("Failed to encode invite");
    let decoded = InviteTicket::decode(&encoded).expect("Failed to decode invite");

    // Verify the key survived the roundtrip
    assert_eq!(decoded.realm_key, realm_key);
    println!("Realm key preserved: {} bytes", decoded.realm_key.len());

    // Use the key from the decoded invite for encryption
    let crypto = RealmCrypto::new(&decoded.realm_key);

    // Encrypt a test message
    let plaintext = b"Secret realm task: Build solar dehydrator";
    let ciphertext = crypto.encrypt(plaintext).expect("Failed to encrypt");
    println!(
        "Encrypted: {} bytes -> {} bytes",
        plaintext.len(),
        ciphertext.len()
    );

    // Verify ciphertext is different from plaintext (and includes nonce + tag)
    assert_ne!(&ciphertext[..], plaintext);
    assert!(ciphertext.len() > plaintext.len());

    // Decrypt using the same key
    let decrypted = crypto.decrypt(&ciphertext).expect("Failed to decrypt");
    assert_eq!(decrypted, plaintext.to_vec());
    println!(
        "Decryption successful: {:?}",
        String::from_utf8_lossy(&decrypted)
    );

    // Verify wrong key fails decryption
    let wrong_key = RealmCrypto::generate_key();
    let wrong_crypto = RealmCrypto::new(&wrong_key);
    let result = wrong_crypto.decrypt(&ciphertext);
    assert!(result.is_err(), "Decryption with wrong key should fail");
    println!("Wrong key correctly rejected");
}

/// Test invite with multiple bootstrap peers
///
/// Verifies that invites can include multiple bootstrap peers and
/// all peer data survives encoding/decoding.
#[tokio::test]
async fn test_invite_multiple_bootstrap_peers() {
    let _ = tracing_subscriber::fmt::try_init();

    // Create multiple test nodes
    let node_a = TestNode::new().await.expect("Failed to create node A");
    let node_b = TestNode::new().await.expect("Failed to create node B");
    let node_c = TestNode::new().await.expect("Failed to create node C");

    // Create invite with all three as bootstrap peers
    let realm_id = RealmId::new();
    let realm_key = RealmCrypto::generate_key();

    let peer_a = endpoint_addr_to_node_addr_bytes(&node_a.addr());
    let peer_b = endpoint_addr_to_node_addr_bytes(&node_b.addr());
    let peer_c = endpoint_addr_to_node_addr_bytes(&node_c.addr());

    let invite = InviteTicket::new(
        &realm_id,
        realm_key,
        vec![peer_a.clone(), peer_b.clone(), peer_c.clone()],
    )
    .with_name("Multi-peer Realm");

    // Encode and decode
    let encoded = invite.encode().expect("Failed to encode invite");
    let decoded = InviteTicket::decode(&encoded).expect("Failed to decode invite");

    // Verify all peers are present
    assert_eq!(decoded.bootstrap_peers.len(), 3);

    // Verify each peer's node ID
    let decoded_ids: HashSet<[u8; 32]> =
        decoded.bootstrap_peers.iter().map(|p| p.node_id).collect();

    assert!(decoded_ids.contains(&peer_a.node_id));
    assert!(decoded_ids.contains(&peer_b.node_id));
    assert!(decoded_ids.contains(&peer_c.node_id));

    println!(
        "Successfully preserved {} bootstrap peers",
        decoded.bootstrap_peers.len()
    );

    // Cleanup
    node_a.shutdown().await;
    node_b.shutdown().await;
    node_c.shutdown().await;
}

// ============================================================================
// Milestone 6 Tests - Persistence Integration
// ============================================================================
//
// These tests verify that realm data, tasks, and encryption keys persist
// correctly across engine restarts and that nodes can rejoin gossip topics
// after restarting with saved state.
//
// NOTE: These tests use the Storage layer directly until SyncEngine is fully
// implemented. The tests are structured to be easily converted once
// SyncEngine supports persistence.
// ============================================================================

use syncengine_core::Storage;

/// Helper to create a temporary storage path
fn temp_storage_path() -> (tempfile::TempDir, std::path::PathBuf) {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("syncengine.redb");
    (temp_dir, db_path)
}

/// Milestone 6.1: Test basic persistence across restart
///
/// This test verifies that:
/// 1. Create a realm with tasks and save to storage
/// 2. Close the storage (simulating engine shutdown)
/// 3. Reopen storage with same path
/// 4. Verify realm and associated data are restored
#[tokio::test]
async fn test_persistence_restart() {
    let _ = tracing_subscriber::fmt::try_init();

    // Create temporary storage directory
    let (_temp_dir, db_path) = temp_storage_path();

    // Create realm ID and info to track across restart
    let realm_id: RealmId;
    let realm_name = "Persistent Realm";

    // Phase 1: Create storage, add realm and document, save
    {
        let storage = Storage::new(&db_path).expect("Failed to create storage");

        // Create a realm
        let realm_info = RealmInfo::new(realm_name);
        realm_id = realm_info.id.clone();

        // Save realm
        storage.save_realm(&realm_info).expect("Failed to save realm");

        // Create an Automerge document with tasks
        let mut doc = RealmDoc::new();
        doc.add_task("Task 1 - persist me").expect("Failed to add task");
        doc.add_task("Task 2 - also persist me").expect("Failed to add task");

        // Save document
        let doc_bytes = doc.save();
        storage
            .save_document(&realm_id, &doc_bytes)
            .expect("Failed to save document");

        // Save a realm encryption key
        let realm_key = RealmCrypto::generate_key();
        storage
            .save_realm_key(&realm_id, &realm_key)
            .expect("Failed to save realm key");

        // Verify data is in storage before "shutdown"
        assert!(storage.load_realm(&realm_id).unwrap().is_some());
        assert!(storage.load_document(&realm_id).unwrap().is_some());
        assert!(storage.load_realm_key(&realm_id).unwrap().is_some());

        // Storage is dropped here (simulating shutdown)
    }

    // Phase 2: Reopen storage and verify data persisted
    {
        let storage = Storage::new(&db_path).expect("Failed to reopen storage");

        // Load realm
        let loaded_realm = storage
            .load_realm(&realm_id)
            .expect("Failed to load realm")
            .expect("Realm should exist after restart");

        assert_eq!(loaded_realm.name, realm_name);
        assert_eq!(loaded_realm.id, realm_id);

        // Load document
        let doc_bytes = storage
            .load_document(&realm_id)
            .expect("Failed to load document")
            .expect("Document should exist after restart");

        let doc = RealmDoc::load(&doc_bytes).expect("Failed to load automerge doc");
        let tasks = doc.list_tasks().expect("Failed to list tasks");

        assert_eq!(tasks.len(), 2, "Should have 2 tasks after restart");
        assert!(
            tasks.iter().any(|t| t.title == "Task 1 - persist me"),
            "Task 1 should be present"
        );
        assert!(
            tasks.iter().any(|t| t.title == "Task 2 - also persist me"),
            "Task 2 should be present"
        );

        // Load realm key
        let loaded_key = storage
            .load_realm_key(&realm_id)
            .expect("Failed to load realm key")
            .expect("Realm key should exist after restart");

        assert_eq!(loaded_key.len(), 32, "Realm key should be 32 bytes");
    }

    println!("test_persistence_restart: PASSED");
}

/// Milestone 6.2: Test persistence with gossip rejoin
///
/// This test verifies that:
/// 1. Node A creates realm, adds tasks, generates invite
/// 2. Node B joins via invite, receives tasks
/// 3. Node B shuts down (drop engine)
/// 4. Node B restarts with same data dir
/// 5. Node B rejoins gossip using saved realm state
/// 6. Node A adds new task
/// 7. Node B receives the new task
#[tokio::test]
async fn test_persistence_rejoin_gossip() {
    let _ = tracing_subscriber::fmt::try_init();

    // Create temporary storage for both nodes
    let (_temp_dir_a, db_path_a) = temp_storage_path();
    let (_temp_dir_b, db_path_b) = temp_storage_path();

    // Create nodes
    let node_a = TestNode::new().await.expect("Failed to create node A");
    let node_b = TestNode::new().await.expect("Failed to create node B");

    // Store Node B's secret key for recreating with same identity after restart
    // (In production, this would be persisted in storage)

    // Share addresses
    node_a.add_peer(node_b.addr());
    node_b.add_peer(node_a.addr());

    // Create realm and topic
    let realm_id = RealmId::new();
    let realm_key = RealmCrypto::generate_key();
    let topic = realm_topic(&realm_id);

    // Store realm info in both nodes' storage
    let storage_a = Storage::new(&db_path_a).expect("Failed to create storage A");
    let storage_b = Storage::new(&db_path_b).expect("Failed to create storage B");

    let realm_info_a = RealmInfo::new("Shared Realm");
    let realm_info_b = RealmInfo {
        id: realm_id.clone(),
        name: "Shared Realm".to_string(),
        is_shared: true,
        created_at: chrono::Utc::now().timestamp(),
    };

    storage_a.save_realm(&realm_info_a).unwrap();
    storage_b.save_realm(&realm_info_b).unwrap();
    storage_a.save_realm_key(&realm_id, &realm_key).unwrap();
    storage_b.save_realm_key(&realm_id, &realm_key).unwrap();

    // Both nodes subscribe to the topic
    let topic_a = node_a
        .subscribe(topic, vec![])
        .await
        .expect("Node A failed to subscribe");
    let (sender_a, receiver_a) = topic_a.split();

    let topic_b = node_b
        .subscribe(topic, vec![node_a.id()])
        .await
        .expect("Node B failed to subscribe");
    let (_sender_b, receiver_b) = topic_b.split();

    // Wait for neighbors
    let (receiver_a, _) = wait_for_neighbor(receiver_a, 10)
        .await
        .expect("Node A: no neighbor");
    let (mut receiver_b, _) = wait_for_neighbor(receiver_b, 10)
        .await
        .expect("Node B: no neighbor");

    // Node A creates document and broadcasts
    let mut doc_a = RealmDoc::new();
    doc_a.add_task("Initial task").expect("Failed to add task");

    let msg = WireMessage::new(SyncMessage::SyncResponse {
        realm_id: realm_id.clone(),
        document: doc_a.save(),
    });
    sender_a
        .broadcast(Bytes::from(msg.encode().expect("encode")))
        .await
        .expect("broadcast");

    // Node B receives initial document
    let received = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match receiver_b.try_next().await {
                Ok(Some(GossipEvent::Received(msg))) => return Ok(msg),
                Ok(Some(_)) => continue,
                Ok(None) => return Err(anyhow::anyhow!("Stream ended")),
                Err(e) => return Err(anyhow::anyhow!("Error: {}", e)),
            }
        }
    })
    .await
    .expect("Timeout")
    .expect("Receive");

    // Node B saves the document to storage
    let wire = WireMessage::decode(&received.content).expect("decode");
    if let SyncMessage::SyncResponse { document, .. } = wire.into_inner() {
        storage_b
            .save_document(&realm_id, &document)
            .expect("Failed to save document B");
    }

    // Verify B has the document
    let doc_bytes_b = storage_b
        .load_document(&realm_id)
        .unwrap()
        .expect("B should have document");
    let doc_b = RealmDoc::load(&doc_bytes_b).expect("load");
    assert_eq!(
        doc_b.list_tasks().unwrap().len(),
        1,
        "B should have 1 task"
    );

    // "Shutdown" Node B (drop everything including storage)
    drop(receiver_b);
    drop(storage_b); // Must drop storage before reopening
    node_b.shutdown().await;

    // Small delay to simulate time passing
    tokio::time::sleep(Duration::from_millis(200)).await;

    // "Restart" Node B - create new node (in real impl, would use saved secret key)
    let node_b_restarted = TestNode::new().await.expect("Failed to restart node B");

    // Re-share addresses
    node_a.add_peer(node_b_restarted.addr());
    node_b_restarted.add_peer(node_a.addr());

    // Load realm info from storage to know what topics to rejoin
    let storage_b_reopened = Storage::new(&db_path_b).expect("Failed to reopen storage B");
    let _loaded_realm = storage_b_reopened
        .load_realm(&realm_id)
        .expect("Failed to load realm")
        .expect("Realm should exist");

    // Rejoin gossip topic using saved realm info
    let topic_b_rejoined = node_b_restarted
        .subscribe(topic, vec![node_a.id()])
        .await
        .expect("Node B failed to rejoin topic");
    let (_sender_b_rejoined, receiver_b_rejoined) = topic_b_rejoined.split();

    // Wait for neighbor
    let (mut receiver_b_rejoined, _) = wait_for_neighbor(receiver_b_rejoined, 10)
        .await
        .expect("Node B rejoined: no neighbor");

    // Node A adds a new task
    doc_a.add_task("New task after B rejoined").unwrap();

    let msg = WireMessage::new(SyncMessage::SyncResponse {
        realm_id: realm_id.clone(),
        document: doc_a.save(),
    });
    sender_a
        .broadcast(Bytes::from(msg.encode().expect("encode")))
        .await
        .expect("broadcast new task");

    // Node B (restarted) should receive the new task
    let received = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match receiver_b_rejoined.try_next().await {
                Ok(Some(GossipEvent::Received(msg))) => return Ok(msg),
                Ok(Some(_)) => continue,
                Ok(None) => return Err(anyhow::anyhow!("Stream ended")),
                Err(e) => return Err(anyhow::anyhow!("Error: {}", e)),
            }
        }
    })
    .await
    .expect("Timeout receiving new task")
    .expect("Receive new task");

    let wire = WireMessage::decode(&received.content).expect("decode");
    if let SyncMessage::SyncResponse { document, .. } = wire.into_inner() {
        let doc_b_new = RealmDoc::load(&document).expect("load new doc");
        let tasks = doc_b_new.list_tasks().expect("list tasks");
        assert_eq!(tasks.len(), 2, "B should now have 2 tasks");
        assert!(
            tasks
                .iter()
                .any(|t| t.title == "New task after B rejoined"),
            "New task should be present"
        );
    }

    // Cleanup
    drop(receiver_a);
    node_a.shutdown().await;
    node_b_restarted.shutdown().await;

    println!("test_persistence_rejoin_gossip: PASSED");
}

/// Milestone 6.3: Test persistence of realm encryption key
///
/// This test verifies that:
/// 1. Create realm with encryption key
/// 2. Encrypt a message with the key
/// 3. Save to storage and restart
/// 4. Load key from storage
/// 5. Verify decryption still works
#[tokio::test]
async fn test_persistence_realm_key() {
    let _ = tracing_subscriber::fmt::try_init();

    let (_temp_dir, db_path) = temp_storage_path();
    let realm_id = RealmId::new();
    let original_key: [u8; 32];
    let test_plaintext = b"Secret message for the realm";
    let test_ciphertext: Vec<u8>;

    // Phase 1: Create realm with key, encrypt message, save
    {
        let storage = Storage::new(&db_path).expect("Failed to create storage");

        // Create realm
        let realm_info = RealmInfo::new("Encrypted Realm");
        storage.save_realm(&realm_info).unwrap();

        // Generate and save encryption key
        original_key = RealmCrypto::generate_key();
        storage.save_realm_key(&realm_id, &original_key).unwrap();

        // Encrypt a test message
        let crypto = RealmCrypto::new(&original_key);
        test_ciphertext = crypto.encrypt(test_plaintext).expect("Failed to encrypt");

        // Verify encryption worked
        let decrypted = crypto.decrypt(&test_ciphertext).expect("Failed to decrypt");
        assert_eq!(decrypted, test_plaintext.to_vec());

        // Storage dropped here (shutdown)
    }

    // Phase 2: Reopen storage, load key, verify decryption works
    {
        let storage = Storage::new(&db_path).expect("Failed to reopen storage");

        // Load the realm key
        let loaded_key = storage
            .load_realm_key(&realm_id)
            .expect("Failed to load key")
            .expect("Key should exist");

        // Verify key is identical
        assert_eq!(loaded_key, original_key, "Key should survive restart");

        // Verify decryption still works with loaded key
        let crypto = RealmCrypto::new(&loaded_key);
        let decrypted = crypto
            .decrypt(&test_ciphertext)
            .expect("Decryption should work with loaded key");

        assert_eq!(
            decrypted,
            test_plaintext.to_vec(),
            "Decrypted message should match"
        );
    }

    println!("test_persistence_realm_key: PASSED");
}

/// Milestone 6.4: Test persistence with multiple realms
///
/// This test verifies that:
/// 1. Create 3 realms with tasks in each
/// 2. Save all to storage
/// 3. Shutdown and restart
/// 4. Verify all realms and their tasks are restored correctly
#[tokio::test]
async fn test_persistence_multiple_realms() {
    let _ = tracing_subscriber::fmt::try_init();

    let (_temp_dir, db_path) = temp_storage_path();

    // Track realm IDs across restart
    let mut realm_ids: Vec<RealmId> = Vec::new();
    let realm_names = ["Work Tasks", "Home Tasks", "Hobby Projects"];
    let task_counts = [3, 2, 4]; // Different number of tasks per realm

    // Phase 1: Create multiple realms with tasks
    {
        let storage = Storage::new(&db_path).expect("Failed to create storage");

        for (i, name) in realm_names.iter().enumerate() {
            // Create realm
            let realm_info = RealmInfo::new(*name);
            realm_ids.push(realm_info.id.clone());
            storage.save_realm(&realm_info).unwrap();

            // Create document with tasks
            let mut doc = RealmDoc::new();
            for j in 0..task_counts[i] {
                doc.add_task(&format!("{} - Task {}", name, j + 1))
                    .expect("Failed to add task");
            }

            // Save document
            storage
                .save_document(&realm_info.id, &doc.save())
                .expect("Failed to save document");

            // Generate and save realm key
            let key = RealmCrypto::generate_key();
            storage
                .save_realm_key(&realm_info.id, &key)
                .expect("Failed to save key");
        }

        // Verify all realms were created
        let realms = storage.list_realms().unwrap();
        assert_eq!(realms.len(), 3, "Should have 3 realms");
    }

    // Phase 2: Reopen and verify all data persisted
    {
        let storage = Storage::new(&db_path).expect("Failed to reopen storage");

        // List realms
        let loaded_realms = storage.list_realms().unwrap();
        assert_eq!(loaded_realms.len(), 3, "Should still have 3 realms");

        // Verify each realm
        for (i, expected_name) in realm_names.iter().enumerate() {
            let realm_id = &realm_ids[i];

            // Load and verify realm info
            let realm_info = storage
                .load_realm(realm_id)
                .expect("Failed to load realm")
                .expect("Realm should exist");

            assert_eq!(realm_info.name, *expected_name);

            // Load and verify document
            let doc_bytes = storage
                .load_document(realm_id)
                .expect("Failed to load document")
                .expect("Document should exist");

            let doc = RealmDoc::load(&doc_bytes).expect("Failed to load doc");
            let tasks = doc.list_tasks().expect("Failed to list tasks");

            assert_eq!(
                tasks.len(),
                task_counts[i],
                "Realm '{}' should have {} tasks",
                expected_name,
                task_counts[i]
            );

            // Verify task titles
            for j in 0..task_counts[i] {
                let expected_title = format!("{} - Task {}", expected_name, j + 1);
                assert!(
                    tasks.iter().any(|t| t.title == expected_title),
                    "Task '{}' should exist",
                    expected_title
                );
            }

            // Verify realm key exists
            let key = storage
                .load_realm_key(realm_id)
                .expect("Failed to load key")
                .expect("Key should exist");

            assert_eq!(key.len(), 32, "Key should be 32 bytes");
        }
    }

    println!("test_persistence_multiple_realms: PASSED");
}

/// Test that storage handles concurrent realm creation correctly
///
/// This tests the storage layer's ability to handle multiple realms
/// being created and accessed in quick succession.
#[tokio::test]
async fn test_persistence_concurrent_realms() {
    let _ = tracing_subscriber::fmt::try_init();

    let (_temp_dir, db_path) = temp_storage_path();
    let storage = Storage::new(&db_path).expect("Failed to create storage");

    // Create 10 realms concurrently
    let mut handles = Vec::new();
    let storage_arc = std::sync::Arc::new(storage);

    for i in 0..10 {
        let storage_clone = storage_arc.clone();
        let handle = tokio::spawn(async move {
            let realm_info = RealmInfo::new(format!("Concurrent Realm {}", i));
            let realm_id = realm_info.id.clone();

            storage_clone.save_realm(&realm_info).expect("Failed to save");

            // Create and save document
            let mut doc = RealmDoc::new();
            doc.add_task(&format!("Task for realm {}", i)).unwrap();
            storage_clone
                .save_document(&realm_id, &doc.save())
                .expect("Failed to save doc");

            realm_id
        });
        handles.push(handle);
    }

    // Wait for all to complete
    let realm_ids: Vec<RealmId> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.expect("Task panicked"))
        .collect();

    // Verify all realms exist
    let all_realms = storage_arc.list_realms().unwrap();
    assert_eq!(all_realms.len(), 10, "Should have 10 realms");

    // Verify each realm can be loaded
    for realm_id in realm_ids {
        let realm = storage_arc.load_realm(&realm_id).unwrap();
        assert!(realm.is_some(), "Realm should exist");

        let doc_bytes = storage_arc.load_document(&realm_id).unwrap();
        assert!(doc_bytes.is_some(), "Document should exist");
    }

    println!("test_persistence_concurrent_realms: PASSED");
}

/// Test that document updates are persisted correctly
///
/// This verifies that incremental updates to an Automerge document
/// are correctly persisted and can be restored.
#[tokio::test]
async fn test_persistence_document_updates() {
    let _ = tracing_subscriber::fmt::try_init();

    let (_temp_dir, db_path) = temp_storage_path();
    let realm_id = RealmId::new();

    // Phase 1: Create document, add tasks incrementally, save after each
    {
        let storage = Storage::new(&db_path).expect("Failed to create storage");

        let realm_info = RealmInfo::new("Incremental Updates");
        storage.save_realm(&realm_info).unwrap();

        let mut doc = RealmDoc::new();

        // Add first task and save
        doc.add_task("Task 1").unwrap();
        storage
            .save_document(&realm_id, &doc.save())
            .expect("Failed to save");

        // Add second task and save
        doc.add_task("Task 2").unwrap();
        storage
            .save_document(&realm_id, &doc.save())
            .expect("Failed to save");

        // Add third task and save
        doc.add_task("Task 3").unwrap();
        storage
            .save_document(&realm_id, &doc.save())
            .expect("Failed to save");
    }

    // Phase 2: Verify final state persisted
    {
        let storage = Storage::new(&db_path).expect("Failed to reopen storage");

        let doc_bytes = storage
            .load_document(&realm_id)
            .unwrap()
            .expect("Document should exist");

        let doc = RealmDoc::load(&doc_bytes).expect("Failed to load doc");
        let tasks = doc.list_tasks().expect("Failed to list tasks");

        assert_eq!(tasks.len(), 3, "Should have 3 tasks");
        assert!(tasks.iter().any(|t| t.title == "Task 1"));
        assert!(tasks.iter().any(|t| t.title == "Task 2"));
        assert!(tasks.iter().any(|t| t.title == "Task 3"));
    }

    println!("test_persistence_document_updates: PASSED");
}

// ============================================================================
// Milestone 7 Tests - Identity and Signed Message Integration
// ============================================================================
//
// These tests verify the identity system and signed message envelopes:
// - HybridKeypair generation and persistence
// - SyncEnvelope with signatures
// - Signature verification over gossip
// - Rejection of invalid/tampered messages
//
// NOTE: The identity module is being built by another agent. These tests use
// mock implementations that will be replaced once the real identity module
// is available. The test structure and verification logic remain the same.
// ============================================================================

/// Mock identity module for testing envelope logic
///
/// These placeholder implementations will be replaced by actual implementations
/// from the identity module once it's complete.
mod mock_identity {
    use serde::{Deserialize, Serialize};

    /// Decentralized Identifier (DID) for a user
    ///
    /// Format: did:sync:z{base58(hash(public_keys))}
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct Did(pub String);

    impl Did {
        /// Create a DID from a public key hash
        pub fn from_public_key_hash(hash: &[u8; 32]) -> Self {
            let encoded = bs58::encode(hash).into_string();
            Self(format!("did:sync:z{}", encoded))
        }

        /// Validate DID format
        pub fn is_valid(&self) -> bool {
            self.0.starts_with("did:sync:z")
                && self.0.len() > 10
                && bs58::decode(&self.0[10..]).into_vec().is_ok()
        }

        /// Extract the base58 portion of the DID
        pub fn base58_portion(&self) -> Option<&str> {
            if self.0.starts_with("did:sync:z") {
                Some(&self.0[10..])
            } else {
                None
            }
        }
    }

    impl std::fmt::Display for Did {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    /// Mock hybrid keypair (Ed25519 + ML-DSA-65 placeholder)
    ///
    /// In production, this would contain:
    /// - Ed25519 keypair (classical)
    /// - ML-DSA-65 keypair (post-quantum)
    ///
    /// For testing, we use a simplified mock based on blake3 hashing.
    #[derive(Clone)]
    pub struct MockHybridKeypair {
        /// Secret key material (32 bytes for mock)
        secret: [u8; 32],
        /// Public key hash (32 bytes)
        pub public_hash: [u8; 32],
        /// DID derived from public key
        pub did: Did,
    }

    impl MockHybridKeypair {
        /// Generate a new random keypair
        pub fn generate() -> Self {
            let mut secret = [0u8; 32];
            rand::RngCore::fill_bytes(&mut rand::rng(), &mut secret);
            Self::from_secret(secret)
        }

        /// Create keypair from secret key material
        pub fn from_secret(secret: [u8; 32]) -> Self {
            // Derive public key hash from secret (mock derivation)
            let public_hash = *blake3::hash(&secret).as_bytes();
            let did = Did::from_public_key_hash(&public_hash);
            Self {
                secret,
                public_hash,
                did,
            }
        }

        /// Get the DID for this keypair
        pub fn did(&self) -> &Did {
            &self.did
        }

        /// Sign data with this keypair
        ///
        /// Mock signature: blake3(secret || data)
        pub fn sign(&self, data: &[u8]) -> Vec<u8> {
            let mut hasher = blake3::Hasher::new();
            hasher.update(&self.secret);
            hasher.update(data);
            hasher.finalize().as_bytes().to_vec()
        }

        /// Verify a signature against this keypair's public identity
        ///
        /// For mock purposes, we need access to the secret to verify.
        /// In production, verification uses only public keys.
        pub fn verify(&self, data: &[u8], signature: &[u8]) -> bool {
            let expected = self.sign(data);
            expected == signature
        }

        /// Export secret key for persistence
        pub fn export_secret(&self) -> [u8; 32] {
            self.secret
        }
    }

    /// Verify a signature given a DID and public hash
    ///
    /// This simulates looking up the public key from the DID and verifying.
    /// In production, this would use the actual public keys.
    ///
    /// For testing, we pass the expected signer to verify against.
    pub fn verify_signature_with_keypair(
        keypair: &MockHybridKeypair,
        data: &[u8],
        signature: &[u8],
    ) -> bool {
        keypair.verify(data, signature)
    }

    /// SyncEnvelope wraps a message with sender identity and signature
    ///
    /// This is the wire format for signed messages over gossip.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SyncEnvelope {
        /// DID of the sender
        pub sender_did: Did,
        /// Public key hash (for verification without DID resolution)
        pub sender_public_hash: [u8; 32],
        /// Timestamp of message creation (Unix millis)
        pub timestamp: u64,
        /// Message nonce for replay protection
        pub nonce: [u8; 16],
        /// Encrypted payload (ciphertext)
        pub payload: Vec<u8>,
        /// Signature over (sender_did || timestamp || nonce || payload)
        pub signature: Vec<u8>,
    }

    impl SyncEnvelope {
        /// Create a new envelope with the given keypair and payload
        pub fn new(keypair: &MockHybridKeypair, payload: Vec<u8>) -> Self {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            let mut nonce = [0u8; 16];
            rand::RngCore::fill_bytes(&mut rand::rng(), &mut nonce);

            let mut envelope = Self {
                sender_did: keypair.did().clone(),
                sender_public_hash: keypair.public_hash,
                timestamp,
                nonce,
                payload,
                signature: vec![],
            };

            // Sign the envelope
            envelope.signature = keypair.sign(&envelope.signable_bytes());

            envelope
        }

        /// Get the bytes that are signed
        pub fn signable_bytes(&self) -> Vec<u8> {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(self.sender_did.0.as_bytes());
            bytes.extend_from_slice(&self.timestamp.to_le_bytes());
            bytes.extend_from_slice(&self.nonce);
            bytes.extend_from_slice(&self.payload);
            bytes
        }

        /// Verify the envelope's signature using the sender's keypair
        pub fn verify_with_keypair(&self, keypair: &MockHybridKeypair) -> bool {
            // Check DID matches
            if self.sender_did != keypair.did {
                return false;
            }
            // Check public hash matches
            if self.sender_public_hash != keypair.public_hash {
                return false;
            }
            // Verify signature
            verify_signature_with_keypair(keypair, &self.signable_bytes(), &self.signature)
        }

        /// Encode envelope to bytes
        pub fn encode(&self) -> Result<Vec<u8>, postcard::Error> {
            postcard::to_stdvec(self)
        }

        /// Decode envelope from bytes
        pub fn decode(bytes: &[u8]) -> Result<Self, postcard::Error> {
            postcard::from_bytes(bytes)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_did_format() {
            let keypair = MockHybridKeypair::generate();
            let did = keypair.did();
            assert!(did.is_valid());
            assert!(did.0.starts_with("did:sync:z"));
        }

        #[test]
        fn test_sign_verify() {
            let keypair = MockHybridKeypair::generate();
            let data = b"test message";
            let signature = keypair.sign(data);
            assert!(keypair.verify(data, &signature));
        }

        #[test]
        fn test_envelope_roundtrip() {
            let keypair = MockHybridKeypair::generate();
            let payload = b"encrypted payload".to_vec();
            let envelope = SyncEnvelope::new(&keypair, payload.clone());

            let encoded = envelope.encode().unwrap();
            let decoded = SyncEnvelope::decode(&encoded).unwrap();

            assert_eq!(decoded.payload, payload);
            assert!(decoded.verify_with_keypair(&keypair));
        }
    }
}

use mock_identity::{Did, MockHybridKeypair, SyncEnvelope};

// ============================================================================
// Milestone 7.1: Signed Message Over Gossip
// ============================================================================

/// Milestone 7.1: Test signed message exchange over gossip
///
/// This test verifies that:
/// 1. Node A generates identity (MockHybridKeypair)
/// 2. Node A creates SyncEnvelope with signed message
/// 3. Node A broadcasts envelope via gossip
/// 4. Node B receives and verifies signature
/// 5. Node B decrypts and processes message
#[tokio::test]
async fn test_signed_message_over_gossip() {
    let _ = tracing_subscriber::fmt::try_init();

    // Create two nodes
    let node_a = TestNode::new().await.expect("Failed to create node A");
    let node_b = TestNode::new().await.expect("Failed to create node B");

    println!("Node A ID: {}", node_a.id());
    println!("Node B ID: {}", node_b.id());

    // Share addresses
    node_a.add_peer(node_b.addr());
    node_b.add_peer(node_a.addr());

    // Generate identity for Node A
    let keypair_a = MockHybridKeypair::generate();
    println!("Node A DID: {}", keypair_a.did());

    // Create realm and topic
    let realm_id = RealmId::new();
    let realm_key = RealmCrypto::generate_key();
    let topic = realm_topic(&realm_id);

    // Both nodes subscribe
    let topic_a = node_a
        .subscribe(topic, vec![])
        .await
        .expect("Node A failed to subscribe");
    let (sender_a, receiver_a) = topic_a.split();

    let topic_b = node_b
        .subscribe(topic, vec![node_a.id()])
        .await
        .expect("Node B failed to subscribe");
    let (_sender_b, receiver_b) = topic_b.split();

    // Wait for neighbors
    let (_receiver_a, _) = wait_for_neighbor(receiver_a, 10)
        .await
        .expect("Node A: no neighbor");
    let (mut receiver_b, _) = wait_for_neighbor(receiver_b, 10)
        .await
        .expect("Node B: no neighbor");

    // Node A creates a signed envelope with encrypted payload
    let plaintext = b"Secret task: Build quantum-resistant communication system";
    let crypto = RealmCrypto::new(&realm_key);
    let ciphertext = crypto.encrypt(plaintext).expect("Failed to encrypt");

    let envelope = SyncEnvelope::new(&keypair_a, ciphertext.clone());
    println!("Envelope signature: {} bytes", envelope.signature.len());

    // Broadcast the signed envelope
    let envelope_bytes = envelope.encode().expect("Failed to encode envelope");
    sender_a
        .broadcast(Bytes::from(envelope_bytes))
        .await
        .expect("Failed to broadcast");

    // Node B receives and verifies
    let received = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match receiver_b.try_next().await {
                Ok(Some(GossipEvent::Received(msg))) => return Ok(msg),
                Ok(Some(_)) => continue,
                Ok(None) => return Err(anyhow::anyhow!("Stream ended")),
                Err(e) => return Err(anyhow::anyhow!("Error: {}", e)),
            }
        }
    })
    .await
    .expect("Timeout waiting for message")
    .expect("Failed to receive message");

    // Decode and verify the envelope
    let received_envelope =
        SyncEnvelope::decode(&received.content).expect("Failed to decode envelope");

    // Verify sender DID format
    assert!(
        received_envelope.sender_did.is_valid(),
        "Sender DID should be valid"
    );
    assert_eq!(
        received_envelope.sender_did,
        *keypair_a.did(),
        "Sender DID should match"
    );

    // Verify signature (in production, would lookup public key from DID)
    // For testing, we verify using the known keypair
    assert!(
        received_envelope.verify_with_keypair(&keypair_a),
        "Signature verification should pass"
    );

    // Decrypt the payload
    let decrypted = crypto
        .decrypt(&received_envelope.payload)
        .expect("Failed to decrypt");
    assert_eq!(
        decrypted, plaintext,
        "Decrypted payload should match original"
    );

    println!(
        "Successfully received, verified, and decrypted: {:?}",
        String::from_utf8_lossy(&decrypted)
    );

    // Cleanup
    node_a.shutdown().await;
    node_b.shutdown().await;
}

// ============================================================================
// Milestone 7.2: Identity Persistence
// ============================================================================

/// Milestone 7.2: Test identity persistence across restarts
///
/// This test verifies that:
/// 1. Generate identity and save to storage
/// 2. Restart and load identity
/// 3. Verify signatures still work with loaded identity
#[tokio::test]
async fn test_identity_persistence() {
    let _ = tracing_subscriber::fmt::try_init();

    let (_temp_dir, db_path) = temp_storage_path();

    // Store the original DID and a test signature for verification
    let original_did: Did;
    let test_data = b"Test data for signature verification";
    let test_signature: Vec<u8>;
    let exported_secret: [u8; 32];

    // Phase 1: Generate identity, sign data, save to storage
    {
        let storage = Storage::new(&db_path).expect("Failed to create storage");

        // Generate keypair
        let keypair = MockHybridKeypair::generate();
        original_did = keypair.did().clone();
        println!("Generated DID: {}", original_did);

        // Sign some test data
        test_signature = keypair.sign(test_data);
        println!("Generated signature: {} bytes", test_signature.len());

        // Export and save secret key
        // In production, this would be encrypted before storage
        exported_secret = keypair.export_secret();

        // Save identity to storage using the identity table
        // For now, we'll save it as a document (identity module will provide proper API)
        let realm_id = RealmId::from_bytes([0u8; 32]); // Special "identity" realm
        storage
            .save_document(&realm_id, &exported_secret)
            .expect("Failed to save identity");

        // Verify the signature works before "shutdown"
        assert!(
            keypair.verify(test_data, &test_signature),
            "Signature should verify before restart"
        );
    }

    // Phase 2: Reopen storage, load identity, verify signatures
    {
        let storage = Storage::new(&db_path).expect("Failed to reopen storage");

        // Load identity
        let realm_id = RealmId::from_bytes([0u8; 32]);
        let loaded_secret_vec = storage
            .load_document(&realm_id)
            .expect("Failed to load identity")
            .expect("Identity should exist");

        // Reconstruct keypair from loaded secret
        let mut loaded_secret = [0u8; 32];
        loaded_secret.copy_from_slice(&loaded_secret_vec);
        let loaded_keypair = MockHybridKeypair::from_secret(loaded_secret);

        // Verify DID matches
        assert_eq!(
            *loaded_keypair.did(),
            original_did,
            "Loaded DID should match original"
        );
        println!("Loaded DID: {}", loaded_keypair.did());

        // Verify the original signature still works
        assert!(
            loaded_keypair.verify(test_data, &test_signature),
            "Original signature should verify with loaded keypair"
        );

        // Create and verify a new signature
        let new_data = b"New data after restart";
        let new_signature = loaded_keypair.sign(new_data);
        assert!(
            loaded_keypair.verify(new_data, &new_signature),
            "New signature should verify"
        );

        println!("Identity persistence verified successfully");
    }

    println!("test_identity_persistence: PASSED");
}

// ============================================================================
// Milestone 7.3: Reject Invalid Signature
// ============================================================================

/// Milestone 7.3: Test rejection of messages with invalid signatures
///
/// This test verifies that:
/// 1. Node A sends message with valid signature
/// 2. Tamper with ciphertext after signing
/// 3. Node B should reject the message (signature won't verify)
#[tokio::test]
async fn test_reject_invalid_signature() {
    let _ = tracing_subscriber::fmt::try_init();

    // Create two nodes
    let node_a = TestNode::new().await.expect("Failed to create node A");
    let node_b = TestNode::new().await.expect("Failed to create node B");

    // Share addresses
    node_a.add_peer(node_b.addr());
    node_b.add_peer(node_a.addr());

    // Generate identity for Node A
    let keypair_a = MockHybridKeypair::generate();

    // Create realm and topic
    let realm_id = RealmId::new();
    let realm_key = RealmCrypto::generate_key();
    let topic = realm_topic(&realm_id);

    // Both nodes subscribe
    let topic_a = node_a
        .subscribe(topic, vec![])
        .await
        .expect("Node A failed to subscribe");
    let (sender_a, receiver_a) = topic_a.split();

    let topic_b = node_b
        .subscribe(topic, vec![node_a.id()])
        .await
        .expect("Node B failed to subscribe");
    let (_sender_b, receiver_b) = topic_b.split();

    // Wait for neighbors
    let (_receiver_a, _) = wait_for_neighbor(receiver_a, 10)
        .await
        .expect("Node A: no neighbor");
    let (mut receiver_b, _) = wait_for_neighbor(receiver_b, 10)
        .await
        .expect("Node B: no neighbor");

    // Node A creates a signed envelope
    let plaintext = b"Original message";
    let crypto = RealmCrypto::new(&realm_key);
    let ciphertext = crypto.encrypt(plaintext).expect("Failed to encrypt");

    let mut envelope = SyncEnvelope::new(&keypair_a, ciphertext);

    // Verify envelope is valid before tampering
    assert!(
        envelope.verify_with_keypair(&keypair_a),
        "Envelope should be valid before tampering"
    );

    // TAMPER: Modify the payload after signing
    if !envelope.payload.is_empty() {
        envelope.payload[0] ^= 0xFF; // Flip bits in first byte
    }

    // Verify envelope is now INVALID
    assert!(
        !envelope.verify_with_keypair(&keypair_a),
        "Tampered envelope should NOT verify"
    );

    // Broadcast the tampered envelope
    let envelope_bytes = envelope.encode().expect("Failed to encode envelope");
    sender_a
        .broadcast(Bytes::from(envelope_bytes))
        .await
        .expect("Failed to broadcast");

    // Node B receives and attempts to verify
    let received = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match receiver_b.try_next().await {
                Ok(Some(GossipEvent::Received(msg))) => return Ok(msg),
                Ok(Some(_)) => continue,
                Ok(None) => return Err(anyhow::anyhow!("Stream ended")),
                Err(e) => return Err(anyhow::anyhow!("Error: {}", e)),
            }
        }
    })
    .await
    .expect("Timeout waiting for message")
    .expect("Failed to receive message");

    // Decode the envelope
    let received_envelope =
        SyncEnvelope::decode(&received.content).expect("Failed to decode envelope");

    // Verification should FAIL
    let verification_result = received_envelope.verify_with_keypair(&keypair_a);
    assert!(
        !verification_result,
        "Tampered message should fail signature verification"
    );

    println!("Correctly rejected tampered message");

    // Also verify decryption would fail (tampered ciphertext)
    let decrypt_result = crypto.decrypt(&received_envelope.payload);
    assert!(
        decrypt_result.is_err(),
        "Decryption of tampered ciphertext should fail"
    );

    println!("test_reject_invalid_signature: PASSED");

    // Cleanup
    node_a.shutdown().await;
    node_b.shutdown().await;
}

// ============================================================================
// Milestone 7.4: Reject Wrong Sender
// ============================================================================

/// Milestone 7.4: Test rejection of messages claiming wrong sender
///
/// This test verifies that:
/// 1. Node A creates envelope claiming to be from Node C's DID
/// 2. Node B should reject (signature won't verify against claimed DID)
#[tokio::test]
async fn test_reject_wrong_sender() {
    let _ = tracing_subscriber::fmt::try_init();

    // Create two nodes
    let node_a = TestNode::new().await.expect("Failed to create node A");
    let node_b = TestNode::new().await.expect("Failed to create node B");

    // Share addresses
    node_a.add_peer(node_b.addr());
    node_b.add_peer(node_a.addr());

    // Generate identities
    let keypair_a = MockHybridKeypair::generate(); // Node A's real identity
    let keypair_c = MockHybridKeypair::generate(); // Node C's identity (impersonation target)

    println!("Node A DID: {}", keypair_a.did());
    println!("Node C DID (target): {}", keypair_c.did());

    // Create realm and topic
    let realm_id = RealmId::new();
    let realm_key = RealmCrypto::generate_key();
    let topic = realm_topic(&realm_id);

    // Both nodes subscribe
    let topic_a = node_a
        .subscribe(topic, vec![])
        .await
        .expect("Node A failed to subscribe");
    let (sender_a, receiver_a) = topic_a.split();

    let topic_b = node_b
        .subscribe(topic, vec![node_a.id()])
        .await
        .expect("Node B failed to subscribe");
    let (_sender_b, receiver_b) = topic_b.split();

    // Wait for neighbors
    let (_receiver_a, _) = wait_for_neighbor(receiver_a, 10)
        .await
        .expect("Node A: no neighbor");
    let (mut receiver_b, _) = wait_for_neighbor(receiver_b, 10)
        .await
        .expect("Node B: no neighbor");

    // Node A creates envelope but LIES about sender (claims to be Node C)
    let plaintext = b"Forged message";
    let crypto = RealmCrypto::new(&realm_key);
    let ciphertext = crypto.encrypt(plaintext).expect("Failed to encrypt");

    // Create envelope with A's keypair (signed by A)
    let mut envelope = SyncEnvelope::new(&keypair_a, ciphertext);

    // FORGERY: Replace sender DID with Node C's DID
    envelope.sender_did = keypair_c.did().clone();
    envelope.sender_public_hash = keypair_c.public_hash;
    // NOTE: signature is still from keypair_a, so it won't match

    // Broadcast the forged envelope
    let envelope_bytes = envelope.encode().expect("Failed to encode envelope");
    sender_a
        .broadcast(Bytes::from(envelope_bytes))
        .await
        .expect("Failed to broadcast");

    // Node B receives and attempts to verify
    let received = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match receiver_b.try_next().await {
                Ok(Some(GossipEvent::Received(msg))) => return Ok(msg),
                Ok(Some(_)) => continue,
                Ok(None) => return Err(anyhow::anyhow!("Stream ended")),
                Err(e) => return Err(anyhow::anyhow!("Error: {}", e)),
            }
        }
    })
    .await
    .expect("Timeout waiting for message")
    .expect("Failed to receive message");

    let received_envelope =
        SyncEnvelope::decode(&received.content).expect("Failed to decode envelope");

    // The envelope claims to be from Node C
    assert_eq!(
        received_envelope.sender_did,
        *keypair_c.did(),
        "Forged envelope should claim Node C as sender"
    );

    // But verification against Node C's keypair should FAIL
    // (because the signature was made with Node A's key)
    let verification_result = received_envelope.verify_with_keypair(&keypair_c);
    assert!(
        !verification_result,
        "Forged message should fail verification against claimed sender"
    );

    // Also verify it wouldn't pass with Node A's key either
    // (because the DID/public_hash fields don't match)
    let verification_with_actual_signer = received_envelope.verify_with_keypair(&keypair_a);
    assert!(
        !verification_with_actual_signer,
        "Forged message should also fail against actual signer (DID mismatch)"
    );

    println!("Correctly rejected forged sender identity");
    println!("test_reject_wrong_sender: PASSED");

    // Cleanup
    node_a.shutdown().await;
    node_b.shutdown().await;
}

// ============================================================================
// Milestone 7.5: DID Format Validation
// ============================================================================

/// Milestone 7.5: Test DID format validation
///
/// This test verifies that:
/// 1. Valid DIDs: `did:sync:z...` with proper base58 are accepted
/// 2. Invalid DIDs are rejected
#[tokio::test]
async fn test_did_format_validation() {
    let _ = tracing_subscriber::fmt::try_init();

    // Test valid DID generation
    let keypair = MockHybridKeypair::generate();
    let valid_did = keypair.did();

    assert!(valid_did.is_valid(), "Generated DID should be valid");
    assert!(
        valid_did.0.starts_with("did:sync:z"),
        "DID should start with did:sync:z"
    );
    println!("Valid DID: {}", valid_did);

    // Test base58 portion extraction
    let base58 = valid_did
        .base58_portion()
        .expect("Should have base58 portion");
    let decoded = bs58::decode(base58)
        .into_vec()
        .expect("Base58 should decode");
    assert_eq!(decoded.len(), 32, "Base58 should decode to 32 bytes");

    // Test various invalid DID formats
    let invalid_dids = vec![
        Did("".to_string()),                         // Empty
        Did("did:sync:".to_string()),                // Missing z prefix
        Did("did:sync:z".to_string()),               // No base58 content
        Did("did:other:z123abc".to_string()),        // Wrong method
        Did("did:sync:x123abc".to_string()),         // Wrong multibase prefix
        Did("did:sync:z!!!invalid!!!".to_string()),  // Invalid base58 chars
        Did("notadid".to_string()),                  // Not a DID at all
        Did("did:sync:zO0Il".to_string()),           // Invalid base58 (O, 0, I, l)
    ];

    for invalid_did in &invalid_dids {
        assert!(
            !invalid_did.is_valid(),
            "DID '{}' should be invalid",
            invalid_did.0
        );
    }
    println!("Correctly rejected {} invalid DID formats", invalid_dids.len());

    // Test DID from specific public key hash
    let test_hash = [42u8; 32];
    let did_from_hash = Did::from_public_key_hash(&test_hash);
    assert!(did_from_hash.is_valid(), "DID from hash should be valid");

    // Verify reproducibility
    let did_from_hash_2 = Did::from_public_key_hash(&test_hash);
    assert_eq!(
        did_from_hash, did_from_hash_2,
        "Same hash should produce same DID"
    );

    // Different hash should produce different DID
    let other_hash = [99u8; 32];
    let other_did = Did::from_public_key_hash(&other_hash);
    assert_ne!(
        did_from_hash, other_did,
        "Different hashes should produce different DIDs"
    );

    println!("test_did_format_validation: PASSED");
}

// ============================================================================
// Milestone 7.6: Envelope Replay Protection (Future)
// ============================================================================

/// Milestone 7.6: Test envelope replay protection
///
/// This test verifies that:
/// 1. Same envelope received twice
/// 2. Second should be detected as replay
///
/// NOTE: This test demonstrates the mechanism. Full replay protection
/// requires maintaining a seen-nonces set, which would be implemented
/// in the sync engine.
#[tokio::test]
async fn test_envelope_replay_protection() {
    let _ = tracing_subscriber::fmt::try_init();

    // Generate identity
    let keypair = MockHybridKeypair::generate();

    // Create an envelope
    let payload = b"Important message".to_vec();
    let envelope1 = SyncEnvelope::new(&keypair, payload.clone());

    // Create another envelope with same payload
    let envelope2 = SyncEnvelope::new(&keypair, payload.clone());

    // The nonces should be different (each envelope has unique nonce)
    assert_ne!(
        envelope1.nonce, envelope2.nonce,
        "Different envelopes should have different nonces"
    );

    // Timestamps might differ (or be same if created in same millisecond)
    println!("Envelope 1 nonce: {:?}", &envelope1.nonce[..4]);
    println!("Envelope 2 nonce: {:?}", &envelope2.nonce[..4]);

    // Simulate replay detection with a seen-nonces set
    let mut seen_nonces: HashSet<[u8; 16]> = HashSet::new();

    // First envelope is new
    let is_replay_1 = seen_nonces.contains(&envelope1.nonce);
    assert!(!is_replay_1, "First envelope should not be detected as replay");
    seen_nonces.insert(envelope1.nonce);

    // If same envelope received again, it's a replay
    let is_replay_1_again = seen_nonces.contains(&envelope1.nonce);
    assert!(
        is_replay_1_again,
        "Same envelope again should be detected as replay"
    );

    // Second envelope (different nonce) is new
    let is_replay_2 = seen_nonces.contains(&envelope2.nonce);
    assert!(
        !is_replay_2,
        "Different envelope should not be detected as replay"
    );
    seen_nonces.insert(envelope2.nonce);

    println!(
        "Replay protection working: {} nonces tracked",
        seen_nonces.len()
    );

    // Test timestamp-based replay protection (optional additional check)
    // Messages older than a threshold could be rejected
    let max_age_ms: u64 = 300_000; // 5 minutes
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let is_too_old = current_time - envelope1.timestamp > max_age_ms;
    assert!(
        !is_too_old,
        "Fresh envelope should not be rejected for age"
    );

    // Create artificially old envelope for testing
    let mut old_envelope = SyncEnvelope::new(&keypair, b"old message".to_vec());
    old_envelope.timestamp = current_time - max_age_ms - 1000; // 5+ minutes ago

    let is_old_too_old = current_time - old_envelope.timestamp > max_age_ms;
    assert!(is_old_too_old, "Old envelope should be detected as too old");

    println!("test_envelope_replay_protection: PASSED");
}

/// Test multiple signed messages in sequence
///
/// Verifies that multiple messages from the same sender can be
/// signed, sent, and verified correctly.
#[tokio::test]
async fn test_multiple_signed_messages() {
    let _ = tracing_subscriber::fmt::try_init();

    // Create two nodes
    let node_a = TestNode::new().await.expect("Failed to create node A");
    let node_b = TestNode::new().await.expect("Failed to create node B");

    // Share addresses
    node_a.add_peer(node_b.addr());
    node_b.add_peer(node_a.addr());

    // Generate identity for Node A
    let keypair_a = MockHybridKeypair::generate();
    let realm_id = RealmId::new();
    let realm_key = RealmCrypto::generate_key();
    let topic = realm_topic(&realm_id);
    let crypto = RealmCrypto::new(&realm_key);

    // Both nodes subscribe
    let topic_a = node_a
        .subscribe(topic, vec![])
        .await
        .expect("Node A failed to subscribe");
    let (sender_a, receiver_a) = topic_a.split();

    let topic_b = node_b
        .subscribe(topic, vec![node_a.id()])
        .await
        .expect("Node B failed to subscribe");
    let (_sender_b, receiver_b) = topic_b.split();

    // Wait for neighbors
    let (_receiver_a, _) = wait_for_neighbor(receiver_a, 10)
        .await
        .expect("Node A: no neighbor");
    let (mut receiver_b, _) = wait_for_neighbor(receiver_b, 10)
        .await
        .expect("Node B: no neighbor");

    // Send multiple signed messages
    let messages = vec![
        b"First signed message".to_vec(),
        b"Second signed message".to_vec(),
        b"Third signed message".to_vec(),
    ];

    for msg in &messages {
        let ciphertext = crypto.encrypt(msg).expect("Failed to encrypt");
        let envelope = SyncEnvelope::new(&keypair_a, ciphertext);
        let envelope_bytes = envelope.encode().expect("Failed to encode");
        sender_a
            .broadcast(Bytes::from(envelope_bytes))
            .await
            .expect("Failed to broadcast");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Receive and verify all messages
    let mut received_messages = Vec::new();
    let receive_result = tokio::time::timeout(Duration::from_secs(10), async {
        while received_messages.len() < messages.len() {
            match receiver_b.try_next().await {
                Ok(Some(GossipEvent::Received(msg))) => {
                    let envelope = SyncEnvelope::decode(&msg.content).expect("Failed to decode");
                    // Verify signature
                    assert!(
                        envelope.verify_with_keypair(&keypair_a),
                        "Message {} signature should verify",
                        received_messages.len() + 1
                    );
                    // Decrypt and store
                    let decrypted = crypto.decrypt(&envelope.payload).expect("Failed to decrypt");
                    received_messages.push(decrypted);
                }
                Ok(Some(_)) => continue,
                Ok(None) => break,
                Err(e) => return Err(anyhow::anyhow!("Error: {}", e)),
            }
        }
        Ok(())
    })
    .await;

    assert!(
        receive_result.is_ok() && receive_result.unwrap().is_ok(),
        "Should receive all messages"
    );
    assert_eq!(
        received_messages.len(),
        messages.len(),
        "Should receive all messages"
    );

    // Verify content (order may vary in gossip)
    for msg in &messages {
        assert!(
            received_messages.contains(msg),
            "Missing message: {:?}",
            String::from_utf8_lossy(msg)
        );
    }

    println!(
        "Successfully sent, verified, and decrypted {} signed messages",
        messages.len()
    );

    // Cleanup
    node_a.shutdown().await;
    node_b.shutdown().await;
}

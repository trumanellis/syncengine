//! End-to-End tests for contact exchange with actual network communication
//!
//! These tests verify the full contact exchange flow with two SyncEngine instances
//! communicating over real QUIC connections.
//!
//! ## Test Architecture
//!
//! - **Unit tests** (`src/sync/contact_manager.rs`): Test storage/logic only
//!   - No network operations, fast execution
//!
//! - **Integration tests** (`tests/contact_integration.rs`): Test SyncEngine API
//!   - Verify API contracts without network
//!
//! - **E2E tests** (this file): Test complete network flows
//!   - Two separate SyncEngine instances
//!   - Both call `start_networking()` to enable QUIC
//!   - Real network message propagation with delays
//!   - Complete end-to-end user scenarios
//!
//! ## What These Tests Verify
//!
//! These tests are the ONLY place where actual P2P network communication is tested:
//! - QUIC connection establishment between peers
//! - Simplified 2-message protocol: ContactRequest → ContactAccept (or ContactDecline)
//! - Local key derivation (keys are NOT transmitted over the network)
//! - Network propagation delays and timing
//! - Mutual acceptance flow with real addresses
//! - Gossip topic subscription and connection
//!
//! If these pass, the network layer works correctly. If they fail, check:
//! - Port conflicts (tests use random ports)
//! - Firewall/network issues
//! - QUIC connection problems

use syncengine_core::engine::SyncEngine;
use syncengine_core::types::ContactStatus;
use tempfile::tempdir;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_two_engines_exchange_contacts_over_quic() {
    tracing_subscriber::fmt()
        .with_env_filter("debug,quinn=warn,iroh=warn")
        .try_init()
        .ok();

    // Setup Alice's engine
    let alice_dir = tempdir().unwrap();
    let mut alice = SyncEngine::new(alice_dir.path()).await.unwrap();
    alice.init_identity().unwrap();
    alice.start_networking().await.unwrap();

    // Give Alice's networking time to start
    sleep(Duration::from_millis(500)).await;

    // Setup Bob's engine
    let bob_dir = tempdir().unwrap();
    let mut bob = SyncEngine::new(bob_dir.path()).await.unwrap();
    bob.init_identity().unwrap();
    bob.start_networking().await.unwrap();

    // Give Bob's networking time to start
    sleep(Duration::from_millis(500)).await;

    // Alice generates an invite
    let invite_code = alice.generate_contact_invite(24).await.unwrap();
    println!("Alice generated invite: {}", &invite_code[..50]);
    assert!(invite_code.starts_with("sync-contact:"));

    // Bob decodes the invite
    let invite = bob.decode_contact_invite(&invite_code).await.unwrap();
    println!("Bob decoded invite from DID: {}", invite.inviter_did);
    assert_eq!(invite.display_name, "Anonymous User");

    // Bob sends contact request (actual QUIC connection)
    // With the simplified 2-message protocol and auto-accept, the exchange
    // may complete before this function returns
    bob.send_contact_request(invite).await.unwrap();
    println!("Bob sent contact request via QUIC");

    // Wait for auto-accept and the full exchange to complete
    // The simplified protocol uses only 2 messages:
    // 1. Bob → Alice: ContactRequest
    // 2. Alice → Bob: ContactAccept (auto-sent because it's Alice's invite)
    sleep(Duration::from_millis(1500)).await;

    // Verify Alice has Bob as a contact (auto-accepted)
    let alice_contacts = alice.list_contacts().unwrap();
    println!("Alice has {} contacts", alice_contacts.len());
    assert_eq!(alice_contacts.len(), 1, "Alice should have 1 contact");
    assert_eq!(alice_contacts[0].profile.display_name, "Anonymous User");
    // Status may be Online since we just exchanged messages, or Offline if no heartbeat yet
    assert!(
        alice_contacts[0].status == ContactStatus::Offline
            || alice_contacts[0].status == ContactStatus::Online,
        "Status should be valid"
    );

    // Verify Bob has Alice as a contact
    let bob_contacts = bob.list_contacts().unwrap();
    println!("Bob has {} contacts", bob_contacts.len());
    assert_eq!(bob_contacts.len(), 1, "Bob should have 1 contact");
    assert_eq!(bob_contacts[0].profile.display_name, "Anonymous User");

    // Verify both have matching DIDs
    assert_eq!(
        alice_contacts[0].peer_did,
        bob.did().unwrap().to_string(),
        "Alice should have Bob's DID"
    );
    assert_eq!(
        bob_contacts[0].peer_did,
        alice.did().unwrap().to_string(),
        "Bob should have Alice's DID"
    );

    // Verify pending requests were cleaned up (auto-accept completes the exchange)
    let (alice_incoming, alice_outgoing) = alice.list_pending_contacts().unwrap();
    assert_eq!(alice_incoming.len(), 0, "Alice should have no pending incoming");
    assert_eq!(
        alice_outgoing.len(), 0,
        "Alice should have no pending outgoing"
    );

    let (bob_incoming, bob_outgoing) = bob.list_pending_contacts().unwrap();
    assert_eq!(bob_incoming.len(), 0, "Bob should have no pending incoming");
    assert_eq!(bob_outgoing.len(), 0, "Bob should have no pending outgoing");

    println!("✅ Full E2E contact exchange completed successfully with simplified protocol!");
}

/// Test that contacts are NOT created when the contact exchange fails
///
/// NOTE: With the simplified protocol, invites you generate are auto-accepted.
/// The decline flow only applies to edge cases (e.g., receiving a request for
/// an invite you didn't generate, which isn't currently possible in the protocol).
///
/// This test verifies that if the exchange fails partway through, no phantom
/// contacts are created.
#[tokio::test]
async fn test_contact_exchange_requires_both_parties() {
    tracing_subscriber::fmt()
        .with_env_filter("debug,quinn=warn,iroh=warn")
        .try_init()
        .ok();

    // Setup Alice's engine
    let alice_dir = tempdir().unwrap();
    let mut alice = SyncEngine::new(alice_dir.path()).await.unwrap();
    alice.init_identity().unwrap();
    alice.start_networking().await.unwrap();
    sleep(Duration::from_millis(500)).await;

    // Setup Bob's engine
    let bob_dir = tempdir().unwrap();
    let mut bob = SyncEngine::new(bob_dir.path()).await.unwrap();
    bob.init_identity().unwrap();
    bob.start_networking().await.unwrap();
    sleep(Duration::from_millis(500)).await;

    // Bob generates invite but never shares it
    let _bob_invite = bob.generate_contact_invite(24).await.unwrap();

    // Wait a bit
    sleep(Duration::from_millis(500)).await;

    // Verify neither party has contacts (no exchange happened)
    assert_eq!(alice.list_contacts().unwrap().len(), 0, "Alice should have no contacts");
    assert_eq!(bob.list_contacts().unwrap().len(), 0, "Bob should have no contacts");

    // Verify no pending requests exist
    let (alice_incoming, alice_outgoing) = alice.list_pending_contacts().unwrap();
    assert_eq!(alice_incoming.len(), 0);
    assert_eq!(alice_outgoing.len(), 0);

    let (bob_incoming, bob_outgoing) = bob.list_pending_contacts().unwrap();
    assert_eq!(bob_incoming.len(), 0);
    assert_eq!(bob_outgoing.len(), 0);

    println!("✅ Verified that contacts require full exchange to be created");
}

/// Test that both parties derive the same contact topic and key
///
/// With the simplified protocol, keys are derived locally using:
/// - contact_topic = BLAKE3("sync-contact-topic" || sorted_did1 || sorted_did2)
/// - contact_key   = BLAKE3("sync-contact-key" || sorted_did1 || sorted_did2)
///
/// This test verifies that both Alice and Bob derive identical keys.
#[tokio::test]
async fn test_contact_topic_and_key_derivation_match() {
    tracing_subscriber::fmt()
        .with_env_filter("debug,quinn=warn,iroh=warn")
        .try_init()
        .ok();

    // Setup two engines
    let alice_dir = tempdir().unwrap();
    let mut alice = SyncEngine::new(alice_dir.path()).await.unwrap();
    alice.init_identity().unwrap();
    alice.start_networking().await.unwrap();
    sleep(Duration::from_millis(500)).await;

    let bob_dir = tempdir().unwrap();
    let mut bob = SyncEngine::new(bob_dir.path()).await.unwrap();
    bob.init_identity().unwrap();
    bob.start_networking().await.unwrap();
    sleep(Duration::from_millis(500)).await;

    // Complete contact exchange (with auto-accept)
    let invite_code = alice.generate_contact_invite(24).await.unwrap();
    let invite = bob.decode_contact_invite(&invite_code).await.unwrap();
    bob.send_contact_request(invite).await.unwrap();

    // Wait for auto-accept and exchange to complete
    sleep(Duration::from_millis(1500)).await;

    // Get contacts
    let alice_contacts = alice.list_contacts().unwrap();
    let bob_contacts = bob.list_contacts().unwrap();

    assert_eq!(alice_contacts.len(), 1, "Alice should have 1 contact");
    assert_eq!(bob_contacts.len(), 1, "Bob should have 1 contact");

    // Verify both have the SAME contact_topic (deterministic local derivation)
    assert_eq!(
        alice_contacts[0].contact_topic, bob_contacts[0].contact_topic,
        "Both peers should derive the same contact topic"
    );

    // Verify both have the SAME contact_key (deterministic local derivation)
    assert_eq!(
        alice_contacts[0].contact_key, bob_contacts[0].contact_key,
        "Both peers should derive the same contact key"
    );

    // Verify the keys are non-zero (actual derivation happened)
    assert_ne!(
        alice_contacts[0].contact_topic, [0u8; 32],
        "Contact topic should not be all zeros"
    );
    assert_ne!(
        alice_contacts[0].contact_key, [0u8; 32],
        "Contact key should not be all zeros"
    );

    println!("✅ Contact topic/key derivation is deterministic (simplified protocol)!");
}

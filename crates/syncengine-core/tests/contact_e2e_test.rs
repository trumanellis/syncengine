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
//! - ContactRequest/ContactResponse/ContactAccepted message exchange
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
    assert_eq!(invite.profile_snapshot.display_name, "Anonymous User");

    // Bob sends contact request (actual QUIC connection)
    bob.send_contact_request(invite).await.unwrap();
    println!("Bob sent contact request via QUIC");

    // Verify Bob has outgoing pending request
    let (bob_incoming, bob_outgoing) = bob.list_pending_contacts().unwrap();
    assert_eq!(bob_incoming.len(), 0, "Bob should have no incoming requests");
    assert_eq!(bob_outgoing.len(), 1, "Bob should have 1 outgoing request");

    // Wait for Alice to receive the request over the network
    sleep(Duration::from_millis(1000)).await;

    // Alice checks for incoming requests
    let (alice_incoming, alice_outgoing) = alice.list_pending_contacts().unwrap();
    assert_eq!(
        alice_incoming.len(),
        1,
        "Alice should have 1 incoming request"
    );
    assert_eq!(alice_outgoing.len(), 0, "Alice should have no outgoing requests");

    let alice_request = &alice_incoming[0];
    println!(
        "Alice received request from: {}",
        alice_request.profile.display_name
    );

    // Alice accepts the request (sends acceptance via QUIC)
    alice
        .accept_contact(&alice_request.invite_id)
        .await
        .unwrap();
    println!("Alice accepted contact request");

    // Wait for mutual acceptance to complete
    sleep(Duration::from_millis(1500)).await;

    // Verify Alice has Bob as a contact
    let alice_contacts = alice.list_contacts().unwrap();
    println!("Alice has {} contacts", alice_contacts.len());
    assert_eq!(alice_contacts.len(), 1, "Alice should have 1 contact");
    assert_eq!(alice_contacts[0].profile.display_name, "Anonymous User");
    assert_eq!(alice_contacts[0].status, ContactStatus::Offline);

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

    // Verify pending requests were cleaned up
    let (alice_incoming, alice_outgoing) = alice.list_pending_contacts().unwrap();
    assert_eq!(alice_incoming.len(), 0, "Alice should have no pending incoming");
    assert_eq!(
        alice_outgoing.len(), 0,
        "Alice should have no pending outgoing"
    );

    let (bob_incoming, bob_outgoing) = bob.list_pending_contacts().unwrap();
    assert_eq!(bob_incoming.len(), 0, "Bob should have no pending incoming");
    assert_eq!(bob_outgoing.len(), 0, "Bob should have no pending outgoing");

    println!("✅ Full E2E contact exchange completed successfully!");
}

#[tokio::test]
async fn test_contact_request_declined_over_quic() {
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

    // Alice generates invite
    let invite_code = alice.generate_contact_invite(24).await.unwrap();

    // Bob sends contact request
    let invite = bob.decode_contact_invite(&invite_code).await.unwrap();
    bob.send_contact_request(invite).await.unwrap();

    // Wait for network propagation
    sleep(Duration::from_millis(1000)).await;

    // Alice receives request
    let (alice_incoming, _) = alice.list_pending_contacts().unwrap();
    assert_eq!(alice_incoming.len(), 1);

    // Alice DECLINES the request
    alice
        .decline_contact(&alice_incoming[0].invite_id)
        .await
        .unwrap();

    // Wait for decline message to propagate
    sleep(Duration::from_millis(1000)).await;

    // Verify Alice has no contacts
    assert_eq!(alice.list_contacts().unwrap().len(), 0);

    // Verify Bob has no contacts (received decline)
    assert_eq!(bob.list_contacts().unwrap().len(), 0);

    // Verify Alice's pending is cleared
    let (alice_incoming, _) = alice.list_pending_contacts().unwrap();
    assert_eq!(alice_incoming.len(), 0);

    println!("✅ Contact request decline flow completed successfully!");
}

#[tokio::test]
async fn test_contact_topic_and_key_derivation_match() {
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

    // Complete contact exchange
    let invite_code = alice.generate_contact_invite(24).await.unwrap();
    let invite = bob.decode_contact_invite(&invite_code).await.unwrap();
    bob.send_contact_request(invite).await.unwrap();
    sleep(Duration::from_millis(1000)).await;

    let (alice_incoming, _) = alice.list_pending_contacts().unwrap();
    alice
        .accept_contact(&alice_incoming[0].invite_id)
        .await
        .unwrap();
    sleep(Duration::from_millis(1500)).await;

    // Get contacts
    let alice_contacts = alice.list_contacts().unwrap();
    let bob_contacts = bob.list_contacts().unwrap();

    assert_eq!(alice_contacts.len(), 1);
    assert_eq!(bob_contacts.len(), 1);

    // Verify both have the SAME contact_topic (deterministic derivation)
    assert_eq!(
        alice_contacts[0].contact_topic, bob_contacts[0].contact_topic,
        "Both peers should derive the same contact topic"
    );

    // Verify both have the SAME contact_key (deterministic derivation)
    assert_eq!(
        alice_contacts[0].contact_key, bob_contacts[0].contact_key,
        "Both peers should derive the same contact key"
    );

    println!("✅ Contact topic/key derivation is deterministic!");
}

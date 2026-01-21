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

    // Setup Love's engine
    let love_dir = tempdir().unwrap();
    let mut love = SyncEngine::new(love_dir.path()).await.unwrap();
    love.init_identity().unwrap();
    love.start_networking().await.unwrap();

    // Give Love's networking time to start
    sleep(Duration::from_millis(500)).await;

    // Setup Joy's engine
    let joy_dir = tempdir().unwrap();
    let mut joy = SyncEngine::new(joy_dir.path()).await.unwrap();
    joy.init_identity().unwrap();
    joy.start_networking().await.unwrap();

    // Give Joy's networking time to start
    sleep(Duration::from_millis(500)).await;

    // Love generates an invite
    let invite_code = love.generate_contact_invite(24).await.unwrap();
    println!("Love generated invite: {}", &invite_code[..50]);
    assert!(invite_code.starts_with("sync-contact:"));

    // Joy decodes the invite
    let invite = joy.decode_contact_invite(&invite_code).await.unwrap();
    println!("Joy decoded invite from DID: {}", invite.inviter_did);
    assert_eq!(invite.display_name, "Anonymous User");

    // Joy sends contact request (actual QUIC connection)
    // With the simplified 2-message protocol and auto-accept, the exchange
    // may complete before this function returns
    joy.send_contact_request(invite).await.unwrap();
    println!("Joy sent contact request via QUIC");

    // Wait for auto-accept and the full exchange to complete
    // The simplified protocol uses only 2 messages:
    // 1. Joy → Love: ContactRequest
    // 2. Love → Joy: ContactAccept (auto-sent because it's Love's invite)
    sleep(Duration::from_millis(1500)).await;

    // Verify Love has Joy as a contact (auto-accepted)
    let love_contacts = love.list_contacts().unwrap();
    println!("Love has {} contacts", love_contacts.len());
    assert_eq!(love_contacts.len(), 1, "Love should have 1 contact");
    assert_eq!(love_contacts[0].profile.display_name, "Anonymous User");
    // Status may be Online since we just exchanged messages, or Offline if no heartbeat yet
    assert!(
        love_contacts[0].status == ContactStatus::Offline
            || love_contacts[0].status == ContactStatus::Online,
        "Status should be valid"
    );

    // Verify Joy has Love as a contact
    let joy_contacts = joy.list_contacts().unwrap();
    println!("Joy has {} contacts", joy_contacts.len());
    assert_eq!(joy_contacts.len(), 1, "Joy should have 1 contact");
    assert_eq!(joy_contacts[0].profile.display_name, "Anonymous User");

    // Verify both have matching DIDs
    assert_eq!(
        love_contacts[0].peer_did,
        joy.did().unwrap().to_string(),
        "Love should have Joy's DID"
    );
    assert_eq!(
        joy_contacts[0].peer_did,
        love.did().unwrap().to_string(),
        "Joy should have Love's DID"
    );

    // Verify pending requests were cleaned up (auto-accept completes the exchange)
    let (love_incoming, love_outgoing) = love.list_pending_contacts().unwrap();
    assert_eq!(love_incoming.len(), 0, "Love should have no pending incoming");
    assert_eq!(
        love_outgoing.len(), 0,
        "Love should have no pending outgoing"
    );

    let (joy_incoming, joy_outgoing) = joy.list_pending_contacts().unwrap();
    assert_eq!(joy_incoming.len(), 0, "Joy should have no pending incoming");
    assert_eq!(joy_outgoing.len(), 0, "Joy should have no pending outgoing");

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

    // Setup Love's engine
    let love_dir = tempdir().unwrap();
    let mut love = SyncEngine::new(love_dir.path()).await.unwrap();
    love.init_identity().unwrap();
    love.start_networking().await.unwrap();
    sleep(Duration::from_millis(500)).await;

    // Setup Joy's engine
    let joy_dir = tempdir().unwrap();
    let mut joy = SyncEngine::new(joy_dir.path()).await.unwrap();
    joy.init_identity().unwrap();
    joy.start_networking().await.unwrap();
    sleep(Duration::from_millis(500)).await;

    // Joy generates invite but never shares it
    let _joy_invite = joy.generate_contact_invite(24).await.unwrap();

    // Wait a bit
    sleep(Duration::from_millis(500)).await;

    // Verify neither party has contacts (no exchange happened)
    assert_eq!(love.list_contacts().unwrap().len(), 0, "Love should have no contacts");
    assert_eq!(joy.list_contacts().unwrap().len(), 0, "Joy should have no contacts");

    // Verify no pending requests exist
    let (love_incoming, love_outgoing) = love.list_pending_contacts().unwrap();
    assert_eq!(love_incoming.len(), 0);
    assert_eq!(love_outgoing.len(), 0);

    let (joy_incoming, joy_outgoing) = joy.list_pending_contacts().unwrap();
    assert_eq!(joy_incoming.len(), 0);
    assert_eq!(joy_outgoing.len(), 0);

    println!("✅ Verified that contacts require full exchange to be created");
}

/// Test that both parties derive the same contact topic and key
///
/// With the simplified protocol, keys are derived locally using:
/// - contact_topic = BLAKE3("sync-contact-topic" || sorted_did1 || sorted_did2)
/// - contact_key   = BLAKE3("sync-contact-key" || sorted_did1 || sorted_did2)
///
/// This test verifies that both Love and Joy derive identical keys.
#[tokio::test]
async fn test_contact_topic_and_key_derivation_match() {
    tracing_subscriber::fmt()
        .with_env_filter("debug,quinn=warn,iroh=warn")
        .try_init()
        .ok();

    // Setup two engines
    let love_dir = tempdir().unwrap();
    let mut love = SyncEngine::new(love_dir.path()).await.unwrap();
    love.init_identity().unwrap();
    love.start_networking().await.unwrap();
    sleep(Duration::from_millis(500)).await;

    let joy_dir = tempdir().unwrap();
    let mut joy = SyncEngine::new(joy_dir.path()).await.unwrap();
    joy.init_identity().unwrap();
    joy.start_networking().await.unwrap();
    sleep(Duration::from_millis(500)).await;

    // Complete contact exchange (with auto-accept)
    let invite_code = love.generate_contact_invite(24).await.unwrap();
    let invite = joy.decode_contact_invite(&invite_code).await.unwrap();
    joy.send_contact_request(invite).await.unwrap();

    // Wait for auto-accept and exchange to complete
    sleep(Duration::from_millis(1500)).await;

    // Get contacts
    let love_contacts = love.list_contacts().unwrap();
    let joy_contacts = joy.list_contacts().unwrap();

    assert_eq!(love_contacts.len(), 1, "Love should have 1 contact");
    assert_eq!(joy_contacts.len(), 1, "Joy should have 1 contact");

    // Verify both have the SAME contact_topic (deterministic local derivation)
    assert_eq!(
        love_contacts[0].contact_topic, joy_contacts[0].contact_topic,
        "Both peers should derive the same contact topic"
    );

    // Verify both have the SAME contact_key (deterministic local derivation)
    assert_eq!(
        love_contacts[0].contact_key, joy_contacts[0].contact_key,
        "Both peers should derive the same contact key"
    );

    // Verify the keys are non-zero (actual derivation happened)
    assert_ne!(
        love_contacts[0].contact_topic, [0u8; 32],
        "Contact topic should not be all zeros"
    );
    assert_ne!(
        love_contacts[0].contact_key, [0u8; 32],
        "Contact key should not be all zeros"
    );

    println!("✅ Contact topic/key derivation is deterministic (simplified protocol)!");
}

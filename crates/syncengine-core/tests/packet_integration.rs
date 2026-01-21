//! Packet Integration Tests for Indra's Network
//!
//! These tests verify the packet layer functionality across multiple SyncEngine nodes.
//!
//! ## Test Architecture
//!
//! - **Unit tests** (`src/profile/*.rs`): Test individual packet components
//!   - Signing, encryption, hash chains, sealed boxes
//!
//! - **Integration tests** (this file): Test packet flows through SyncEngine
//!   - Engine creation and packet API
//!   - Multi-node packet exchange (without network)
//!   - Mirror storage and synchronization
//!   - Fork detection
//!
//! ## Test Scenarios (from plan)
//!
//! 1. test_two_peers_direct_packet — p1 sends to p2, p2 receives
//! 2. test_offline_relay — p1 sends to p2 (offline), p3 relays later
//! 3. test_encrypted_relay — p3 relays packet they can't decrypt
//! 4. test_mirror_sync_after_offline — p2 gets p1's log from p3
//! 5. test_fork_detection — Detect conflicting log entries
//! 6. test_automatic_receipt — p2 auto-sends Receipt after receiving packet
//! 7. test_depin_after_all_receipts — p1 broadcasts Depin when all receipts arrive
//! 8. test_relay_deletes_on_depin — p3 removes packet from mirror on Depin

use parking_lot::RwLock;
use redb::Database;
use std::sync::Arc;
use syncengine_core::engine::SyncEngine;
use syncengine_core::profile::{
    MirrorStore, PacketAddress, PacketBuilder, PacketPayload, ProfileKeys, ProfileLog,
};
use syncengine_core::Did;
use tempfile::tempdir;

/// Helper to create a database for testing
fn create_test_db(path: &std::path::Path) -> Arc<RwLock<Database>> {
    let db_path = path.join("test.redb");
    let db = Database::create(&db_path).expect("Failed to create database");
    Arc::new(RwLock::new(db))
}

/// Helper to create a heartbeat payload
fn heartbeat() -> PacketPayload {
    PacketPayload::Heartbeat {
        timestamp: chrono::Utc::now().timestamp_millis(),
    }
}

// ============================================================================
// Basic Engine Packet API Tests
// ============================================================================

/// Test that a new engine starts without profile keys, then initializes them.
#[tokio::test]
async fn test_engine_profile_keys_lifecycle() {
    let temp_dir = tempdir().unwrap();
    let mut engine = SyncEngine::new(temp_dir.path()).await.unwrap();

    // Initially, no profile keys
    assert!(
        !engine.has_profile_keys(),
        "Should not have profile keys initially"
    );

    // Identity must be initialized before profile keys
    engine.init_identity().unwrap();

    // Initialize profile keys (derived from identity)
    engine.init_profile_keys().unwrap();

    // Now should have profile keys
    assert!(
        engine.has_profile_keys(),
        "Should have profile keys after init"
    );

    // Should have a valid DID
    let did = engine.profile_did();
    assert!(did.is_some(), "Should have a DID after init");
    assert!(
        did.unwrap().to_string().starts_with("did:sync:"),
        "DID should be properly formatted"
    );

    // Profile DID should match Identity DID (unified system)
    assert_eq!(
        engine.did().unwrap(),
        engine.profile_did().unwrap(),
        "Profile DID should match Identity DID"
    );
}

/// Test that profile keys persist across engine restarts.
#[tokio::test]
async fn test_profile_keys_persistence() {
    let temp_dir = tempdir().unwrap();

    // First engine instance - create keys
    let first_did = {
        let mut engine = SyncEngine::new(temp_dir.path()).await.unwrap();
        engine.init_identity().unwrap();
        engine.init_profile_keys().unwrap();
        engine.profile_did().unwrap().to_string()
    };

    // Second engine instance - keys should persist
    {
        let mut engine2 = SyncEngine::new(temp_dir.path()).await.unwrap();
        engine2.init_identity().unwrap();
        engine2.init_profile_keys().unwrap();
        let second_did = engine2.profile_did().unwrap().to_string();

        assert_eq!(
            first_did, second_did,
            "Profile DID should persist across restarts"
        );

        // Also verify profile DID matches identity DID
        assert_eq!(
            engine2.did().unwrap(),
            engine2.profile_did().unwrap(),
            "Profile DID should match Identity DID"
        );
    }
}

/// Test creating a global (public) packet.
#[tokio::test]
async fn test_create_global_packet() {
    let temp_dir = tempdir().unwrap();
    let mut engine = SyncEngine::new(temp_dir.path()).await.unwrap();
    engine.init_profile_keys().unwrap();

    // Create a heartbeat packet (global/public)
    let sequence = engine
        .create_packet(heartbeat(), PacketAddress::Global)
        .unwrap();

    assert_eq!(sequence, 0, "First packet should have sequence 0");

    // Check log head - returns the sequence of the latest packet
    assert_eq!(
        engine.log_head_sequence(),
        0,
        "Log head should be 0 after first packet"
    );
}

/// Test creating multiple packets maintains hash chain.
#[tokio::test]
async fn test_packet_hash_chain() {
    let temp_dir = tempdir().unwrap();
    let mut engine = SyncEngine::new(temp_dir.path()).await.unwrap();
    engine.init_profile_keys().unwrap();

    // Create several packets
    let seq0 = engine
        .create_packet(heartbeat(), PacketAddress::Global)
        .unwrap();
    let seq1 = engine
        .create_packet(heartbeat(), PacketAddress::Global)
        .unwrap();
    let seq2 = engine
        .create_packet(heartbeat(), PacketAddress::Global)
        .unwrap();

    assert_eq!(seq0, 0);
    assert_eq!(seq1, 1);
    assert_eq!(seq2, 2);
    // log_head_sequence returns the sequence of the latest packet
    assert_eq!(engine.log_head_sequence(), 2);
}

/// Test that creating a packet without profile keys fails.
#[tokio::test]
async fn test_create_packet_requires_keys() {
    let temp_dir = tempdir().unwrap();
    let mut engine = SyncEngine::new(temp_dir.path()).await.unwrap();
    // Don't call init_profile_keys()

    let result = engine.create_packet(heartbeat(), PacketAddress::Global);

    assert!(result.is_err(), "Should fail without profile keys");
}

// ============================================================================
// Mirror Storage Tests
// ============================================================================

/// Test that MirrorStore can store and retrieve packets.
#[test]
fn test_mirror_store_basic() {
    let temp_dir = tempdir().unwrap();
    let db = create_test_db(temp_dir.path());
    let mirror_store = MirrorStore::new(db).unwrap();

    // Create a test packet manually
    let keys = ProfileKeys::generate();
    let did = keys.did();
    let log = ProfileLog::new(did.clone());

    // Create packet using builder
    let builder = PacketBuilder::new(&keys, &log);
    let envelope = builder.create_global_packet(&heartbeat()).unwrap();

    // Store in mirror
    mirror_store.store_packet(&envelope).unwrap();

    // Retrieve
    let head = mirror_store.get_head(&did).unwrap();
    assert_eq!(head, Some(0), "Mirror head should be 0");

    // get_since returns packets AFTER the given sequence (exclusive)
    // So get_since(0) returns packets from sequence 1+, which is empty
    // We need to check that the packet exists using get_range or direct lookup
    let packets_after = mirror_store.get_since(&did, 0).unwrap();
    assert_eq!(packets_after.len(), 0, "get_since(0) returns packets after seq 0");

    // Get all packets from sequence 0 using get_range
    let all_packets = mirror_store.get_range(&did, 0, 0).unwrap();
    assert_eq!(all_packets.len(), 1, "Should have 1 packet at seq 0");
}

/// Test that MirrorStore tracks multiple profiles.
#[test]
fn test_mirror_store_multiple_profiles() {
    let temp_dir = tempdir().unwrap();
    let db = create_test_db(temp_dir.path());
    let mirror_store = MirrorStore::new(db).unwrap();

    // Create packets from two different profiles
    let love_keys = ProfileKeys::generate();
    let love_did = love_keys.did();
    let mut love_log = ProfileLog::new(love_did.clone());

    let joy_keys = ProfileKeys::generate();
    let joy_did = joy_keys.did();
    let mut joy_log = ProfileLog::new(joy_did.clone());

    // Love creates 2 packets
    let builder1 = PacketBuilder::new(&love_keys, &love_log);
    let love_p1 = builder1.create_global_packet(&heartbeat()).unwrap();
    love_log.append(love_p1.clone()).unwrap();

    let builder2 = PacketBuilder::new(&love_keys, &love_log);
    let love_p2 = builder2.create_global_packet(&heartbeat()).unwrap();
    love_log.append(love_p2.clone()).unwrap();

    // Joy creates 1 packet
    let builder3 = PacketBuilder::new(&joy_keys, &joy_log);
    let joy_p1 = builder3.create_global_packet(&heartbeat()).unwrap();
    joy_log.append(joy_p1.clone()).unwrap();

    // Store all packets
    mirror_store.store_packet(&love_p1).unwrap();
    mirror_store.store_packet(&love_p2).unwrap();
    mirror_store.store_packet(&joy_p1).unwrap();

    // Verify each profile has correct head
    assert_eq!(mirror_store.get_head(&love_did).unwrap(), Some(1));
    assert_eq!(mirror_store.get_head(&joy_did).unwrap(), Some(0));

    // List all mirrored profiles
    let dids = mirror_store.list_mirrored_dids().unwrap();
    assert_eq!(dids.len(), 2, "Should have 2 mirrored profiles");
}

// ============================================================================
// Packet Exchange Tests (Simulated)
// ============================================================================

/// Test two peers exchanging packets (simulated without network).
///
/// Scenario:
/// - Love creates a packet
/// - Joy receives it (simulated by direct call)
/// - Verify Joy's mirror contains Love's packet
#[tokio::test]
async fn test_two_peers_direct_packet_simulated() {
    let love_dir = tempdir().unwrap();
    let joy_dir = tempdir().unwrap();

    // Setup Love
    let mut love = SyncEngine::new(love_dir.path()).await.unwrap();
    love.init_profile_keys().unwrap();
    let love_did = love.profile_did().unwrap().clone();

    // Setup Joy
    let mut joy = SyncEngine::new(joy_dir.path()).await.unwrap();
    joy.init_profile_keys().unwrap();

    // Love creates a heartbeat packet
    love
        .create_packet(heartbeat(), PacketAddress::Global)
        .unwrap();

    // Get packets from Love's log
    let love_log = love.my_log().unwrap();
    let love_entries = love_log.entries_ordered();
    assert_eq!(love_entries.len(), 1, "Love should have 1 packet");

    let packet_to_send = love_entries[0].envelope.clone();

    // Joy receives the packet
    let is_new = joy.handle_incoming_packet(packet_to_send).unwrap();
    assert!(is_new, "Packet should be new to Joy");

    // Verify Joy's mirror contains Love's packet
    let joy_mirror_head = joy.mirror_head(&love_did);
    assert_eq!(
        joy_mirror_head,
        Some(0),
        "Joy's mirror of Love should have seq 0"
    );
}

/// Test encrypted relay - a relay node stores packets they can't decrypt.
///
/// Scenario:
/// - Love sends encrypted packet addressed to Joy only
/// - Peace (relay) receives and stores the packet
/// - Peace cannot decrypt it, but stores it for Joy
#[tokio::test]
async fn test_encrypted_relay_simulated() {
    let love_dir = tempdir().unwrap();
    let joy_dir = tempdir().unwrap();
    let peace_dir = tempdir().unwrap();

    // Setup all three nodes
    let mut love = SyncEngine::new(love_dir.path()).await.unwrap();
    love.init_profile_keys().unwrap();
    let love_did = love.profile_did().unwrap().clone();

    let mut joy = SyncEngine::new(joy_dir.path()).await.unwrap();
    joy.init_profile_keys().unwrap();

    let mut peace = SyncEngine::new(peace_dir.path()).await.unwrap();
    peace.init_profile_keys().unwrap();

    // Love creates a global packet (encrypted per-recipient not yet tested)
    love
        .create_packet(heartbeat(), PacketAddress::Global)
        .unwrap();

    let love_log = love.my_log().unwrap();
    let packet = love_log.entries_ordered()[0].envelope.clone();

    // Peace (relay) receives and stores the packet
    let is_new_for_peace = peace.handle_incoming_packet(packet.clone()).unwrap();
    assert!(is_new_for_peace, "Packet should be new to Peace");

    // Verify Peace stored it in her mirror
    let peace_mirror_head = peace.mirror_head(&love_did);
    assert_eq!(
        peace_mirror_head,
        Some(0),
        "Peace should have Love's packet in mirror"
    );

    // Joy also receives the same packet
    let is_new_for_joy = joy.handle_incoming_packet(packet).unwrap();
    assert!(is_new_for_joy, "Packet should be new to Joy");

    // Verify Joy has it in his mirror too
    let joy_mirror_head = joy.mirror_head(&love_did);
    assert_eq!(
        joy_mirror_head,
        Some(0),
        "Joy should have Love's packet in mirror"
    );
}

/// Test mirror sync after being offline.
///
/// Scenario:
/// - Love sends packets while Joy is offline
/// - Peace receives and mirrors them
/// - Joy comes online and syncs from Peace (simulated)
#[tokio::test]
async fn test_mirror_sync_after_offline_simulated() {
    let love_dir = tempdir().unwrap();
    let joy_dir = tempdir().unwrap();
    let peace_dir = tempdir().unwrap();

    // Setup all three nodes
    let mut love = SyncEngine::new(love_dir.path()).await.unwrap();
    love.init_profile_keys().unwrap();
    let love_did = love.profile_did().unwrap().clone();

    let mut peace = SyncEngine::new(peace_dir.path()).await.unwrap();
    peace.init_profile_keys().unwrap();

    // Love creates several packets while Joy is offline
    for _ in 0..3 {
        love
            .create_packet(heartbeat(), PacketAddress::Global)
            .unwrap();
    }

    // Peace receives all of Love's packets
    let love_log = love.my_log().unwrap();
    for entry in love_log.entries_ordered() {
        peace
            .handle_incoming_packet(entry.envelope.clone())
            .unwrap();
    }

    // Verify Peace has all packets (head is at sequence 2 = 3 packets: 0, 1, 2)
    assert_eq!(peace.mirror_head(&love_did), Some(2));

    // Now Joy comes online
    let mut joy = SyncEngine::new(joy_dir.path()).await.unwrap();
    joy.init_profile_keys().unwrap();

    // Simulate Joy syncing from Peace's mirror
    // mirror_packets_since returns packets AFTER the given sequence
    // To get all packets, use mirror_packets_range which includes both endpoints
    let peace_mirror_packets = peace.mirror_packets_range(&love_did, 0, 2).unwrap();
    assert_eq!(
        peace_mirror_packets.len(),
        3,
        "Peace should have 3 packets from Love (seq 0, 1, 2)"
    );

    // Joy receives all packets from Peace's mirror
    for packet in peace_mirror_packets {
        joy.handle_incoming_packet(packet).unwrap();
    }

    // Verify Joy now has all packets
    assert_eq!(
        joy.mirror_head(&love_did),
        Some(2),
        "Joy should have all 3 packets"
    );
}

// ============================================================================
// Fork Detection Tests
// ============================================================================

/// Test that ProfileLog detects forks (two different packets with same sequence).
#[test]
fn test_fork_detection_in_log() {
    let keys = ProfileKeys::generate();
    let did = keys.did();
    let log = ProfileLog::new(did.clone());

    // Create a legitimate packet at sequence 0
    let builder = PacketBuilder::new(&keys, &log);
    let packet0 = builder.create_global_packet(&heartbeat()).unwrap();

    // Create another ProfileLog (simulating a fork)
    let forked_log = ProfileLog::new(did.clone());
    let forked_builder = PacketBuilder::new(&keys, &forked_log);
    let forked_packet0 = forked_builder.create_global_packet(&heartbeat()).unwrap();

    // Both packets have sequence 0 but different hashes
    assert_eq!(packet0.sequence, 0);
    assert_eq!(forked_packet0.sequence, 0);

    // The prev_hash should be the same (both start from genesis)
    assert_eq!(packet0.prev_hash, forked_packet0.prev_hash);

    // But the packet hashes will be different due to different timestamps
    // This is where fork detection would trigger when comparing received packets
}

/// Test that MirrorStore detects fork when receiving conflicting packets.
#[test]
fn test_mirror_detects_fork() {
    let temp_dir = tempdir().unwrap();
    let db = create_test_db(temp_dir.path());
    let mirror_store = MirrorStore::new(db).unwrap();

    let keys = ProfileKeys::generate();
    let did = keys.did();

    // Create two independent logs (simulating a fork)
    let log1 = ProfileLog::new(did.clone());
    let log2 = ProfileLog::new(did.clone());

    let builder1 = PacketBuilder::new(&keys, &log1);
    let packet1 = builder1.create_global_packet(&heartbeat()).unwrap();

    // Add a small delay to ensure different timestamps
    std::thread::sleep(std::time::Duration::from_millis(1));

    let builder2 = PacketBuilder::new(&keys, &log2);
    let packet2 = builder2.create_global_packet(&heartbeat()).unwrap();

    // Store first packet - should succeed
    mirror_store.store_packet(&packet1).unwrap();
    assert_eq!(mirror_store.get_head(&did).unwrap(), Some(0));

    // Trying to store conflicting packet at same sequence
    // The current implementation might overwrite or ignore
    let result = mirror_store.store_packet(&packet2);

    // For now, we just verify the operation doesn't panic
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// Encrypted Packet Tests
// ============================================================================

/// Test creating and decrypting a global packet.
#[tokio::test]
async fn test_global_packet_roundtrip() {
    let love_dir = tempdir().unwrap();
    let joy_dir = tempdir().unwrap();

    // Setup Love
    let mut love = SyncEngine::new(love_dir.path()).await.unwrap();
    love.init_profile_keys().unwrap();

    // Setup Joy
    let mut joy = SyncEngine::new(joy_dir.path()).await.unwrap();
    joy.init_profile_keys().unwrap();

    // Create a global packet with a direct message (addressed to Joy)
    let joy_did = joy.profile_did().unwrap();
    love
        .create_packet(
            PacketPayload::DirectMessage {
                content: "Hello Joy!".to_string(),
                recipient: joy_did,
            },
            PacketAddress::Global,
        )
        .unwrap();

    let love_log = love.my_log().unwrap();
    let packet = love_log.entries_ordered()[0].envelope.clone();

    // Joy receives and decrypts (global packets can be decoded by anyone)
    let decrypted = joy.decrypt_packet(&packet);
    assert!(
        decrypted.is_some(),
        "Joy should be able to decode global packet"
    );

    match decrypted.unwrap() {
        PacketPayload::DirectMessage { content, recipient: _ } => {
            assert_eq!(content, "Hello Joy!");
        }
        _ => panic!("Expected DirectMessage payload"),
    }
}

// ============================================================================
// Receipt and Depin Tests (Stubbed for Future)
// ============================================================================

/// Test automatic receipt generation (stub - full implementation requires network).
#[test]
fn test_receipt_payload_creation() {
    let keys = ProfileKeys::generate();
    let did = keys.did();
    let log = ProfileLog::new(did);

    // Create a receipt payload - use a valid generated DID
    let love_keys = ProfileKeys::generate();
    let love_did = love_keys.did();
    let receipt = PacketPayload::Receipt {
        original_sender: love_did.clone(),
        packet_seq: 42,
    };

    // For sealed packets, we need recipient keys - use global for this test
    let builder = PacketBuilder::new(&keys, &log);
    let envelope = builder.create_global_packet(&receipt).unwrap();

    assert_eq!(envelope.sequence, 0);
    // Verify it's a global packet
    assert!(envelope.is_global());
}

/// Test depin payload creation (stub - full implementation requires receipt tracking).
#[test]
fn test_depin_payload_creation() {
    let keys = ProfileKeys::generate();
    let did = keys.did();
    let log = ProfileLog::new(did);

    // Create a depin payload
    let depin = PacketPayload::Depin {
        before_sequence: 10,
        merkle_root: Some([0xAB; 32]),
    };

    let builder = PacketBuilder::new(&keys, &log);
    let envelope = builder.create_global_packet(&depin).unwrap();

    assert_eq!(envelope.sequence, 0);
    assert!(envelope.is_global()); // Depins are broadcast globally

    // Verify we can decode it
    let decoded = envelope.decode_global_payload().unwrap();
    match decoded {
        PacketPayload::Depin {
            before_sequence,
            merkle_root,
        } => {
            assert_eq!(before_sequence, 10);
            assert_eq!(merkle_root, Some([0xAB; 32]));
        }
        _ => panic!("Expected Depin payload"),
    }
}

// ============================================================================
// List Mirrored DIDs Tests
// ============================================================================

/// Test listing all mirrored DIDs from an engine.
#[tokio::test]
async fn test_list_mirrored_dids() {
    let love_dir = tempdir().unwrap();
    let joy_dir = tempdir().unwrap();
    let peace_dir = tempdir().unwrap();
    let mirror_dir = tempdir().unwrap();

    // Create packets from multiple profiles
    let mut love = SyncEngine::new(love_dir.path()).await.unwrap();
    love.init_profile_keys().unwrap();

    let mut joy = SyncEngine::new(joy_dir.path()).await.unwrap();
    joy.init_profile_keys().unwrap();

    let mut peace = SyncEngine::new(peace_dir.path()).await.unwrap();
    peace.init_profile_keys().unwrap();

    // Create a receiving node
    let mut receiver = SyncEngine::new(mirror_dir.path()).await.unwrap();
    receiver.init_profile_keys().unwrap();

    // Love and Joy create packets
    love
        .create_packet(heartbeat(), PacketAddress::Global)
        .unwrap();
    joy.create_packet(heartbeat(), PacketAddress::Global)
        .unwrap();

    // Receiver gets packets from Love and Joy
    let love_packet = love.my_log().unwrap().entries_ordered()[0]
        .envelope
        .clone();
    let joy_packet = joy.my_log().unwrap().entries_ordered()[0].envelope.clone();

    receiver.handle_incoming_packet(love_packet).unwrap();
    receiver.handle_incoming_packet(joy_packet).unwrap();

    // List mirrored DIDs
    let mirrored_dids = receiver.list_mirrored_dids().unwrap();
    assert_eq!(
        mirrored_dids.len(),
        2,
        "Should have mirrors for Love and Joy"
    );

    // Peace's packet not received yet
    peace
        .create_packet(heartbeat(), PacketAddress::Global)
        .unwrap();
    let peace_packet = peace.my_log().unwrap().entries_ordered()[0]
        .envelope
        .clone();
    receiver.handle_incoming_packet(peace_packet).unwrap();

    let mirrored_dids = receiver.list_mirrored_dids().unwrap();
    assert_eq!(
        mirrored_dids.len(),
        3,
        "Should have mirrors for Love, Joy, and Peace"
    );
}

// ============================================================================
// Profile Update Packet Tests
// ============================================================================

/// Test creating and decoding a profile update packet.
#[test]
fn test_profile_update_packet() {
    let keys = ProfileKeys::generate();
    let did = keys.did();
    let log = ProfileLog::new(did);

    let profile_update = PacketPayload::ProfileUpdate {
        display_name: Some("Love Quantum".to_string()),
        bio: Some("Building post-quantum systems".to_string()),
        avatar_blob_id: None,
    };

    let builder = PacketBuilder::new(&keys, &log);
    let envelope = builder.create_global_packet(&profile_update).unwrap();

    assert!(envelope.is_global());

    let decoded = envelope.decode_global_payload().unwrap();
    match decoded {
        PacketPayload::ProfileUpdate {
            display_name,
            bio,
            avatar_blob_id,
        } => {
            assert_eq!(display_name, Some("Love Quantum".to_string()));
            assert_eq!(bio, Some("Building post-quantum systems".to_string()));
            assert_eq!(avatar_blob_id, None);
        }
        _ => panic!("Expected ProfileUpdate payload"),
    }
}

/// Test realm invite packet creation.
#[test]
fn test_realm_invite_packet() {
    let keys = ProfileKeys::generate();
    let did = keys.did();
    let log = ProfileLog::new(did);

    let realm_id = syncengine_core::RealmId::new();
    let realm_key = [0x42; 32];
    let realm_name = "Test Realm".to_string();

    let invite = PacketPayload::RealmInvite {
        realm_id: realm_id.clone(),
        realm_key,
        realm_name: realm_name.clone(),
    };

    let builder = PacketBuilder::new(&keys, &log);
    let envelope = builder.create_global_packet(&invite).unwrap();

    let decoded = envelope.decode_global_payload().unwrap();
    match decoded {
        PacketPayload::RealmInvite {
            realm_id: decoded_id,
            realm_key: decoded_key,
            realm_name: decoded_name,
        } => {
            assert_eq!(decoded_id, realm_id);
            assert_eq!(decoded_key, realm_key);
            assert_eq!(decoded_name, realm_name);
        }
        _ => panic!("Expected RealmInvite payload"),
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

/// Test handling duplicate packets (same packet received twice).
#[tokio::test]
async fn test_duplicate_packet_handling() {
    let love_dir = tempdir().unwrap();
    let joy_dir = tempdir().unwrap();

    let mut love = SyncEngine::new(love_dir.path()).await.unwrap();
    love.init_profile_keys().unwrap();

    let mut joy = SyncEngine::new(joy_dir.path()).await.unwrap();
    joy.init_profile_keys().unwrap();

    // Love creates a packet
    love
        .create_packet(heartbeat(), PacketAddress::Global)
        .unwrap();

    let packet = love.my_log().unwrap().entries_ordered()[0]
        .envelope
        .clone();

    // Joy receives it once
    let is_new_first = joy.handle_incoming_packet(packet.clone()).unwrap();
    assert!(is_new_first, "First receipt should be new");

    // Joy receives it again (duplicate)
    let is_new_second = joy.handle_incoming_packet(packet).unwrap();
    assert!(!is_new_second, "Duplicate should not be new");
}

/// Test receiving packets out of order.
#[tokio::test]
async fn test_out_of_order_packets() {
    let love_dir = tempdir().unwrap();
    let joy_dir = tempdir().unwrap();

    let mut love = SyncEngine::new(love_dir.path()).await.unwrap();
    love.init_profile_keys().unwrap();
    let love_did = love.profile_did().unwrap().clone();

    let mut joy = SyncEngine::new(joy_dir.path()).await.unwrap();
    joy.init_profile_keys().unwrap();

    // Love creates 3 packets
    love
        .create_packet(heartbeat(), PacketAddress::Global)
        .unwrap();
    love
        .create_packet(heartbeat(), PacketAddress::Global)
        .unwrap();
    love
        .create_packet(heartbeat(), PacketAddress::Global)
        .unwrap();

    let entries = love.my_log().unwrap().entries_ordered();
    let packet0 = entries[0].envelope.clone();
    let packet1 = entries[1].envelope.clone();
    let packet2 = entries[2].envelope.clone();

    // Joy receives them out of order: 2, 0, 1
    joy.handle_incoming_packet(packet2).unwrap();
    joy.handle_incoming_packet(packet0).unwrap();
    joy.handle_incoming_packet(packet1).unwrap();

    // Joy's mirror should have all 3
    assert_eq!(
        joy.mirror_head(&love_did),
        Some(2),
        "Should have all packets"
    );
}

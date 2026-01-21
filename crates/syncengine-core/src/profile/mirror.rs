//! Mirror storage for profile logs
//!
//! Each peer can store "mirrors" of other profiles' logs. This enables:
//!
//! - **Offline delivery**: Relays store packets for offline recipients
//! - **Redundancy**: Multiple peers can mirror the same profile
//! - **Sync**: New peers can catch up by requesting mirrors from existing peers
//!
//! ## Storage Schema
//!
//! ```text
//! PROFILE_LOGS table: (did_str, sequence) -> PacketEnvelope bytes
//! LOG_HEADS table: did_str -> latest_sequence
//! ```
//!
//! ## Mirror vs Own Log
//!
//! - **Own log**: Append-only, we create packets, sign them
//! - **Mirror**: Read-only copy of another profile's log, we verify packets

use crate::error::SyncError;
use crate::identity::Did;
use parking_lot::RwLock;
use redb::{Database, ReadableTable, TableDefinition};
use std::sync::Arc;
use tracing::{debug, info};

use super::keys::ProfilePublicKeys;
use super::packet::PacketEnvelope;
use super::log::{ProfileLog, ForkDetection};

/// Table for storing packet envelopes
/// Key: "{did}:{sequence}" (e.g., "did:sync:z123:42")
/// Value: Serialized PacketEnvelope bytes
pub(crate) const PROFILE_LOGS_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("profile_logs");

/// Table for storing log head sequence numbers
/// Key: did string
/// Value: Latest sequence number (as 8-byte LE u64)
pub(crate) const LOG_HEADS_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("log_heads");

/// Table for indexing packets by recipient (for offline relay delivery)
/// Key: "{recipient_did}:{packet_hash_hex}"
/// Value: Serialized (sender_did_str, sequence) for lookup in PROFILE_LOGS
pub(crate) const PACKETS_FOR_RECIPIENT_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("packets_for_recipient");

/// Storage for profile mirrors.
///
/// Provides persistence for profile logs, both owned and mirrored from others.
pub struct MirrorStore {
    db: Arc<RwLock<Database>>,
}

impl MirrorStore {
    /// Create a new mirror store with a shared database handle.
    pub fn new(db: Arc<RwLock<Database>>) -> Result<Self, SyncError> {
        // Initialize tables
        {
            let db_guard = db.read();
            let write_txn = db_guard.begin_write()?;
            {
                let _ = write_txn.open_table(PROFILE_LOGS_TABLE)?;
                let _ = write_txn.open_table(LOG_HEADS_TABLE)?;
                let _ = write_txn.open_table(PACKETS_FOR_RECIPIENT_TABLE)?;
            }
            write_txn.commit()?;
        }

        Ok(Self { db })
    }

    /// Get the database handle.
    pub fn db_handle(&self) -> Arc<RwLock<Database>> {
        self.db.clone()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Packet Operations
    // ═══════════════════════════════════════════════════════════════════════

    /// Store a packet envelope.
    ///
    /// This stores the packet and updates the log head if this is the newest packet.
    /// Returns the fork detection result.
    pub fn store_packet(&self, envelope: &PacketEnvelope) -> Result<ForkDetection, SyncError> {
        let did_str = envelope.sender.as_str();
        let sequence = envelope.sequence;
        let key = format_packet_key(did_str, sequence);
        let new_hash = envelope.hash();

        // DIAGNOSTIC: Info level with DID bytes for key comparison debugging
        info!(
            sender_did = %did_str,
            sender_did_len = did_str.len(),
            sender_did_bytes = ?did_str.as_bytes(),
            sequence = sequence,
            storage_key = %key,
            "MirrorStore: STORING packet with key"
        );

        // Serialize envelope
        let bytes = envelope.encode()?;

        let db = self.db.read();
        let write_txn = db.begin_write()?;

        // First, check for existing entry (read operation)
        let existing_data: Option<Vec<u8>> = {
            let logs_table = write_txn.open_table(PROFILE_LOGS_TABLE)?;
            let result = logs_table.get(key.as_str())?.map(|v| v.value().to_vec());
            result
        };

        // Now handle based on whether entry exists
        let fork_result = if let Some(existing_bytes) = existing_data {
            let existing_envelope = PacketEnvelope::decode(&existing_bytes)?;
            let existing_hash = existing_envelope.hash();

            if existing_hash != new_hash {
                // Fork detected!
                ForkDetection::Fork {
                    sequence,
                    existing_hash,
                    conflicting_hash: new_hash,
                }
            } else {
                // Same packet, no change
                ForkDetection::NoFork
            }
        } else {
            // Store new packet (write operations)
            {
                let mut logs_table = write_txn.open_table(PROFILE_LOGS_TABLE)?;
                logs_table.insert(key.as_str(), bytes.as_slice())?;
            }

            // Index packet by recipient for relay forwarding
            // Only index non-global packets (those with specific recipients)
            if !envelope.is_global() {
                let mut recipient_index = write_txn.open_table(PACKETS_FOR_RECIPIENT_TABLE)?;
                for recipient_did in envelope.recipients() {
                    let index_key = format_recipient_index_key(recipient_did.as_str(), &new_hash);
                    let value_data = format_recipient_index_value(did_str, sequence);
                    recipient_index.insert(index_key.as_str(), value_data.as_slice())?;
                    debug!(
                        recipient = %recipient_did,
                        sender = %did_str,
                        sequence = sequence,
                        "Indexed packet for recipient (relay)"
                    );
                }
            }

            // Update head if this is the newest
            {
                let mut heads_table = write_txn.open_table(LOG_HEADS_TABLE)?;
                let current_head = self.get_head_sequence_from_table(&heads_table, did_str)?;
                if current_head.map(|h| sequence > h).unwrap_or(true) {
                    heads_table.insert(did_str, &sequence.to_le_bytes()[..])?;
                }
            }

            ForkDetection::NoFork
        };

        write_txn.commit()?;
        Ok(fork_result)
    }

    /// Load a packet by DID and sequence.
    pub fn get_packet(&self, did: &Did, sequence: u64) -> Result<Option<PacketEnvelope>, SyncError> {
        let key = format_packet_key(did.as_str(), sequence);

        let db = self.db.read();
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(PROFILE_LOGS_TABLE)?;

        match table.get(key.as_str())? {
            Some(v) => {
                let envelope = PacketEnvelope::decode(v.value())?;
                Ok(Some(envelope))
            }
            None => Ok(None),
        }
    }

    /// Get the head sequence for a DID.
    pub fn get_head(&self, did: &Did) -> Result<Option<u64>, SyncError> {
        let db = self.db.read();
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(LOG_HEADS_TABLE)?;

        self.get_head_sequence_from_table(&table, did.as_str())
    }

    /// Get packets in a range (inclusive).
    pub fn get_range(
        &self,
        did: &Did,
        from: u64,
        to: u64,
    ) -> Result<Vec<PacketEnvelope>, SyncError> {
        let db = self.db.read();
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(PROFILE_LOGS_TABLE)?;

        let mut result = Vec::new();
        for seq in from..=to {
            let key = format_packet_key(did.as_str(), seq);
            if let Some(v) = table.get(key.as_str())? {
                let envelope = PacketEnvelope::decode(v.value())?;
                result.push(envelope);
            }
        }

        Ok(result)
    }

    /// Get all packets since a given sequence (exclusive).
    pub fn get_since(&self, did: &Did, from: u64) -> Result<Vec<PacketEnvelope>, SyncError> {
        let did_str = did.as_str();
        // DIAGNOSTIC: Info level with DID bytes for key comparison debugging
        info!(
            query_did = %did_str,
            query_did_len = did_str.len(),
            query_did_bytes = ?did_str.as_bytes(),
            from_sequence = from,
            "MirrorStore: QUERYING packets for DID"
        );

        let head = match self.get_head(did)? {
            Some(h) => {
                info!(
                    query_did = %did_str,
                    head_sequence = h,
                    "MirrorStore: FOUND head sequence {} for DID",
                    h
                );
                h
            }
            None => {
                info!(
                    query_did = %did_str,
                    "MirrorStore: NO HEAD found for DID (no packets stored for this sender)"
                );
                return Ok(Vec::new());
            }
        };

        if from >= head {
            debug!(
                query_did = %did_str,
                from_sequence = from,
                head_sequence = head,
                "MirrorStore: from >= head, returning empty"
            );
            return Ok(Vec::new());
        }

        let packets = self.get_range(did, from + 1, head)?;
        debug!(
            query_did = %did_str,
            from_sequence = from + 1,
            to_sequence = head,
            packets_found = packets.len(),
            "MirrorStore: get_range completed"
        );
        Ok(packets)
    }

    /// Get ALL packets for a DID (inclusive of sequence 0).
    ///
    /// This is different from `get_since(did, 0)` which excludes sequence 0.
    /// Use this when you want all packets including the very first one.
    pub fn get_all(&self, did: &Did) -> Result<Vec<PacketEnvelope>, SyncError> {
        let did_str = did.as_str();
        debug!(
            query_did = %did_str,
            "MirrorStore: get_all - fetching ALL packets including sequence 0"
        );

        match self.get_head(did)? {
            Some(head) => {
                let packets = self.get_range(did, 0, head)?;
                debug!(
                    query_did = %did_str,
                    from_sequence = 0,
                    to_sequence = head,
                    packets_found = packets.len(),
                    "MirrorStore: get_all completed"
                );
                Ok(packets)
            }
            None => {
                debug!(
                    query_did = %did_str,
                    "MirrorStore: get_all - no head found, returning empty"
                );
                Ok(Vec::new())
            }
        }
    }

    /// Delete packets before a given sequence (for garbage collection).
    pub fn delete_before(&self, did: &Did, sequence: u64) -> Result<usize, SyncError> {
        let db = self.db.read();
        let write_txn = db.begin_write()?;

        let deleted = {
            let mut table = write_txn.open_table(PROFILE_LOGS_TABLE)?;
            let mut count = 0;

            for seq in 0..sequence {
                let key = format_packet_key(did.as_str(), seq);
                if table.remove(key.as_str())?.is_some() {
                    count += 1;
                }
            }

            count
        };

        write_txn.commit()?;
        Ok(deleted)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Mirror Operations
    // ═══════════════════════════════════════════════════════════════════════

    /// Load a complete profile log from storage.
    ///
    /// Returns a ProfileLog populated with all stored packets for the given DID.
    pub fn load_log(&self, did: &Did) -> Result<ProfileLog, SyncError> {
        let mut log = ProfileLog::new(did.clone());

        let head = match self.get_head(did)? {
            Some(h) => h,
            None => return Ok(log), // Empty log
        };

        // Load all packets
        let packets = self.get_range(did, 0, head)?;
        for packet in packets {
            // Ignore fork detection here - we're loading from storage
            let _ = log.append(packet);
        }

        Ok(log)
    }

    /// List all DIDs that have stored packets.
    pub fn list_mirrored_dids(&self) -> Result<Vec<Did>, SyncError> {
        let db = self.db.read();
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(LOG_HEADS_TABLE)?;

        let mut dids = Vec::new();
        for entry in table.iter()? {
            let (key, _) = entry?;
            let did = Did::parse(key.value())?;
            dids.push(did);
        }

        Ok(dids)
    }

    /// Count total stored packets across all mirrors.
    pub fn total_packet_count(&self) -> Result<usize, SyncError> {
        let db = self.db.read();
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(PROFILE_LOGS_TABLE)?;

        let count = table.iter()?.count();
        Ok(count)
    }

    /// Delete all packets for a DID.
    pub fn delete_mirror(&self, did: &Did) -> Result<usize, SyncError> {
        let head = match self.get_head(did)? {
            Some(h) => h,
            None => return Ok(0),
        };

        let db = self.db.read();
        let write_txn = db.begin_write()?;

        let deleted = {
            let mut logs_table = write_txn.open_table(PROFILE_LOGS_TABLE)?;
            let mut heads_table = write_txn.open_table(LOG_HEADS_TABLE)?;
            let mut count = 0;

            // Delete all packets
            for seq in 0..=head {
                let key = format_packet_key(did.as_str(), seq);
                if logs_table.remove(key.as_str())?.is_some() {
                    count += 1;
                }
            }

            // Delete head entry
            heads_table.remove(did.as_str())?;

            count
        };

        write_txn.commit()?;
        Ok(deleted)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Relay Operations (for offline delivery)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get all packets addressed to a specific recipient.
    ///
    /// This is used by relay nodes to find packets they're holding for
    /// a peer who just came online. Returns packets from all senders
    /// that were encrypted to this recipient.
    pub fn get_packets_for_recipient(&self, recipient: &Did) -> Result<Vec<PacketEnvelope>, SyncError> {
        let db = self.db.read();
        let read_txn = db.begin_read()?;
        let recipient_index = read_txn.open_table(PACKETS_FOR_RECIPIENT_TABLE)?;
        let logs_table = read_txn.open_table(PROFILE_LOGS_TABLE)?;

        let mut packets = Vec::new();
        let recipient_str = recipient.as_str();
        let prefix = format!("{}:", recipient_str);

        // Scan index for entries starting with "recipient_did:"
        for entry in recipient_index.iter()? {
            let (key, value) = entry?;
            let key_str = key.value();

            // Only process entries for this recipient
            if !key_str.starts_with(&prefix) {
                continue;
            }

            // Parse the stored value to get sender + sequence
            match parse_recipient_index_value(value.value()) {
                Ok((sender_did_str, sequence)) => {
                    // Look up the full packet
                    let packet_key = format_packet_key(&sender_did_str, sequence);
                    if let Ok(Some(packet_data)) = logs_table.get(packet_key.as_str()) {
                        if let Ok(envelope) = PacketEnvelope::decode(packet_data.value()) {
                            packets.push(envelope);
                        }
                    }
                }
                Err(e) => {
                    debug!(error = %e, key = %key_str, "Skipping malformed recipient index entry");
                    continue;
                }
            }
        }

        info!(
            recipient = %recipient_str,
            packets_found = packets.len(),
            "Retrieved packets for recipient (relay delivery)"
        );
        Ok(packets)
    }

    /// Mark a packet as delivered to a recipient (remove from relay index).
    ///
    /// Called after the recipient acknowledges receipt via a Receipt packet.
    /// This prevents re-sending the same packet on future connections.
    pub fn mark_delivered(
        &self,
        recipient: &Did,
        packet_hash: &[u8; 32],
    ) -> Result<(), SyncError> {
        let db = self.db.read();
        let write_txn = db.begin_write()?;

        {
            let mut recipient_index = write_txn.open_table(PACKETS_FOR_RECIPIENT_TABLE)?;
            let index_key = format_recipient_index_key(recipient.as_str(), packet_hash);
            recipient_index.remove(index_key.as_str())?;
            debug!(
                recipient = %recipient,
                hash = %hex::encode(packet_hash),
                "Marked packet as delivered (removed from relay index)"
            );
        }

        write_txn.commit()?;
        Ok(())
    }

    /// Clear all relay entries for a recipient.
    ///
    /// Useful for testing and cleanup. Returns the number of entries removed.
    pub fn clear_recipient_relay_entries(&self, recipient: &Did) -> Result<usize, SyncError> {
        let db = self.db.read();
        let write_txn = db.begin_write()?;

        let count = {
            let mut recipient_index = write_txn.open_table(PACKETS_FOR_RECIPIENT_TABLE)?;
            let recipient_str = recipient.as_str();
            let prefix = format!("{}:", recipient_str);

            // Collect keys to remove (can't modify while iterating)
            let keys_to_remove: Vec<String> = recipient_index
                .iter()?
                .filter_map(|entry| {
                    entry.ok().and_then(|(key, _)| {
                        let key_str = key.value();
                        if key_str.starts_with(&prefix) {
                            Some(key_str.to_string())
                        } else {
                            None
                        }
                    })
                })
                .collect();

            let mut deleted = 0;
            for key in keys_to_remove {
                if recipient_index.remove(key.as_str())?.is_some() {
                    deleted += 1;
                }
            }
            deleted
        };

        write_txn.commit()?;
        Ok(count)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Internal Helpers
    // ═══════════════════════════════════════════════════════════════════════

    fn get_head_sequence_from_table<T: redb::ReadableTable<&'static str, &'static [u8]>>(
        &self,
        table: &T,
        did_str: &str,
    ) -> Result<Option<u64>, SyncError> {
        match table.get(did_str)? {
            Some(v) => {
                let bytes: [u8; 8] = v.value().try_into()
                    .map_err(|_| SyncError::Storage("Invalid head sequence bytes".to_string()))?;
                Ok(Some(u64::from_le_bytes(bytes)))
            }
            None => Ok(None),
        }
    }
}

/// Format a key for the profile logs table.
fn format_packet_key(did_str: &str, sequence: u64) -> String {
    format!("{}:{}", did_str, sequence)
}

/// Format a key for the recipient index table.
/// Key format: "{recipient_did}:{packet_hash_hex}"
fn format_recipient_index_key(recipient_did_str: &str, packet_hash: &[u8; 32]) -> String {
    let hash_hex = hex::encode(packet_hash);
    format!("{}:{}", recipient_did_str, hash_hex)
}

/// Format a value for the recipient index (sender_did + sequence).
/// Binary format: [4 bytes sender len][sender bytes][8 bytes sequence]
fn format_recipient_index_value(sender_did_str: &str, sequence: u64) -> Vec<u8> {
    let sender_bytes = sender_did_str.as_bytes();
    let mut value = Vec::with_capacity(4 + sender_bytes.len() + 8);

    // Sender DID length (4 bytes LE) + sender DID bytes
    value.extend_from_slice(&(sender_bytes.len() as u32).to_le_bytes());
    value.extend_from_slice(sender_bytes);

    // Sequence (8 bytes LE)
    value.extend_from_slice(&sequence.to_le_bytes());

    value
}

/// Parse a recipient index value back into (sender_did, sequence).
fn parse_recipient_index_value(bytes: &[u8]) -> Result<(String, u64), SyncError> {
    if bytes.len() < 12 {
        return Err(SyncError::Storage(
            "Invalid recipient index value length".to_string(),
        ));
    }

    // Parse sender DID length
    let sender_len = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;

    if bytes.len() < 4 + sender_len + 8 {
        return Err(SyncError::Storage(
            "Recipient index value truncated".to_string(),
        ));
    }

    // Parse sender DID
    let sender_bytes = &bytes[4..4 + sender_len];
    let sender_did = String::from_utf8(sender_bytes.to_vec())
        .map_err(|_| SyncError::Storage("Invalid sender DID in recipient index".to_string()))?;

    // Parse sequence
    let seq_offset = 4 + sender_len;
    let seq_bytes: [u8; 8] = bytes[seq_offset..seq_offset + 8]
        .try_into()
        .map_err(|_| SyncError::Storage("Sequence bytes invalid".to_string()))?;
    let sequence = u64::from_le_bytes(seq_bytes);

    Ok((sender_did, sequence))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::keys::ProfileKeys;
    use crate::profile::packet::PacketPayload;
    use tempfile::TempDir;
    use redb::Database;

    fn create_test_store() -> (MirrorStore, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.redb");
        let db = Database::create(&db_path).unwrap();
        let db = Arc::new(RwLock::new(db));
        let store = MirrorStore::new(db).unwrap();
        (store, temp_dir)
    }

    fn create_test_envelope(
        keys: &ProfileKeys,
        sequence: u64,
        prev_hash: [u8; 32],
    ) -> PacketEnvelope {
        let payload = PacketPayload::Heartbeat {
            timestamp: chrono::Utc::now().timestamp_millis(),
        };
        PacketEnvelope::create_global(keys, &payload, sequence, prev_hash)
            .expect("Should create envelope")
    }

    #[test]
    fn test_store_and_get_packet() {
        let (store, _temp) = create_test_store();
        let keys = ProfileKeys::generate();

        let envelope = create_test_envelope(&keys, 0, [0u8; 32]);
        let result = store.store_packet(&envelope).expect("Should store");
        assert_eq!(result, ForkDetection::NoFork);

        let loaded = store.get_packet(&keys.did(), 0)
            .expect("Should load")
            .expect("Should exist");
        assert_eq!(loaded.sequence, 0);
        assert_eq!(loaded.sender, keys.did());
    }

    #[test]
    fn test_get_head() {
        let (store, _temp) = create_test_store();
        let keys = ProfileKeys::generate();

        // Initially no head
        assert!(store.get_head(&keys.did()).unwrap().is_none());

        // Store packet 0
        let envelope0 = create_test_envelope(&keys, 0, [0u8; 32]);
        store.store_packet(&envelope0).unwrap();
        assert_eq!(store.get_head(&keys.did()).unwrap(), Some(0));

        // Store packet 1
        let envelope1 = create_test_envelope(&keys, 1, envelope0.hash());
        store.store_packet(&envelope1).unwrap();
        assert_eq!(store.get_head(&keys.did()).unwrap(), Some(1));
    }

    #[test]
    fn test_get_range() {
        let (store, _temp) = create_test_store();
        let keys = ProfileKeys::generate();

        // Store packets 0-4
        let mut prev_hash = [0u8; 32];
        for seq in 0..5 {
            let envelope = create_test_envelope(&keys, seq, prev_hash);
            prev_hash = envelope.hash();
            store.store_packet(&envelope).unwrap();
        }

        let range = store.get_range(&keys.did(), 1, 3).unwrap();
        assert_eq!(range.len(), 3);
        assert_eq!(range[0].sequence, 1);
        assert_eq!(range[1].sequence, 2);
        assert_eq!(range[2].sequence, 3);
    }

    #[test]
    fn test_get_since() {
        let (store, _temp) = create_test_store();
        let keys = ProfileKeys::generate();

        // Store packets 0-4
        let mut prev_hash = [0u8; 32];
        for seq in 0..5 {
            let envelope = create_test_envelope(&keys, seq, prev_hash);
            prev_hash = envelope.hash();
            store.store_packet(&envelope).unwrap();
        }

        let since = store.get_since(&keys.did(), 2).unwrap();
        assert_eq!(since.len(), 2); // Packets 3 and 4
        assert_eq!(since[0].sequence, 3);
        assert_eq!(since[1].sequence, 4);
    }

    #[test]
    fn test_delete_before() {
        let (store, _temp) = create_test_store();
        let keys = ProfileKeys::generate();

        // Store packets 0-4
        let mut prev_hash = [0u8; 32];
        for seq in 0..5 {
            let envelope = create_test_envelope(&keys, seq, prev_hash);
            prev_hash = envelope.hash();
            store.store_packet(&envelope).unwrap();
        }

        // Delete before sequence 3
        let deleted = store.delete_before(&keys.did(), 3).unwrap();
        assert_eq!(deleted, 3);

        // Verify packets 0, 1, 2 are gone
        assert!(store.get_packet(&keys.did(), 0).unwrap().is_none());
        assert!(store.get_packet(&keys.did(), 1).unwrap().is_none());
        assert!(store.get_packet(&keys.did(), 2).unwrap().is_none());

        // Verify packets 3, 4 still exist
        assert!(store.get_packet(&keys.did(), 3).unwrap().is_some());
        assert!(store.get_packet(&keys.did(), 4).unwrap().is_some());
    }

    #[test]
    fn test_load_log() {
        let (store, _temp) = create_test_store();
        let keys = ProfileKeys::generate();

        // Store packets 0-4
        let mut prev_hash = [0u8; 32];
        for seq in 0..5 {
            let envelope = create_test_envelope(&keys, seq, prev_hash);
            prev_hash = envelope.hash();
            store.store_packet(&envelope).unwrap();
        }

        // Load as ProfileLog
        let log = store.load_log(&keys.did()).unwrap();
        assert_eq!(log.len(), 5);
        assert_eq!(log.head_sequence(), Some(4));
    }

    #[test]
    fn test_list_mirrored_dids() {
        let (store, _temp) = create_test_store();
        let keys1 = ProfileKeys::generate();
        let keys2 = ProfileKeys::generate();
        let keys3 = ProfileKeys::generate();

        // Store packets for multiple DIDs
        store.store_packet(&create_test_envelope(&keys1, 0, [0u8; 32])).unwrap();
        store.store_packet(&create_test_envelope(&keys2, 0, [0u8; 32])).unwrap();
        store.store_packet(&create_test_envelope(&keys3, 0, [0u8; 32])).unwrap();

        let dids = store.list_mirrored_dids().unwrap();
        assert_eq!(dids.len(), 3);
    }

    #[test]
    fn test_total_packet_count() {
        let (store, _temp) = create_test_store();
        let keys = ProfileKeys::generate();

        assert_eq!(store.total_packet_count().unwrap(), 0);

        // Store 5 packets
        let mut prev_hash = [0u8; 32];
        for seq in 0..5 {
            let envelope = create_test_envelope(&keys, seq, prev_hash);
            prev_hash = envelope.hash();
            store.store_packet(&envelope).unwrap();
        }

        assert_eq!(store.total_packet_count().unwrap(), 5);
    }

    #[test]
    fn test_delete_mirror() {
        let (store, _temp) = create_test_store();
        let keys1 = ProfileKeys::generate();
        let keys2 = ProfileKeys::generate();

        // Store packets for two DIDs
        store.store_packet(&create_test_envelope(&keys1, 0, [0u8; 32])).unwrap();
        store.store_packet(&create_test_envelope(&keys1, 1, [1u8; 32])).unwrap();
        store.store_packet(&create_test_envelope(&keys2, 0, [0u8; 32])).unwrap();

        // Delete mirror for keys1
        let deleted = store.delete_mirror(&keys1.did()).unwrap();
        assert_eq!(deleted, 2);

        // Verify keys1 packets are gone
        assert!(store.get_head(&keys1.did()).unwrap().is_none());

        // Verify keys2 packets still exist
        assert!(store.get_packet(&keys2.did(), 0).unwrap().is_some());
    }

    #[test]
    fn test_fork_detection() {
        let (store, _temp) = create_test_store();
        let keys = ProfileKeys::generate();

        // Store first packet
        let envelope1 = create_test_envelope(&keys, 0, [0u8; 32]);
        store.store_packet(&envelope1).unwrap();

        // Try to store different packet at same sequence
        std::thread::sleep(std::time::Duration::from_millis(10));
        let envelope2 = create_test_envelope(&keys, 0, [0u8; 32]);

        let result = store.store_packet(&envelope2).unwrap();
        match result {
            ForkDetection::Fork { sequence, .. } => {
                assert_eq!(sequence, 0);
            }
            _ => panic!("Expected fork detection"),
        }
    }

    #[test]
    fn test_reject_invalid_signature() {
        let (store, _temp) = create_test_store();
        let keys = ProfileKeys::generate();
        let attacker_keys = ProfileKeys::generate();

        // Store a valid packet
        let envelope = create_test_envelope(&keys, 0, [0u8; 32]);
        store.store_packet(&envelope).unwrap();

        // Load and verify with wrong public key should fail
        let log = store.load_log(&keys.did()).unwrap();
        let mut log = log;
        let invalid = log.verify_all(&attacker_keys.public_bundle());
        assert_eq!(invalid.len(), 1);
        assert_eq!(invalid[0], 0);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Relay / Recipient Index Tests
    // ═══════════════════════════════════════════════════════════════════════

    fn create_direct_message_envelope(
        sender_keys: &ProfileKeys,
        recipient_keys: &ProfileKeys,
        sequence: u64,
        prev_hash: [u8; 32],
        content: &str,
    ) -> PacketEnvelope {
        let payload = PacketPayload::DirectMessage {
            content: content.to_string(),
            recipient: recipient_keys.did(),
        };
        PacketEnvelope::create(
            sender_keys,
            &payload,
            &[recipient_keys.public_bundle()],
            sequence,
            prev_hash,
        )
        .expect("Should create envelope")
    }

    #[test]
    fn test_recipient_index_direct_message() {
        let (store, _temp) = create_test_store();
        let sender_keys = ProfileKeys::generate();
        let recipient_keys = ProfileKeys::generate();

        // Create and store a direct message packet
        let envelope = create_direct_message_envelope(
            &sender_keys,
            &recipient_keys,
            0,
            [0u8; 32],
            "Hello, this is a test message!",
        );
        store.store_packet(&envelope).unwrap();

        // Query by recipient - should find the packet
        let packets = store.get_packets_for_recipient(&recipient_keys.did()).unwrap();
        assert_eq!(packets.len(), 1);
        assert_eq!(packets[0].sender, sender_keys.did());
        assert_eq!(packets[0].sequence, 0);
    }

    #[test]
    fn test_recipient_index_multiple_packets() {
        let (store, _temp) = create_test_store();
        let sender_keys = ProfileKeys::generate();
        let recipient_keys = ProfileKeys::generate();

        // Store 3 packets to same recipient
        let mut prev_hash = [0u8; 32];
        for seq in 0..3 {
            let envelope = create_direct_message_envelope(
                &sender_keys,
                &recipient_keys,
                seq,
                prev_hash,
                &format!("Message {}", seq),
            );
            prev_hash = envelope.hash();
            store.store_packet(&envelope).unwrap();
        }

        // Query recipient - should find all 3
        let packets = store.get_packets_for_recipient(&recipient_keys.did()).unwrap();
        assert_eq!(packets.len(), 3);
    }

    #[test]
    fn test_mark_delivered_removes_from_index() {
        let (store, _temp) = create_test_store();
        let sender_keys = ProfileKeys::generate();
        let recipient_keys = ProfileKeys::generate();

        let envelope = create_direct_message_envelope(
            &sender_keys,
            &recipient_keys,
            0,
            [0u8; 32],
            "Test message",
        );
        let packet_hash = envelope.hash();

        // Store it
        store.store_packet(&envelope).unwrap();
        assert_eq!(
            store.get_packets_for_recipient(&recipient_keys.did()).unwrap().len(),
            1
        );

        // Mark as delivered
        store.mark_delivered(&recipient_keys.did(), &packet_hash).unwrap();

        // Should be removed from index
        assert_eq!(
            store.get_packets_for_recipient(&recipient_keys.did()).unwrap().len(),
            0
        );

        // But packet itself should still exist in main log
        assert!(store.get_packet(&sender_keys.did(), 0).unwrap().is_some());
    }

    #[test]
    fn test_global_packets_not_indexed_by_recipient() {
        let (store, _temp) = create_test_store();
        let sender_keys = ProfileKeys::generate();
        let other_keys = ProfileKeys::generate();

        // Create a global (public) packet - these have no specific recipient
        let envelope = create_test_envelope(&sender_keys, 0, [0u8; 32]);
        store.store_packet(&envelope).unwrap();

        // Should NOT appear in recipient index for any DID
        let packets = store.get_packets_for_recipient(&other_keys.did()).unwrap();
        assert_eq!(packets.len(), 0);
    }

    #[test]
    fn test_recipient_index_multiple_senders() {
        let (store, _temp) = create_test_store();
        let sender1_keys = ProfileKeys::generate();
        let sender2_keys = ProfileKeys::generate();
        let recipient_keys = ProfileKeys::generate();

        // Two different senders send to same recipient
        let envelope1 = create_direct_message_envelope(
            &sender1_keys,
            &recipient_keys,
            0,
            [0u8; 32],
            "Message from sender 1",
        );
        let envelope2 = create_direct_message_envelope(
            &sender2_keys,
            &recipient_keys,
            0,
            [0u8; 32],
            "Message from sender 2",
        );

        store.store_packet(&envelope1).unwrap();
        store.store_packet(&envelope2).unwrap();

        // Should find both packets for the recipient
        let packets = store.get_packets_for_recipient(&recipient_keys.did()).unwrap();
        assert_eq!(packets.len(), 2);

        // Verify we got packets from both senders
        let senders: Vec<_> = packets.iter().map(|p| p.sender.clone()).collect();
        assert!(senders.contains(&sender1_keys.did()));
        assert!(senders.contains(&sender2_keys.did()));
    }

    #[test]
    fn test_clear_recipient_relay_entries() {
        let (store, _temp) = create_test_store();
        let sender_keys = ProfileKeys::generate();
        let recipient_keys = ProfileKeys::generate();

        // Store multiple packets
        let mut prev_hash = [0u8; 32];
        for seq in 0..3 {
            let envelope = create_direct_message_envelope(
                &sender_keys,
                &recipient_keys,
                seq,
                prev_hash,
                &format!("Message {}", seq),
            );
            prev_hash = envelope.hash();
            store.store_packet(&envelope).unwrap();
        }

        assert_eq!(
            store.get_packets_for_recipient(&recipient_keys.did()).unwrap().len(),
            3
        );

        // Clear all relay entries for this recipient
        let cleared = store.clear_recipient_relay_entries(&recipient_keys.did()).unwrap();
        assert_eq!(cleared, 3);

        // Index should be empty now
        assert_eq!(
            store.get_packets_for_recipient(&recipient_keys.did()).unwrap().len(),
            0
        );

        // But packets should still exist in main log
        for seq in 0..3 {
            assert!(store.get_packet(&sender_keys.did(), seq).unwrap().is_some());
        }
    }
}

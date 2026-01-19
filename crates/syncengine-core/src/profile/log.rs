//! Append-only log with hash chain
//!
//! Each profile maintains a linear log of packets, linked by hash.
//! This provides:
//!
//! - **Integrity**: Tampering with history is detectable
//! - **Ordering**: Clear sequence of events
//! - **Fork detection**: Multiple packets claiming same sequence = compromise
//!
//! ## Hash Chain Structure
//!
//! ```text
//! ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
//! │  Packet 0   │───▶│  Packet 1   │───▶│  Packet 2   │
//! │  seq: 0     │    │  seq: 1     │    │  seq: 2     │
//! │  prev: 0x0  │    │  prev: H(0) │    │  prev: H(1) │
//! └─────────────┘    └─────────────┘    └─────────────┘
//! ```
//!
//! ## Fork Detection
//!
//! If two packets claim the same sequence number, this indicates either:
//! 1. Key compromise (attacker creating packets)
//! 2. Network partition with unsynchronized clients
//! 3. Bug in the sender's implementation
//!
//! Forks are detected and reported but not automatically resolved.

use crate::error::SyncError;
use crate::identity::Did;

use super::keys::{ProfileKeys, ProfilePublicKeys};
use super::packet::{PacketEnvelope, PacketPayload};

use std::collections::HashMap;

/// Entry in the profile log.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// The packet envelope
    pub envelope: PacketEnvelope,
    /// Computed hash of this entry (for hash chain)
    pub hash: [u8; 32],
    /// Whether this entry has been verified
    pub verified: bool,
}

impl LogEntry {
    /// Create a new log entry from an envelope.
    pub fn new(envelope: PacketEnvelope) -> Self {
        let hash = envelope.hash();
        Self {
            envelope,
            hash,
            verified: false,
        }
    }

    /// Mark this entry as verified.
    pub fn mark_verified(&mut self) {
        self.verified = true;
    }
}

/// Fork detection result.
#[derive(Debug, Clone, PartialEq)]
pub enum ForkDetection {
    /// No fork - the log is linear
    NoFork,
    /// Fork detected at the given sequence number
    Fork {
        /// Sequence number where fork occurred
        sequence: u64,
        /// Hash of the existing entry
        existing_hash: [u8; 32],
        /// Hash of the conflicting entry
        conflicting_hash: [u8; 32],
    },
}

/// Append-only log for a profile.
///
/// Maintains a linear sequence of packets with hash chain integrity.
pub struct ProfileLog {
    /// Owner of this log
    owner: Did,
    /// Entries indexed by sequence number
    entries: HashMap<u64, LogEntry>,
    /// Current head sequence (highest known)
    head_sequence: Option<u64>,
    /// Head hash (hash of highest entry)
    head_hash: [u8; 32],
    /// Detected forks (sequence -> conflicting hashes)
    forks: HashMap<u64, Vec<[u8; 32]>>,
}

impl ProfileLog {
    /// Create a new empty log for a profile.
    pub fn new(owner: Did) -> Self {
        Self {
            owner,
            entries: HashMap::new(),
            head_sequence: None,
            head_hash: [0u8; 32], // Genesis prev_hash
            forks: HashMap::new(),
        }
    }

    /// Get the owner's DID.
    pub fn owner(&self) -> &Did {
        &self.owner
    }

    /// Get the current head sequence number.
    pub fn head_sequence(&self) -> Option<u64> {
        self.head_sequence
    }

    /// Get the current head hash.
    pub fn head_hash(&self) -> [u8; 32] {
        self.head_hash
    }

    /// Get an entry by sequence number.
    pub fn get(&self, sequence: u64) -> Option<&LogEntry> {
        self.entries.get(&sequence)
    }

    /// Get all entries in sequence order.
    pub fn entries_ordered(&self) -> Vec<&LogEntry> {
        let mut seqs: Vec<_> = self.entries.keys().copied().collect();
        seqs.sort();
        seqs.into_iter()
            .filter_map(|seq| self.entries.get(&seq))
            .collect()
    }

    /// Get entries in a range (inclusive).
    pub fn get_range(&self, from: u64, to: u64) -> Vec<&LogEntry> {
        let mut result = Vec::new();
        for seq in from..=to {
            if let Some(entry) = self.entries.get(&seq) {
                result.push(entry);
            }
        }
        result
    }

    /// Get all entries since a given sequence (exclusive).
    pub fn get_since(&self, from: u64) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|(seq, _)| **seq > from)
            .map(|(_, entry)| entry)
            .collect()
    }

    /// Append a packet to the log.
    ///
    /// Returns fork detection result. If a fork is detected, the entry
    /// is still stored but flagged.
    pub fn append(&mut self, envelope: PacketEnvelope) -> Result<ForkDetection, SyncError> {
        // Verify sender matches log owner
        if envelope.sender != self.owner {
            return Err(SyncError::Identity(format!(
                "Packet sender {} doesn't match log owner {}",
                envelope.sender, self.owner
            )));
        }

        let sequence = envelope.sequence;
        let entry = LogEntry::new(envelope);

        // Check for fork
        if let Some(existing) = self.entries.get(&sequence) {
            if existing.hash != entry.hash {
                // Fork detected!
                self.forks
                    .entry(sequence)
                    .or_default()
                    .push(entry.hash);

                return Ok(ForkDetection::Fork {
                    sequence,
                    existing_hash: existing.hash,
                    conflicting_hash: entry.hash,
                });
            }
            // Same packet, no change needed
            return Ok(ForkDetection::NoFork);
        }

        // Verify hash chain (if not the first entry)
        if sequence > 0 {
            if let Some(prev) = self.entries.get(&(sequence - 1)) {
                if entry.envelope.prev_hash != prev.hash {
                    return Err(SyncError::Crypto(format!(
                        "Hash chain broken at sequence {}: expected prev_hash {:?}, got {:?}",
                        sequence,
                        hex::encode(prev.hash),
                        hex::encode(entry.envelope.prev_hash)
                    )));
                }
            }
            // If we don't have the previous entry, we accept this one
            // and will validate the chain when we receive the gap
        }

        // Update head if this is the newest
        match self.head_sequence {
            None => {
                self.head_sequence = Some(sequence);
                self.head_hash = entry.hash;
            }
            Some(head) if sequence > head => {
                self.head_sequence = Some(sequence);
                self.head_hash = entry.hash;
            }
            _ => {}
        }

        self.entries.insert(sequence, entry);
        Ok(ForkDetection::NoFork)
    }

    /// Verify an entry's signature.
    pub fn verify_entry(
        &mut self,
        sequence: u64,
        sender_public: &ProfilePublicKeys,
    ) -> Result<bool, SyncError> {
        let entry = self
            .entries
            .get_mut(&sequence)
            .ok_or_else(|| SyncError::RealmNotFound(format!("No entry at sequence {}", sequence)))?;

        let valid = entry.envelope.verify(sender_public);
        if valid {
            entry.mark_verified();
        }
        Ok(valid)
    }

    /// Verify all entries in the log.
    pub fn verify_all(&mut self, sender_public: &ProfilePublicKeys) -> Vec<u64> {
        let mut invalid = Vec::new();

        for (seq, entry) in self.entries.iter_mut() {
            if entry.envelope.verify(sender_public) {
                entry.mark_verified();
            } else {
                invalid.push(*seq);
            }
        }

        invalid
    }

    /// Check if the log has any detected forks.
    pub fn has_forks(&self) -> bool {
        !self.forks.is_empty()
    }

    /// Get all detected forks.
    pub fn get_forks(&self) -> &HashMap<u64, Vec<[u8; 32]>> {
        &self.forks
    }

    /// Get the number of entries in the log.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the log is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get gaps in the sequence (missing entries).
    pub fn get_gaps(&self) -> Vec<u64> {
        let head = match self.head_sequence {
            Some(h) => h,
            None => return Vec::new(),
        };

        let mut gaps = Vec::new();
        for seq in 0..=head {
            if !self.entries.contains_key(&seq) {
                gaps.push(seq);
            }
        }
        gaps
    }

    /// Check if the hash chain is valid for all entries we have.
    ///
    /// Note: This only checks entries we have, not gaps.
    pub fn validate_chain(&self) -> Result<(), SyncError> {
        let mut seqs: Vec<_> = self.entries.keys().copied().collect();
        seqs.sort();

        for i in 1..seqs.len() {
            let prev_seq = seqs[i - 1];
            let curr_seq = seqs[i];

            // Only validate consecutive entries
            if curr_seq != prev_seq + 1 {
                continue;
            }

            let prev_entry = self.entries.get(&prev_seq).unwrap();
            let curr_entry = self.entries.get(&curr_seq).unwrap();

            if curr_entry.envelope.prev_hash != prev_entry.hash {
                return Err(SyncError::Crypto(format!(
                    "Hash chain broken between {} and {}",
                    prev_seq, curr_seq
                )));
            }
        }

        Ok(())
    }

    /// Delete entries before a sequence (for garbage collection after Depin).
    pub fn delete_before(&mut self, sequence: u64) {
        self.entries.retain(|seq, _| *seq >= sequence);
        self.forks.retain(|seq, _| *seq >= sequence);
    }
}

/// Builder for creating packets in a log.
pub struct PacketBuilder<'a> {
    keys: &'a ProfileKeys,
    log: &'a ProfileLog,
}

impl<'a> PacketBuilder<'a> {
    /// Create a new packet builder.
    pub fn new(keys: &'a ProfileKeys, log: &'a ProfileLog) -> Self {
        Self { keys, log }
    }

    /// Get the next sequence number.
    pub fn next_sequence(&self) -> u64 {
        self.log.head_sequence.map(|s| s + 1).unwrap_or(0)
    }

    /// Get the previous hash (current head hash).
    pub fn prev_hash(&self) -> [u8; 32] {
        self.log.head_hash
    }

    /// Create a packet for specific recipients.
    pub fn create_packet(
        &self,
        payload: &PacketPayload,
        recipients: &[ProfilePublicKeys],
    ) -> Result<PacketEnvelope, SyncError> {
        PacketEnvelope::create(
            self.keys,
            payload,
            recipients,
            self.next_sequence(),
            self.prev_hash(),
        )
    }

    /// Create a global (public) packet.
    pub fn create_global_packet(
        &self,
        payload: &PacketPayload,
    ) -> Result<PacketEnvelope, SyncError> {
        PacketEnvelope::create_global(
            self.keys,
            payload,
            self.next_sequence(),
            self.prev_hash(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_log_new() {
        let keys = ProfileKeys::generate();
        let log = ProfileLog::new(keys.did());

        assert_eq!(log.owner(), &keys.did());
        assert!(log.is_empty());
        assert_eq!(log.head_sequence(), None);
    }

    #[test]
    fn test_log_append_single() {
        let keys = ProfileKeys::generate();
        let mut log = ProfileLog::new(keys.did());

        let envelope = create_test_envelope(&keys, 0, [0u8; 32]);
        let result = log.append(envelope.clone()).expect("Should append");

        assert_eq!(result, ForkDetection::NoFork);
        assert_eq!(log.len(), 1);
        assert_eq!(log.head_sequence(), Some(0));
    }

    #[test]
    fn test_log_append_chain() {
        let keys = ProfileKeys::generate();
        let mut log = ProfileLog::new(keys.did());

        // Append first packet
        let envelope0 = create_test_envelope(&keys, 0, [0u8; 32]);
        log.append(envelope0.clone()).expect("Should append 0");
        let hash0 = envelope0.hash();

        // Append second packet with correct prev_hash
        let envelope1 = create_test_envelope(&keys, 1, hash0);
        log.append(envelope1.clone()).expect("Should append 1");
        let hash1 = envelope1.hash();

        // Append third packet
        let envelope2 = create_test_envelope(&keys, 2, hash1);
        log.append(envelope2).expect("Should append 2");

        assert_eq!(log.len(), 3);
        assert_eq!(log.head_sequence(), Some(2));

        // Validate chain
        log.validate_chain().expect("Chain should be valid");
    }

    #[test]
    fn test_log_fork_detection() {
        let keys = ProfileKeys::generate();
        let mut log = ProfileLog::new(keys.did());

        // Append first packet
        let envelope0 = create_test_envelope(&keys, 0, [0u8; 32]);
        log.append(envelope0.clone()).expect("Should append 0");
        let hash0 = envelope0.hash();

        // Append second packet
        let envelope1 = create_test_envelope(&keys, 1, hash0);
        log.append(envelope1.clone()).expect("Should append 1");

        // Try to append a different packet at same sequence (fork!)
        // Use different timestamp to get different hash
        std::thread::sleep(std::time::Duration::from_millis(10));
        let envelope1_fork = create_test_envelope(&keys, 1, hash0);

        let result = log.append(envelope1_fork).expect("Should handle fork");

        match result {
            ForkDetection::Fork { sequence, .. } => {
                assert_eq!(sequence, 1);
            }
            _ => panic!("Expected fork detection"),
        }

        assert!(log.has_forks());
    }

    #[test]
    fn test_log_wrong_sender() {
        let keys1 = ProfileKeys::generate();
        let keys2 = ProfileKeys::generate();
        let mut log = ProfileLog::new(keys1.did());

        // Try to append packet from different sender
        let envelope = create_test_envelope(&keys2, 0, [0u8; 32]);
        let result = log.append(envelope);

        assert!(result.is_err());
    }

    #[test]
    fn test_log_broken_chain() {
        let keys = ProfileKeys::generate();
        let mut log = ProfileLog::new(keys.did());

        // Append first packet
        let envelope0 = create_test_envelope(&keys, 0, [0u8; 32]);
        log.append(envelope0).expect("Should append 0");

        // Try to append second packet with wrong prev_hash
        let wrong_prev = [99u8; 32];
        let envelope1 = create_test_envelope(&keys, 1, wrong_prev);
        let result = log.append(envelope1);

        assert!(result.is_err());
    }

    #[test]
    fn test_log_get_range() {
        let keys = ProfileKeys::generate();
        let mut log = ProfileLog::new(keys.did());

        let mut prev_hash = [0u8; 32];
        for seq in 0..5 {
            let envelope = create_test_envelope(&keys, seq, prev_hash);
            prev_hash = envelope.hash();
            log.append(envelope).expect("Should append");
        }

        let range = log.get_range(1, 3);
        assert_eq!(range.len(), 3);
        assert_eq!(range[0].envelope.sequence, 1);
        assert_eq!(range[2].envelope.sequence, 3);
    }

    #[test]
    fn test_log_get_since() {
        let keys = ProfileKeys::generate();
        let mut log = ProfileLog::new(keys.did());

        let mut prev_hash = [0u8; 32];
        for seq in 0..5 {
            let envelope = create_test_envelope(&keys, seq, prev_hash);
            prev_hash = envelope.hash();
            log.append(envelope).expect("Should append");
        }

        let since = log.get_since(2);
        assert_eq!(since.len(), 2); // Sequences 3 and 4
    }

    #[test]
    fn test_log_gaps() {
        let keys = ProfileKeys::generate();
        let mut log = ProfileLog::new(keys.did());

        // Append packets 0, 2, 4 (gaps at 1, 3)
        let envelope0 = create_test_envelope(&keys, 0, [0u8; 32]);
        let hash0 = envelope0.hash();
        log.append(envelope0).expect("Should append 0");

        // Skip 1, use hash0 as prev (would be invalid chain but we accept for gaps)
        let envelope2 = create_test_envelope(&keys, 2, hash0);
        let hash2 = envelope2.hash();
        log.entries.insert(2, LogEntry::new(envelope2)); // Direct insert to avoid chain check

        let envelope4 = create_test_envelope(&keys, 4, hash2);
        log.entries.insert(4, LogEntry::new(envelope4));

        // Update head manually since we inserted directly
        log.head_sequence = Some(4);

        let gaps = log.get_gaps();
        assert_eq!(gaps, vec![1, 3]);
    }

    #[test]
    fn test_log_delete_before() {
        let keys = ProfileKeys::generate();
        let mut log = ProfileLog::new(keys.did());

        let mut prev_hash = [0u8; 32];
        for seq in 0..5 {
            let envelope = create_test_envelope(&keys, seq, prev_hash);
            prev_hash = envelope.hash();
            log.append(envelope).expect("Should append");
        }

        assert_eq!(log.len(), 5);

        log.delete_before(3);

        assert_eq!(log.len(), 2); // Only 3 and 4 remain
        assert!(log.get(0).is_none());
        assert!(log.get(2).is_none());
        assert!(log.get(3).is_some());
        assert!(log.get(4).is_some());
    }

    #[test]
    fn test_log_verify_entry() {
        let keys = ProfileKeys::generate();
        let mut log = ProfileLog::new(keys.did());

        let envelope = create_test_envelope(&keys, 0, [0u8; 32]);
        log.append(envelope).expect("Should append");

        let valid = log
            .verify_entry(0, &keys.public_bundle())
            .expect("Should verify");
        assert!(valid);

        let entry = log.get(0).unwrap();
        assert!(entry.verified);
    }

    #[test]
    fn test_packet_builder() {
        let keys = ProfileKeys::generate();
        let recipient_keys = ProfileKeys::generate();
        let mut log = ProfileLog::new(keys.did());

        let builder = PacketBuilder::new(&keys, &log);
        assert_eq!(builder.next_sequence(), 0);
        assert_eq!(builder.prev_hash(), [0u8; 32]);

        // Create and append first packet
        let payload = PacketPayload::Heartbeat {
            timestamp: chrono::Utc::now().timestamp_millis(),
        };
        let envelope = builder
            .create_packet(&payload, &[recipient_keys.public_bundle()])
            .expect("Should create");
        let hash0 = envelope.hash();
        log.append(envelope).expect("Should append");

        // New builder should have updated sequence
        let builder2 = PacketBuilder::new(&keys, &log);
        assert_eq!(builder2.next_sequence(), 1);
        assert_eq!(builder2.prev_hash(), hash0);
    }

    #[test]
    fn test_entries_ordered() {
        let keys = ProfileKeys::generate();
        let mut log = ProfileLog::new(keys.did());

        let mut prev_hash = [0u8; 32];
        for seq in 0..5 {
            let envelope = create_test_envelope(&keys, seq, prev_hash);
            prev_hash = envelope.hash();
            log.append(envelope).expect("Should append");
        }

        let ordered = log.entries_ordered();
        assert_eq!(ordered.len(), 5);
        for (i, entry) in ordered.iter().enumerate() {
            assert_eq!(entry.envelope.sequence, i as u64);
        }
    }
}

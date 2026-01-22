//! Relay message wrapping for store-and-forward delivery.
//!
//! When a direct message to a contact fails (e.g., they're offline), we can
//! send the message through a mutual peer who will store and forward it.
//!
//! ## Architecture
//!
//! ```text
//! Love → Peace (OFFLINE)
//!   │
//!   └──→ Joy (mutual peer, ONLINE)
//!        │
//!        └── Stores encrypted payload indexed by Peace's DID
//!        │
//!        └── When Peace comes online, forwards to Peace
//! ```
//!
//! ## Wire Format
//!
//! RelayWrapper has a magic prefix so relay peers can identify it without
//! decryption. The payload inside is the original encrypted PacketEnvelope
//! bytes - the relay peer never sees the actual message content.

use serde::{Deserialize, Serialize};

/// Magic bytes that identify a relay message wrapper.
/// "RELY" in ASCII.
pub const RELAY_MAGIC: [u8; 4] = [0x52, 0x45, 0x4C, 0x59];

/// Wrapper for relayed messages.
///
/// This struct wraps an encrypted packet for store-and-forward delivery.
/// The relay peer can read the metadata (final_recipient) without decrypting
/// the actual message content in `payload`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayWrapper {
    /// Magic bytes to identify this as a relay message.
    /// Always set to RELAY_MAGIC ("RELY").
    pub magic: [u8; 4],

    /// The DID of the final recipient who should receive this message.
    /// The relay peer uses this to index stored messages and determine
    /// when to forward.
    pub final_recipient: String,

    /// The original sender's DID, for logging/debugging.
    pub original_sender: String,

    /// The original encrypted packet bytes.
    /// This is the serialized PacketEnvelope that was intended for
    /// `final_recipient`. The relay peer cannot decrypt this.
    pub payload: Vec<u8>,

    /// Unix timestamp (milliseconds) when the relay was requested.
    pub timestamp: i64,

    /// Unique ID for deduplication.
    /// Prevents storing the same relayed message multiple times if
    /// sent through multiple relay peers.
    pub relay_id: [u8; 16],
}

impl RelayWrapper {
    /// Create a new relay wrapper for forwarding a packet.
    ///
    /// # Arguments
    ///
    /// * `final_recipient` - DID of who should receive the message
    /// * `original_sender` - DID of who sent the original message
    /// * `payload` - The original encrypted packet bytes
    pub fn new(final_recipient: String, original_sender: String, payload: Vec<u8>) -> Self {
        use rand::RngCore;
        let mut relay_id = [0u8; 16];
        rand::rng().fill_bytes(&mut relay_id);

        Self {
            magic: RELAY_MAGIC,
            final_recipient,
            original_sender,
            payload,
            timestamp: chrono::Utc::now().timestamp_millis(),
            relay_id,
        }
    }

    /// Serialize this wrapper for transmission.
    pub fn to_bytes(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(self)
    }

    /// Deserialize a relay wrapper from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(bytes)
    }

    /// Check if bytes appear to be a relay wrapper (start with magic).
    ///
    /// This is a quick check before attempting full deserialization.
    pub fn is_relay_message(bytes: &[u8]) -> bool {
        bytes.len() >= 4 && bytes[0..4] == RELAY_MAGIC
    }

    /// Validate that this wrapper has valid magic bytes.
    pub fn is_valid(&self) -> bool {
        self.magic == RELAY_MAGIC
    }
}

/// Storage for pending relay messages.
///
/// Relay peers store encrypted messages here until the final recipient
/// comes online and can receive them.
#[derive(Debug, Default)]
pub struct RelayStore {
    /// Messages indexed by recipient DID.
    /// Each recipient may have multiple pending messages.
    pending: std::collections::HashMap<String, Vec<StoredRelay>>,
}

/// A stored relay message awaiting delivery.
#[derive(Debug, Clone)]
pub struct StoredRelay {
    /// The original sender's DID.
    pub original_sender: String,
    /// The encrypted payload to forward.
    pub payload: Vec<u8>,
    /// When the relay request was received.
    pub received_at: i64,
    /// Unique relay ID for deduplication.
    pub relay_id: [u8; 16],
}

impl RelayStore {
    /// Create a new empty relay store.
    pub fn new() -> Self {
        Self {
            pending: std::collections::HashMap::new(),
        }
    }

    /// Store a relay message for later delivery.
    ///
    /// Returns true if the message was stored (new), false if it was a duplicate.
    pub fn store(&mut self, wrapper: &RelayWrapper) -> bool {
        let recipient_msgs = self
            .pending
            .entry(wrapper.final_recipient.clone())
            .or_default();

        // Check for duplicate relay_id
        if recipient_msgs
            .iter()
            .any(|m| m.relay_id == wrapper.relay_id)
        {
            return false;
        }

        recipient_msgs.push(StoredRelay {
            original_sender: wrapper.original_sender.clone(),
            payload: wrapper.payload.clone(),
            received_at: chrono::Utc::now().timestamp_millis(),
            relay_id: wrapper.relay_id,
        });

        tracing::info!(
            recipient = %wrapper.final_recipient,
            sender = %wrapper.original_sender,
            relay_id = ?wrapper.relay_id,
            "Stored relay message for offline recipient"
        );

        true
    }

    /// Get all pending messages for a recipient.
    pub fn get_pending(&self, recipient: &str) -> Vec<StoredRelay> {
        self.pending.get(recipient).cloned().unwrap_or_default()
    }

    /// Take all pending messages for a recipient (removes them from store).
    pub fn take_pending(&mut self, recipient: &str) -> Vec<StoredRelay> {
        self.pending.remove(recipient).unwrap_or_default()
    }

    /// Check if there are any pending messages for a recipient.
    pub fn has_pending(&self, recipient: &str) -> bool {
        self.pending
            .get(recipient)
            .map(|msgs| !msgs.is_empty())
            .unwrap_or(false)
    }

    /// Get count of all pending messages across all recipients.
    pub fn pending_count(&self) -> usize {
        self.pending.values().map(|v| v.len()).sum()
    }

    /// Get count of recipients with pending messages.
    pub fn pending_recipients_count(&self) -> usize {
        self.pending.len()
    }

    /// Remove messages older than the given age (in milliseconds).
    ///
    /// Returns the number of messages removed.
    pub fn expire_old(&mut self, max_age_ms: i64) -> usize {
        let now = chrono::Utc::now().timestamp_millis();
        let cutoff = now - max_age_ms;
        let mut removed = 0;

        for msgs in self.pending.values_mut() {
            let before = msgs.len();
            msgs.retain(|m| m.received_at > cutoff);
            removed += before - msgs.len();
        }

        // Remove empty entries
        self.pending.retain(|_, msgs| !msgs.is_empty());

        if removed > 0 {
            tracing::debug!(removed, "Expired old relay messages");
        }

        removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relay_wrapper_roundtrip() {
        let wrapper = RelayWrapper::new(
            "did:sync:recipient".to_string(),
            "did:sync:sender".to_string(),
            vec![1, 2, 3, 4, 5],
        );

        let bytes = wrapper.to_bytes().unwrap();
        assert!(RelayWrapper::is_relay_message(&bytes));

        let decoded = RelayWrapper::from_bytes(&bytes).unwrap();
        assert!(decoded.is_valid());
        assert_eq!(decoded.final_recipient, "did:sync:recipient");
        assert_eq!(decoded.original_sender, "did:sync:sender");
        assert_eq!(decoded.payload, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_is_relay_message() {
        // Valid relay message starts with "RELY"
        let valid = [0x52, 0x45, 0x4C, 0x59, 0x00, 0x00];
        assert!(RelayWrapper::is_relay_message(&valid));

        // Invalid - wrong magic
        let invalid = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
        assert!(!RelayWrapper::is_relay_message(&invalid));

        // Too short
        let short = [0x52, 0x45];
        assert!(!RelayWrapper::is_relay_message(&short));
    }

    #[test]
    fn test_relay_store_deduplication() {
        let mut store = RelayStore::new();

        let wrapper = RelayWrapper::new(
            "did:sync:recipient".to_string(),
            "did:sync:sender".to_string(),
            vec![1, 2, 3],
        );

        // First store succeeds
        assert!(store.store(&wrapper));
        assert_eq!(store.pending_count(), 1);

        // Duplicate is rejected
        assert!(!store.store(&wrapper));
        assert_eq!(store.pending_count(), 1);
    }

    #[test]
    fn test_relay_store_take_pending() {
        let mut store = RelayStore::new();

        let wrapper1 = RelayWrapper::new(
            "did:sync:alice".to_string(),
            "did:sync:bob".to_string(),
            vec![1, 2, 3],
        );
        let wrapper2 = RelayWrapper::new(
            "did:sync:alice".to_string(),
            "did:sync:charlie".to_string(),
            vec![4, 5, 6],
        );

        store.store(&wrapper1);
        store.store(&wrapper2);
        assert_eq!(store.pending_count(), 2);
        assert!(store.has_pending("did:sync:alice"));

        let pending = store.take_pending("did:sync:alice");
        assert_eq!(pending.len(), 2);
        assert_eq!(store.pending_count(), 0);
        assert!(!store.has_pending("did:sync:alice"));
    }
}

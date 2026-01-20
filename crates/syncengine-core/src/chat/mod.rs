//! Chat conversation management
//!
//! This module provides high-level chat abstractions built on the packet system.
//! It offers a user-friendly API for sending messages, loading conversations,
//! and managing chat history.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │  Chat Layer (this module)                                       │
//! │  - ChatMessage: display-ready message struct                    │
//! │  - Conversation: aggregated message history with a contact      │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  Packet Layer (profile module)                                  │
//! │  - PacketPayload::DirectMessage: encrypted message format       │
//! │  - PacketEnvelope: signed, encrypted container                  │
//! │  - MirrorStore: persistent packet storage                       │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  Transport Layer (sync module)                                  │
//! │  - iroh-gossip: P2P message delivery via 1:1 contact topics     │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Usage
//!
//! Chat functionality is accessed through [`SyncEngine`] methods:
//!
//! ```ignore
//! // Send a message
//! let seq = engine.send_message("did:sync:friend", "Hello!").await?;
//!
//! // Get conversation with a contact
//! let conversation = engine.get_conversation("did:sync:friend")?;
//! for msg in conversation.messages() {
//!     println!("{}: {}", msg.display_sender(), msg.content);
//! }
//!
//! // List all conversations
//! let conversations = engine.list_conversations()?;
//! for convo in conversations {
//!     println!("{}: {} messages", convo.display_name(), convo.len());
//! }
//! ```
//!
//! # Message Flow
//!
//! **Sending:**
//! 1. `send_message()` creates a `PacketPayload::DirectMessage`
//! 2. Packet is encrypted for the recipient using sealed box
//! 3. Packet is broadcast via the 1:1 contact gossip topic
//!
//! **Receiving:**
//! 1. Packets arrive via contact topic subscription
//! 2. `handle_incoming_packet()` stores in MirrorStore
//! 3. `get_conversation()` loads and decrypts messages for display

mod conversation;
mod message;

pub use conversation::Conversation;
pub use message::ChatMessage;

use crate::identity::Did;
use crate::profile::{PacketEnvelope, PacketPayload};
use crate::types::peer::Peer;

/// Extract a ChatMessage from a PacketEnvelope if it contains a DirectMessage.
///
/// # Arguments
///
/// * `envelope` - The packet envelope to extract from
/// * `payload` - The decrypted payload (if available)
/// * `my_did` - Our DID (to determine if message is ours)
/// * `sender_name` - Optional display name for the sender
///
/// # Returns
///
/// Some(ChatMessage) if the payload is a DirectMessage, None otherwise.
pub fn extract_chat_message(
    envelope: &PacketEnvelope,
    payload: &PacketPayload,
    my_did: &str,
    sender_name: Option<String>,
) -> Option<ChatMessage> {
    match payload {
        PacketPayload::DirectMessage { content, recipient: _ } => {
            let is_mine = envelope.sender.as_str() == my_did;
            Some(ChatMessage::new(
                envelope.sender.as_str().to_string(),
                sender_name,
                content.clone(),
                envelope.timestamp,
                envelope.sequence,
                is_mine,
            ))
        }
        _ => None,
    }
}

/// Load a conversation from stored packets.
///
/// This is a helper function that aggregates messages from:
/// 1. Packets we received from the contact (stored in MirrorStore)
/// 2. Packets we sent to the contact (stored in our ProfileLog)
///
/// # Arguments
///
/// * `contact_did` - The contact's DID
/// * `contact_name` - Optional display name
/// * `received_packets` - Packets received from the contact
/// * `sent_packets` - Packets we sent (from our log)
/// * `my_did` - Our DID
/// * `decrypt_fn` - Function to decrypt packet payloads
///
/// # Returns
///
/// A Conversation containing all messages with this contact.
pub fn build_conversation<F>(
    contact_did: &str,
    contact_name: Option<String>,
    received_packets: Vec<PacketEnvelope>,
    sent_packets: Vec<PacketEnvelope>,
    my_did: &str,
    decrypt_fn: F,
) -> Conversation
where
    F: Fn(&PacketEnvelope) -> Option<PacketPayload>,
{
    let mut conversation = Conversation::new(contact_did.to_string(), contact_name.clone());

    // Add received messages
    for envelope in received_packets {
        if let Some(payload) = decrypt_fn(&envelope) {
            if let Some(msg) = extract_chat_message(&envelope, &payload, my_did, contact_name.clone()) {
                conversation.add_message(msg);
            }
        }
    }

    // Add sent messages (from our log, addressed to this contact)
    // Note: We check the payload's recipient field rather than envelope metadata
    // because Individual packets now use topic-level privacy (create_global) rather
    // than sealed box encryption, so is_addressed_to() won't work.
    for envelope in sent_packets {
        if let Some(payload) = decrypt_fn(&envelope) {
            // Check if this DirectMessage was intended for this contact
            if let PacketPayload::DirectMessage { ref recipient, .. } = payload {
                if recipient.as_str() == contact_did {
                    if let Some(msg) = extract_chat_message(&envelope, &payload, my_did, None) {
                        conversation.add_message(msg);
                    }
                }
            }
        }
    }

    conversation
}

/// Get a display name for a contact.
///
/// Looks up the peer by DID and returns their display name if available.
/// Falls back to nickname if profile display_name is not set or empty.
pub fn get_contact_display_name(peer: Option<&Peer>) -> Option<String> {
    peer.and_then(|p| {
        // Try profile display_name first (if non-empty), then nickname
        p.profile
            .as_ref()
            .map(|profile| profile.display_name.clone())
            .filter(|name| !name.is_empty())
            .or_else(|| p.nickname.clone())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::ProfileKeys;

    fn create_test_dm_envelope(
        sender_keys: &ProfileKeys,
        content: &str,
        recipient: Did,
        sequence: u64,
    ) -> (PacketEnvelope, PacketPayload) {
        let payload = PacketPayload::DirectMessage {
            content: content.to_string(),
            recipient,
        };
        let envelope = PacketEnvelope::create_global(sender_keys, &payload, sequence, [0u8; 32])
            .expect("Should create envelope");
        (envelope, payload)
    }

    #[test]
    fn test_extract_chat_message_from_dm() {
        let keys = ProfileKeys::generate();
        let recipient_keys = ProfileKeys::generate();
        let (envelope, payload) = create_test_dm_envelope(&keys, "Hello!", recipient_keys.did(), 1);

        let msg = extract_chat_message(&envelope, &payload, "other_did", Some("Alice".to_string()));

        assert!(msg.is_some());
        let msg = msg.unwrap();
        assert_eq!(msg.content, "Hello!");
        assert_eq!(msg.sender_name, Some("Alice".to_string()));
        assert_eq!(msg.sequence, 1);
        assert!(!msg.is_mine);
    }

    #[test]
    fn test_extract_chat_message_is_mine() {
        let keys = ProfileKeys::generate();
        let recipient_keys = ProfileKeys::generate();
        let my_did = keys.did().as_str().to_string();
        let (envelope, payload) = create_test_dm_envelope(&keys, "My message", recipient_keys.did(), 1);

        let msg = extract_chat_message(&envelope, &payload, &my_did, None);

        assert!(msg.is_some());
        assert!(msg.unwrap().is_mine);
    }

    #[test]
    fn test_extract_chat_message_non_dm_returns_none() {
        let keys = ProfileKeys::generate();
        let payload = PacketPayload::Heartbeat {
            timestamp: chrono::Utc::now().timestamp_millis(),
        };
        let envelope = PacketEnvelope::create_global(&keys, &payload, 1, [0u8; 32])
            .expect("Should create envelope");

        let msg = extract_chat_message(&envelope, &payload, "other_did", None);
        assert!(msg.is_none());
    }

    #[test]
    fn test_build_conversation() {
        let friend_keys = ProfileKeys::generate();
        let my_keys = ProfileKeys::generate();
        let my_did_obj = my_keys.did();
        let friend_did_obj = friend_keys.did();
        let my_did = my_did_obj.as_str().to_string();
        let friend_did = friend_did_obj.as_str().to_string();

        // Create received packets (from friend, addressed to us)
        let (recv1, _) = create_test_dm_envelope(&friend_keys, "Hello from friend", my_did_obj.clone(), 1);
        let (recv2, _) = create_test_dm_envelope(&friend_keys, "How are you?", my_did_obj.clone(), 2);

        // Create sent packets (from us, addressed to friend)
        // The recipient field allows us to track who the message was for
        let (sent1, _) = create_test_dm_envelope(&my_keys, "Hi back!", friend_did_obj.clone(), 1);

        let conversation = build_conversation(
            &friend_did,
            Some("Friend".to_string()),
            vec![recv1, recv2],
            vec![sent1],
            &my_did,
            |envelope| {
                // Simple decrypt for global packets
                envelope.decode_global_payload().ok()
            },
        );

        assert_eq!(conversation.contact_did, friend_did);
        assert_eq!(conversation.contact_name, Some("Friend".to_string()));
        // With the recipient field in DirectMessage, sent messages are now properly tracked
        assert_eq!(conversation.len(), 3); // 2 received + 1 sent

        // Verify message ownership is correct
        let messages = conversation.messages();
        let mine_count = messages.iter().filter(|m| m.is_mine).count();
        let theirs_count = messages.iter().filter(|m| !m.is_mine).count();
        assert_eq!(mine_count, 1);
        assert_eq!(theirs_count, 2);
    }

    #[test]
    fn test_build_conversation_empty() {
        let conversation = build_conversation(
            "did:sync:friend",
            None,
            vec![],
            vec![],
            "did:sync:me",
            |_| None,
        );

        assert!(conversation.is_empty());
        assert_eq!(conversation.contact_did, "did:sync:friend");
    }
}

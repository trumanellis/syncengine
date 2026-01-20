//! Messaging components for chat and direct messaging
//!
//! This module provides UI components for the chat interface:
//! - [`MessagesList`] - List of received messages (legacy)
//! - [`MessageCompose`] - Modal for composing messages
//! - [`MessageBubble`] - Chat-style message bubble
//! - [`MessageInput`] - Inline message input bar
//! - [`ConversationView`] - Full chat view for a contact

mod conversation_view;
mod message_bubble;
mod message_compose;
mod message_input;
mod messages_list;

pub use conversation_view::ConversationView;
pub use message_bubble::{ChatBubbleMessage, MessageBubble, MessageBubbleGroup};
pub use message_compose::MessageCompose;
pub use message_input::MessageInput;
pub use messages_list::{MessagesList, ReceivedMessage};

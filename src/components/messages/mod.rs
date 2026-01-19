//! Messaging components (renamed from packets)

mod message_compose;
mod messages_list;

pub use message_compose::MessageCompose;
pub use messages_list::{MessagesList, ReceivedMessage};

//! Gossip-based synchronization layer
//!
//! Uses iroh-gossip for multi-peer broadcast sync.
//!
//! ## Overview
//!
//! The sync module provides the networking layer for Synchronicity Engine,
//! enabling P2P communication between nodes using iroh-gossip topics.
//! Each realm (shared task list) maps to a gossip topic where all members
//! broadcast and receive changes.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │  GossipSync                                                     │
//! │  ├── Endpoint (QUIC transport, NAT traversal)                  │
//! │  ├── Gossip (topic-based pub/sub)                              │
//! │  └── Router (protocol multiplexing)                            │
//! │                                                                 │
//! │  TopicHandle (per-realm connection)                            │
//! │  ├── sender (broadcast messages)                               │
//! │  └── receiver (incoming messages)                              │
//! │                                                                 │
//! │  SyncManager (background sync orchestration)                   │
//! │  ├── Manages multiple realm syncs concurrently                 │
//! │  ├── Tracks per-realm SyncStatus                               │
//! │  └── Emits SyncEvents for UI updates                           │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Protocol
//!
//! The sync protocol uses four message types:
//!
//! - **Announce**: Broadcast document heads for comparison
//! - **SyncRequest**: Request full document when behind
//! - **SyncResponse**: Send full document state
//! - **Changes**: Broadcast incremental updates
//!
//! ## Usage
//!
//! ```ignore
//! // Create gossip node
//! let gossip = GossipSync::new().await?;
//!
//! // Get our ID for sharing with peers
//! let my_id = gossip.endpoint_id();
//!
//! // Subscribe to a realm topic
//! let topic = TopicId::from_bytes(realm_id.as_bytes());
//! let handle = gossip.subscribe(topic, bootstrap_peers).await?;
//!
//! // Broadcast a sync message
//! let msg = SyncMessage::Changes { realm_id, data: changes };
//! handle.broadcast(msg.encode()?).await?;
//!
//! // Receive messages
//! while let Some(gossip_msg) = handle.recv().await {
//!     let sync_msg = SyncMessage::decode(&gossip_msg.content)?;
//!     // Handle sync message...
//! }
//! ```

pub mod contact_handler;
pub mod contact_manager;
pub mod contact_protocol;
pub mod envelope;
pub mod events;
pub mod gossip;
pub mod manager;
pub mod packet_protocol;
pub mod profile_pinning;
pub mod profile_protocol;
pub mod protocol;

pub use contact_handler::ContactProtocolHandler;
pub use contact_manager::{ContactEvent, ContactManager};
pub use contact_protocol::{
    derive_contact_key, derive_contact_topic, ContactMessage, CONTACT_ALPN,
};
pub use envelope::{SyncEnvelope, ENVELOPE_VERSION};
pub use events::{NetworkDebugInfo, SyncEvent, SyncStatus};
pub use gossip::{GossipMessage, GossipSync, TopicEvent, TopicHandle, TopicReceiver, TopicSender};
pub use manager::SyncManager;
pub use profile_pinning::{
    derive_profile_topic, global_profile_topic, ProfileAction, ProfileGossipMessage,
    ProfileMessageHandler,
};
pub use profile_protocol::{ProfileMessage, ProfileProtocolHandler, PublicProfile, PROFILE_ALPN};
pub use packet_protocol::{CombinedMessage, MessageType, PacketSyncMessage, PacketWireMessage};
pub use protocol::{SyncMessage, WireMessage};

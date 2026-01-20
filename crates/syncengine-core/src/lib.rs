//! Synchronicity Engine Core Library
//!
//! P2P task sharing with gossip-based sync and Automerge CRDTs.
//!
//! ## Overview
//!
//! Synchronicity Engine is a censorship-resistant, local-first, peer-to-peer
//! task sharing application implementing a sacred gifting economy. Users create
//! "realms" (shared task lists), invite others via QR codes, and tasks
//! synchronize automatically without central servers.
//!
//! ## Core Principles
//!
//! - **Local-first**: App works fully offline; sync when connected
//! - **Censorship-resistant**: No mandatory coordination server
//! - **Gossip-based sync**: All P2P sync via iroh-gossip topics
//!
//! ## Quick Start
//!
//! ```ignore
//! use syncengine_core::SyncEngine;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let engine = SyncEngine::new("~/.syncengine/data").await?;
//!
//!     // Create a realm
//!     let realm_id = engine.create_realm("My Tasks").await?;
//!
//!     // Add tasks
//!     engine.add_task(&realm_id, "Build solar dehydrator").await?;
//!     engine.add_task(&realm_id, "Plant garden").await?;
//!
//!     // List tasks
//!     for task in engine.list_tasks(&realm_id).await? {
//!         println!("{}: {}", if task.completed { "✓" } else { "○" }, task.title);
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod blobs;
pub mod chat;
pub mod crypto;
pub mod engine;
pub mod error;
pub mod identity;
pub mod invite;
pub mod peers;
pub mod profile;
pub mod realm;
pub mod storage;
pub mod sync;
pub mod types;

// Re-exports
pub use blobs::{BlobManager, BlobProtocolHandler};
pub use crypto::RealmCrypto;
pub use engine::{NetworkStats, NodeInfo, StartupSyncResult, SyncEngine};
pub use error::SyncError;
pub use identity::{Did, HybridKeypair, HybridPublicKey, HybridSignature};
pub use invite::{InviteTicket, NodeAddrBytes};
// Legacy peer types (deprecated in favor of unified Peer type)
pub use peers::{PeerInfo, PeerRegistry};
// Re-export from types module (the unified version)
pub use types::peer::{ContactDetails, Peer, PeerSource, PeerStatus};
pub use realm::RealmDoc;
pub use storage::{PinnerInfo, PinningConfig, Storage};
pub use sync::{
    ContactEvent, GossipMessage, GossipSync, NetworkDebugInfo, SyncEnvelope, SyncEvent,
    SyncManager, SyncMessage, SyncStatus, TopicHandle, WireMessage, ENVELOPE_VERSION,
};
pub use types::*;

// Chat module
pub use chat::{ChatMessage, Conversation};

// Profile packet layer (Indra's Network)
pub use profile::{
    derive_profile_packet_topic, derive_realm_packet_topic, ForkDetection, HybridKeyExchange,
    LogEntry, MirrorStore, PacketAddress, PacketBuilder, PacketEnvelope, PacketPayload,
    PacketRoute, ProfileKeys, ProfileLog, ProfilePublicKeys, ProfileTopicTracker, SealedBox,
    SealedKey,
};

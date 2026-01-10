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

pub mod crypto;
pub mod engine;
pub mod error;
pub mod identity;
pub mod invite;
pub mod realm;
pub mod storage;
pub mod sync;
pub mod types;

// Re-exports
pub use crypto::RealmCrypto;
pub use engine::{NodeInfo, SyncEngine};
pub use error::SyncError;
pub use identity::{Did, HybridKeypair, HybridPublicKey, HybridSignature};
pub use invite::{InviteTicket, NodeAddrBytes};
pub use realm::RealmDoc;
pub use storage::Storage;
pub use sync::{
    GossipMessage, GossipSync, SyncEnvelope, SyncEvent, SyncManager, SyncMessage, SyncStatus,
    TopicHandle, WireMessage, ENVELOPE_VERSION,
};
pub use types::*;

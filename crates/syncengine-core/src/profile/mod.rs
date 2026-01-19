//! Profile Packet Layer - Indra's Network
//!
//! This module implements a **packet-based communication layer** alongside
//! the existing Automerge realm sync. It provides:
//!
//! - **Append-only logs**: Each profile maintains a hash-chained log of packets
//! - **Per-recipient encryption**: Sealed boxes with hybrid X25519 + ML-KEM
//! - **Automatic receipts**: Recipients acknowledge packets, enabling garbage collection
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │  REALMS (Automerge CRDTs)          │  PROFILES (Packet Logs)   │
//! │  ─────────────────────────         │  ─────────────────────    │
//! │  • Collaborative task documents    │  • Append-only signed log │
//! │  • Synced via gossip topics        │  • Per-recipient encryption│
//! │  • Unchanged from current impl     │  • Mirrors stored by peers │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Cryptographic Stack
//!
//! | Purpose | Algorithm |
//! |---------|-----------|
//! | Signing | ML-DSA-65 + Ed25519 (existing hybrid) |
//! | Key Exchange | X25519 + ML-KEM-768 (new hybrid) |
//! | Symmetric Encryption | ChaCha20-Poly1305 (existing) |
//!
//! ## Example
//!
//! ```ignore
//! use syncengine_core::profile::{ProfileKeys, PacketPayload, PacketAddress};
//!
//! // Generate profile keys (signing + key exchange)
//! let keys = ProfileKeys::generate();
//!
//! // Create a packet
//! let payload = PacketPayload::DirectMessage {
//!     content: "Hello!".to_string(),
//! };
//!
//! // Create and sign an envelope addressed to specific recipients
//! let envelope = keys.create_packet(payload, &[recipient_did])?;
//! ```

mod keys;
mod log;
mod mirror;
mod packet;
mod sealed;
mod topic;

// Re-exports
pub use keys::{ProfileKeys, ProfilePublicKeys};
pub use log::{ProfileLog, LogEntry, ForkDetection, PacketBuilder};
pub use mirror::MirrorStore;
pub use packet::{PacketEnvelope, PacketPayload, PacketAddress};
pub use sealed::{SealedBox, SealedKey, HybridKeyExchange};
pub use topic::{
    derive_profile_packet_topic, derive_realm_packet_topic, PacketRoute, ProfileTopicTracker,
};

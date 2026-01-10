# Synchronicity Engine v2: Complete Specification

> **Purpose**: Single source of truth for Claude Code to build Synchronicity Engine from scratch
> **Approach**: TDD, CLI-first, Dioxus-ready architecture
> **Foundation**: `iroh-automerge` patterns for proven P2P sync
> **Version**: 2.0 — Clean Rewrite
> **Date**: January 2026

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [Architecture](#2-architecture)
3. [Development Phases](#3-development-phases)
4. [Phase 1: P2P Foundation (CLI)](#4-phase-1-p2p-foundation-cli)
5. [Phase 2: Core Engine](#5-phase-2-core-engine)
6. [Phase 3: Dioxus UI](#6-phase-3-dioxus-ui)
7. [Sacred Design System](#7-sacred-design-system)
8. [Testing Strategy](#8-testing-strategy)
9. [Security Model](#9-security-model)
10. [API Reference](#10-api-reference)

---

## 1. Project Overview

### What We're Building

Synchronicity Engine is a **peer-to-peer task sharing application** implementing a sacred gifting economy. Users create "realms" (shared task lists), invite others via QR codes or links, and tasks synchronize automatically across all participants without any central server.

### Core Principles

| Principle | Implementation |
|-----------|----------------|
| **Local-first** | Works offline, syncs when connected |
| **Censorship-resistant** | No mandatory coordination server |
| **End-to-end encrypted** | Even relay servers see only ciphertext |
| **Quantum-secure identity** | Hybrid ML-DSA-65 + Ed25519 signatures |
| **Effortless sharing** | One tap to join via invite link |

### Why Rewrite

The previous Tauri + Svelte implementation had a broken P2P layer. Rather than debug while juggling two languages and IPC serialization, we start fresh from `iroh-automerge` (known working P2P) and build up incrementally with TDD.

### Technology Stack

```
┌─────────────────────────────────────────────────────────────┐
│                         Dioxus UI                          │
│            (RSX components, signals, routing)              │
├─────────────────────────────────────────────────────────────┤
│                       Core Engine                          │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐  │
│  │   Realms    │ │  Identity   │ │       Crypto        │  │
│  │ (Automerge) │ │(ML-DSA+Ed25)│ │(ChaCha20-Poly1305)  │  │
│  └─────────────┘ └─────────────┘ └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                       Sync Layer                           │
│  ┌─────────────────────────────────────────────────────┐  │
│  │  iroh-automerge-repo pattern:                        │  │
│  │  • Iroh Endpoint (QUIC, NAT traversal, relay)       │  │
│  │  • iroh-gossip (per-realm topic swarms)             │  │
│  │  • Automerge sync protocol per document             │  │
│  └─────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                       Storage                              │
│  ┌─────────────────────────────────────────────────────┐  │
│  │              redb (embedded key-value)              │  │
│  └─────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Foundation: iroh-automerge-repo

We base our sync layer on the `iroh-automerge-repo` example from n0-computer, NOT the simpler `iroh-automerge`. Key differences:

| iroh-automerge (simple) | iroh-automerge-repo (our foundation) |
|-------------------------|--------------------------------------|
| Single document sync | Repository of multiple documents |
| Point-to-point streams | Gossip-based broadcast |
| Two peers only | Multi-peer swarm per topic |
| Manual connection | Topic-based peer discovery |

The repo pattern maps perfectly to our model:
- **Realm** = Automerge document + gossip topic
- **RealmId** = TopicId (blake3 hash)
- **Invite** = Topic + bootstrap peers + encryption key

---

## 2. Architecture

### Working Directory Structure

Claude Code operates from a parent directory containing these projects:

```
workspace/                        # Claude Code's working directory
│
├── iroh-examples/               # REFERENCE: Iroh P2P patterns
│   ├── iroh-automerge/         # Simple two-peer sync (reference only)
│   ├── iroh-automerge-repo/    # ★ P2P FOUNDATION - gossip-based sync
│   ├── iroh-gossip-chat/       # Chat example (gossip patterns)
│   └── ...
│
├── syncengine-v0/               # REFERENCE: Previous UI implementation
│   └── (Svelte frontend)       # ★ AESTHETIC REFERENCE - match this style!
│                               # Cyber-mystical terminal, sacred geometry
│                               # See DESIGN_SYSTEM.md for extracted patterns
│
├── syncengine-tauri/            # OLD PROJECT: Working Rust code to copy
│   └── src-tauri/src/
│       ├── identity.rs         # ✓ ML-DSA-65 + Ed25519 (TESTED, COPY)
│       ├── crypto.rs           # ✓ ChaCha20-Poly1305 (TESTED, COPY)
│       ├── storage.rs          # ✓ redb patterns (TESTED, COPY)
│       ├── types.rs            # ✓ Core types (COPY)
│       └── sync/               # ✗ BROKEN — do not use
│
└── syncengine/                  # NEW PROJECT: Build here
    ├── Cargo.toml              # Workspace root
    ├── Dioxus.toml             # Dioxus CLI config (Phase 3)
    ├── CLAUDE.md               # Worker instructions
    ├── DESIGN_SYSTEM.md        # ★ UI/UX patterns from syncengine-v0
    │
    ├── crates/
    │   ├── syncengine-core/    # Core library (all business logic)
    │   │   ├── Cargo.toml
    │   │   └── src/
    │   │       ├── lib.rs      # Public API
    │   │       ├── engine.rs   # SyncEngine struct
    │   │       ├── realm.rs    # Realm + intention management
    │   │       ├── identity.rs # ← Copied from syncengine-tauri
    │   │       ├── crypto.rs   # ← Copied from syncengine-tauri
    │   │       ├── storage.rs  # ← Copied from syncengine-tauri
    │   │       ├── sync/
    │   │       │   ├── mod.rs
    │   │       │   ├── gossip.rs    # ← from iroh-automerge-repo
    │   │       │   └── protocol.rs  # ← from iroh-automerge-repo
    │   │       └── types.rs    # ← Copied from syncengine-tauri
    │   │
    │   └── syncengine-cli/     # CLI binary (thin wrapper)
    │       ├── Cargo.toml
    │       └── src/main.rs
    │
    ├── src/                    # Dioxus app (Phase 3)
    │   ├── main.rs
    │   ├── app.rs
    │   ├── theme/              # Color, typography tokens
    │   └── components/         # UI components
    │
    ├── assets/
    │   ├── style.css           # Global styles
    │   └── seed-of-life.svg    # Sacred geometry pattern
    │
    └── tests/
        ├── p2p_integration.rs  # Gossip sync tests
        └── cli_scenarios.rs    # CLI workflow tests
```

### Code Reuse Strategy

**Copy these files from `syncengine-tauri` and modify in place:**
```bash
# These modules have passing tests - copy them directly
cp ../syncengine-tauri/src-tauri/src/identity.rs  crates/syncengine-core/src/
cp ../syncengine-tauri/src-tauri/src/crypto.rs    crates/syncengine-core/src/
cp ../syncengine-tauri/src-tauri/src/storage.rs   crates/syncengine-core/src/
cp ../syncengine-tauri/src-tauri/src/types.rs     crates/syncengine-core/src/

# Then modify imports and remove Tauri-specific code
```

**Study these in `iroh-examples` for gossip patterns:**
```bash
# Read and adapt patterns - don't copy directly (different structure)
../iroh-examples/iroh-automerge-repo/src/main.rs  # Gossip + Automerge patterns
../iroh-examples/iroh-gossip-chat/src/main.rs     # Topic subscription patterns
```

**DO NOT copy from `syncengine-tauri`:**
- `sync/` module (broken)
- `network.rs` (broken)  
- `commands.rs` (Tauri IPC handlers - not needed)
- `lib.rs` (Tauri app setup - not needed)

### Dependency Graph

```
syncengine-cli ──────► syncengine-core ◄────── dioxus-app
                    │
    ┌───────────────┼───────────────┐
    ▼               ▼               ▼
automerge        iroh           redb
    │               │
    └───────┬───────┘
            ▼
    iroh-automerge patterns
```

### Data Model

```rust
// Core types
pub struct RealmId(pub [u8; 32]);      // blake3 hash
pub struct TaskId(pub Ulid);           // Time-sortable
pub struct DeviceId(pub [u8; 32]);     // Ed25519 pubkey
pub struct Did(pub String);            // did:sync:z{base58}

// Realm structure (stored in Automerge)
pub struct Realm {
    pub id: RealmId,
    pub name: String,
    pub is_shared: bool,
    pub created_at: i64,
    pub tasks: Vec<Task>,
    pub members: Vec<Did>,             // Empty for private realms
}

// Task structure
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub completed: bool,
    pub created_by: Did,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

// Invite ticket (for sharing)
pub struct InviteTicket {
    pub realm_id: RealmId,
    pub realm_key: [u8; 32],           // ChaCha20 key
    pub creator_node_id: iroh::NodeId,
    pub relay_url: Option<String>,
    pub expires_at: Option<i64>,
}
```

---

## 3. Development Phases

### Overview

```
Phase 1: P2P Foundation (CLI)     ████████░░░░░░░░  Week 1-2
Phase 2: Core Engine              ░░░░░░░░████████  Week 2-3
Phase 3: Dioxus UI                ░░░░░░░░░░░░████  Week 3-4
```

### Phase 1 Goals (This Document's Focus)
- Two CLI instances sync an Automerge document over Iroh
- Basic realm creation and task CRUD
- Invite generation and consumption
- All operations testable via CLI commands

### Phase 2 Goals
- Identity system (quantum-secure signatures)
- Encryption layer (per-realm keys)
- Persistent storage (redb)
- Multi-realm support

### Phase 3 Goals
- Dioxus desktop app
- Sacred geometry UI
- Mobile builds (iOS/Android)

---

## 4. Phase 1: P2P Foundation (CLI)

### Starting Point

The `iroh-examples` repository is already available in the adjacent directory. Study and run the `iroh-automerge-repo` example first:

```bash
cd ../iroh-examples/iroh-automerge-repo
cargo run
# Open a second terminal and run again to see sync
```

This example demonstrates:
- Multiple Automerge documents in a "repo"
- Gossip-based sync (not point-to-point)
- Topic swarms for document discovery
- The patterns we'll adapt for realms

Copy working code from old project and modify in place:
```bash
# Working modules to copy (tested, ~240 tests passing):
cp ../syncengine-tauri/src-tauri/src/identity.rs crates/syncengine-core/src/
cp ../syncengine-tauri/src-tauri/src/crypto.rs   crates/syncengine-core/src/
cp ../syncengine-tauri/src-tauri/src/storage.rs  crates/syncengine-core/src/
cp ../syncengine-tauri/src-tauri/src/types.rs    crates/syncengine-core/src/

# Then fix imports and remove any Tauri-specific code
```

### 4.1 Milestone 1: Gossip Echo Test (Day 1)

**Goal**: Prove gossip messaging works between two nodes on the same topic.

```rust
// tests/p2p_integration.rs
use iroh::{Endpoint, protocol::Router};
use iroh_gossip::{Gossip, TopicId};

#[tokio::test]
async fn test_gossip_echo() {
    // Node A
    let endpoint_a = Endpoint::builder().bind().await.unwrap();
    let gossip_a = Gossip::builder().spawn(endpoint_a.clone());
    let router_a = Router::builder(endpoint_a.clone())
        .accept(iroh_gossip::ALPN, gossip_a.clone())
        .spawn()
        .await
        .unwrap();
    
    // Node B
    let endpoint_b = Endpoint::builder().bind().await.unwrap();
    let gossip_b = Gossip::builder().spawn(endpoint_b.clone());
    let router_b = Router::builder(endpoint_b.clone())
        .accept(iroh_gossip::ALPN, gossip_b.clone())
        .spawn()
        .await
        .unwrap();
    
    // Shared topic (like a realm)
    let topic = TopicId::from_bytes(blake3::hash(b"test-realm").as_bytes().try_into().unwrap());
    
    // A subscribes to topic
    let (sender_a, mut recv_a) = gossip_a
        .subscribe(topic, vec![])
        .await
        .unwrap();
    
    // B joins topic with A as bootstrap peer
    let node_addr_a = endpoint_a.node_addr().await.unwrap();
    let (sender_b, mut recv_b) = gossip_b
        .subscribe(topic, vec![node_addr_a])
        .await
        .unwrap();
    
    // Wait for connection
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // B broadcasts message
    sender_b.broadcast(b"hello from B".to_vec().into()).await.unwrap();
    
    // A receives it
    let event = tokio::time::timeout(Duration::from_secs(5), recv_a.next())
        .await
        .unwrap()
        .unwrap();
    
    match event {
        iroh_gossip::Event::Received(msg) => {
            assert_eq!(msg.content.as_ref(), b"hello from B");
        }
        _ => panic!("Expected Received event"),
    }
}
```

**CLI Test**:
```bash
# Terminal 1: Start node and subscribe to test topic
$ cargo run -p syncengine-cli -- gossip-listen test-topic
Node ID: abc123...
Subscribed to topic: test-topic
Waiting for messages...

# Terminal 2: Join topic and send message
$ cargo run -p syncengine-cli -- gossip-send test-topic abc123 "hello world"
Joined topic with peer abc123
Sent: hello world

# Terminal 1 output:
Received from xyz789: hello world
```

### 4.2 Milestone 2: Automerge over Gossip (Day 2-3)

**Goal**: Sync Automerge documents using gossip broadcast (iroh-automerge-repo pattern).

```rust
// tests/p2p_integration.rs
#[tokio::test]
async fn test_automerge_gossip_sync() {
    let (node_a, node_b, topic) = create_gossip_pair().await;
    
    // Node A creates document with a task
    let doc_a = node_a.create_document(&topic);
    node_a.add_task(&doc_a, "Buy groceries");
    
    // A broadcasts sync message via gossip
    let sync_msg = node_a.generate_sync_message(&topic);
    node_a.broadcast(&topic, sync_msg).await;
    
    // B receives and applies
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Verify B has the task
    let tasks_b = node_b.get_tasks(&topic);
    assert_eq!(tasks_b.len(), 1);
    assert_eq!(tasks_b[0].title, "Buy groceries");
}

#[tokio::test]
async fn test_three_node_gossip_sync() {
    // This is the key test - gossip should propagate to all nodes
    let (node_a, node_b, node_c, topic) = create_gossip_trio().await;
    
    // A adds task
    node_a.add_task(&topic, "Task from A");
    node_a.broadcast_changes(&topic).await;
    
    // Wait for gossip propagation
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Both B and C should have it
    assert_eq!(node_b.get_tasks(&topic).len(), 1);
    assert_eq!(node_c.get_tasks(&topic).len(), 1);
    
    // C adds task
    node_c.add_task(&topic, "Task from C");
    node_c.broadcast_changes(&topic).await;
    
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // All three should have both tasks
    assert_eq!(node_a.get_tasks(&topic).len(), 2);
    assert_eq!(node_b.get_tasks(&topic).len(), 2);
    assert_eq!(node_c.get_tasks(&topic).len(), 2);
}
```

**Implementation** (adapted from iroh-automerge-repo):

```rust
// crates/syncengine-core/src/sync/gossip_sync.rs
use automerge::{sync, AutoCommit};
use iroh_gossip::{Gossip, TopicId, GossipEvent};
use std::collections::HashMap;

/// Manages Automerge documents synced via gossip
pub struct GossipDocRepo {
    /// Gossip protocol handle
    gossip: Gossip,
    /// Documents by topic (realm)
    docs: HashMap<TopicId, AutoCommit>,
    /// Sync states per peer per topic
    sync_states: HashMap<(TopicId, PublicKey), sync::State>,
}

impl GossipDocRepo {
    /// Subscribe to a realm's gossip topic
    pub async fn join_realm(
        &mut self,
        topic: TopicId,
        bootstrap_peers: Vec<NodeAddr>,
    ) -> Result<(), SyncError> {
        let (sender, receiver) = self.gossip
            .subscribe(topic, bootstrap_peers)
            .await?;
        
        // Store sender for broadcasting
        self.senders.insert(topic, sender);
        
        // Spawn task to handle incoming messages
        let docs = self.docs.clone();
        let sync_states = self.sync_states.clone();
        tokio::spawn(async move {
            Self::handle_gossip_events(topic, receiver, docs, sync_states).await;
        });
        
        Ok(())
    }
    
    /// Broadcast local changes to realm
    pub async fn broadcast_changes(&self, topic: &TopicId) -> Result<(), SyncError> {
        let doc = self.docs.get(topic).ok_or(SyncError::UnknownRealm)?;
        let sender = self.senders.get(topic).ok_or(SyncError::NotSubscribed)?;
        
        // Generate sync message for broadcast
        // Note: For broadcast, we send changes without peer-specific state
        if let Some(changes) = doc.get_changes(&[]) {
            let msg = GossipSyncMessage::Changes(changes.to_vec());
            sender.broadcast(postcard::to_allocvec(&msg)?).await?;
        }
        
        Ok(())
    }
    
    /// Handle incoming gossip events
    async fn handle_gossip_events(
        topic: TopicId,
        mut receiver: GossipReceiver,
        docs: Arc<RwLock<HashMap<TopicId, AutoCommit>>>,
        sync_states: Arc<RwLock<HashMap<(TopicId, PublicKey), sync::State>>>,
    ) {
        while let Some(event) = receiver.next().await {
            match event {
                GossipEvent::Received(msg) => {
                    let sync_msg: GossipSyncMessage = postcard::from_bytes(&msg.content)
                        .expect("invalid sync message");
                    
                    let mut docs = docs.write().await;
                    if let Some(doc) = docs.get_mut(&topic) {
                        match sync_msg {
                            GossipSyncMessage::Changes(changes) => {
                                doc.apply_changes(changes).ok();
                            }
                            GossipSyncMessage::SyncMessage(peer, data) => {
                                let mut states = sync_states.write().await;
                                let state = states
                                    .entry((topic, peer))
                                    .or_insert_with(sync::State::new);
                                
                                if let Ok(msg) = sync::Message::decode(&data) {
                                    doc.sync().receive_sync_message(state, msg).ok();
                                }
                            }
                        }
                    }
                }
                GossipEvent::NeighborUp(peer) => {
                    // New peer joined - initiate sync
                    // Send our current state to help them catch up
                }
                _ => {}
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
enum GossipSyncMessage {
    /// Broadcast changes (no specific peer)
    Changes(Vec<u8>),
    /// Peer-specific sync message
    SyncMessage(PublicKey, Vec<u8>),
    /// Request full sync
    SyncRequest(PublicKey),
}
```

### 4.3 Milestone 3: Invite System with Gossip (Day 4-5)

**Goal**: Generate invite tickets that include gossip topic + bootstrap peers.

```rust
#[tokio::test]
async fn test_invite_flow_with_gossip() {
    let mut node_a = TestNode::new().await;
    let mut node_b = TestNode::new().await;
    
    // A creates realm (also creates gossip topic)
    let realm_id = node_a.create_realm("Team Tasks");
    node_a.add_task(&realm_id, "Review PR");
    
    // A generates invite (includes topic + A as bootstrap peer)
    let invite = node_a.create_invite(&realm_id).await;
    let ticket_string = invite.to_string();
    
    // B receives invite
    let parsed_invite = InviteTicket::from_str(&ticket_string).unwrap();
    
    // B joins realm via invite - connects to gossip topic
    node_b.join_via_invite(parsed_invite).await.unwrap();
    
    // Wait for gossip sync
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Verify B has the task
    let tasks = node_b.get_tasks(&realm_id);
    assert_eq!(tasks.len(), 1);
}

#[tokio::test]
async fn test_invite_enables_bidirectional_gossip() {
    let mut node_a = TestNode::new().await;
    let mut node_b = TestNode::new().await;
    
    let realm_id = node_a.create_realm("Shared Realm");
    let invite = node_a.create_invite(&realm_id).await;
    node_b.join_via_invite(invite).await.unwrap();
    
    // Both add tasks - should propagate via gossip
    node_a.add_task(&realm_id, "From A");
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    node_b.add_task(&realm_id, "From B");
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Both should have both tasks
    assert_eq!(node_a.get_tasks(&realm_id).len(), 2);
    assert_eq!(node_b.get_tasks(&realm_id).len(), 2);
}
```

**Invite Ticket Format** (gossip-aware):
```rust
// crates/syncengine-core/src/types.rs
use serde::{Deserialize, Serialize};
use iroh_gossip::TopicId;

#[derive(Serialize, Deserialize)]
pub struct InviteTicket {
    /// Gossip topic for this realm
    pub topic: TopicId,
    /// Symmetric key for realm encryption (Phase 2)
    pub realm_key: [u8; 32],
    /// Bootstrap peers to join the gossip swarm
    pub bootstrap_peers: Vec<NodeAddr>,
    /// Human-readable realm name (optional, for display)
    pub realm_name: Option<String>,
    /// Expiration timestamp (optional)
    pub expires_at: Option<i64>,
}

impl InviteTicket {
    /// Create invite for a realm
    pub async fn create(
        realm_id: &RealmId,
        realm_key: &[u8; 32],
        endpoint: &Endpoint,
    ) -> Result<Self, InviteError> {
        // Topic derived from realm ID
        let topic = TopicId::from_bytes(*realm_id.as_bytes());
        
        // Include our node as bootstrap peer
        let our_addr = endpoint.node_addr().await?;
        
        Ok(Self {
            topic,
            realm_key: *realm_key,
            bootstrap_peers: vec![our_addr],
            realm_name: None,
            expires_at: None,
        })
    }
    
    /// Encode to URL-safe base58 string
    pub fn to_string(&self) -> String {
        let bytes = postcard::to_allocvec(self).unwrap();
        format!("sync-invite:{}", bs58::encode(&bytes).into_string())
    }
    
    /// Decode from string
    pub fn from_str(s: &str) -> Result<Self, InviteError> {
        let data = s.strip_prefix("sync-invite:")
            .ok_or(InviteError::InvalidPrefix)?;
        let bytes = bs58::decode(data).into_vec()?;
        Ok(postcard::from_bytes(&bytes)?)
    }
}
```

**CLI Commands** (updated for gossip):
```bash
# Create a new realm (subscribes to gossip topic)
$ syncengine-cli realm create "Shopping List"
Created realm: realm_7x8k2m
Subscribed to gossip topic

# Add a task (broadcasts via gossip automatically)
$ syncengine-cli task add realm_7x8k2m "Buy milk"
Added task: task_01HX
Broadcasting to 0 peers...

# Generate invite (includes our node as bootstrap)
$ syncengine-cli invite create realm_7x8k2m
Invite ticket:
sync-invite:3xK7mN2p...

QR Code: [displayed in terminal]

# Join via invite (connects to gossip swarm)
$ syncengine-cli invite join sync-invite:3xK7mN2p...
Connecting to bootstrap peers...
Joined gossip swarm for "Shopping List"
Syncing history...
Received 1 task(s)

# Now both nodes are in the gossip swarm
# Any changes broadcast automatically
```

### 4.4 Milestone 4: Persistence (Day 5-6)

**Goal**: State survives restarts.

```rust
#[tokio::test]
async fn test_persistence_across_restart() {
    let db_path = tempfile::tempdir().unwrap();
    
    // First session: create data
    {
        let engine = SyncEngine::open(db_path.path()).await.unwrap();
        let realm_id = engine.create_realm("Persistent Realm");
        engine.add_task(&realm_id, "Survive restart");
    }
    
    // Second session: verify data exists
    {
        let engine = SyncEngine::open(db_path.path()).await.unwrap();
        let realms = engine.list_realms();
        assert_eq!(realms.len(), 1);
        
        let tasks = engine.get_tasks(&realms[0].id);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Survive restart");
    }
}
```

**Storage Schema**:
```rust
// crates/syncengine-core/src/storage.rs
use redb::{Database, TableDefinition};

const REALMS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("realms");
const DOCUMENTS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("documents");
const IDENTITY: TableDefinition<&str, &[u8]> = TableDefinition::new("identity");
const SYNC_STATE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("sync_state");

pub struct Storage {
    db: Database,
}

impl Storage {
    pub fn open(path: &Path) -> Result<Self, StorageError> {
        let db = Database::create(path.join("sync.redb"))?;
        
        // Create tables
        let write_txn = db.begin_write()?;
        write_txn.open_table(REALMS)?;
        write_txn.open_table(DOCUMENTS)?;
        write_txn.open_table(IDENTITY)?;
        write_txn.open_table(SYNC_STATE)?;
        write_txn.commit()?;
        
        Ok(Self { db })
    }
    
    pub fn save_document(&self, realm_id: &RealmId, doc: &AutoCommit) -> Result<(), StorageError> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(DOCUMENTS)?;
            table.insert(realm_id.as_bytes(), doc.save().as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }
    
    pub fn load_document(&self, realm_id: &RealmId) -> Result<Option<AutoCommit>, StorageError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(DOCUMENTS)?;
        
        match table.get(realm_id.as_bytes())? {
            Some(data) => Ok(Some(AutoCommit::load(&data.value())?)),
            None => Ok(None),
        }
    }
}
```

### 4.5 CLI Implementation

```rust
// crates/syncengine-cli/src/main.rs
use clap::{Parser, Subcommand};
use sync_core::{SyncEngine, RealmId, InviteTicket};

#[derive(Parser)]
#[command(name = "syncengine-cli")]
#[command(about = "Synchronicity Engine CLI")]
struct Cli {
    /// Data directory
    #[arg(short, long, default_value = "~/.sync-engine")]
    data_dir: PathBuf,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Realm management
    Realm {
        #[command(subcommand)]
        action: RealmAction,
    },
    /// Task management
    Task {
        #[command(subcommand)]
        action: TaskAction,
    },
    /// Invite management
    Invite {
        #[command(subcommand)]
        action: InviteAction,
    },
    /// Start sync server
    Serve,
    /// Sync with a peer
    Sync {
        /// Peer's node ID
        node_id: String,
        /// Realm to sync
        realm_id: String,
    },
    /// Show node info
    Info,
}

#[derive(Subcommand)]
enum RealmAction {
    Create { name: String },
    List,
    Delete { realm_id: String },
}

#[derive(Subcommand)]
enum TaskAction {
    Add { realm_id: String, title: String },
    List { realm_id: String },
    Toggle { realm_id: String, task_id: String },
    Delete { realm_id: String, task_id: String },
}

#[derive(Subcommand)]
enum InviteAction {
    Create { realm_id: String },
    Join { ticket: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let engine = SyncEngine::open(&cli.data_dir).await?;
    
    match cli.command {
        Commands::Realm { action } => match action {
            RealmAction::Create { name } => {
                let realm_id = engine.create_realm(&name);
                println!("Created realm: {}", realm_id);
            }
            RealmAction::List => {
                for realm in engine.list_realms() {
                    let status = if realm.is_shared { "shared" } else { "private" };
                    println!("{} - {} ({})", realm.id, realm.name, status);
                }
            }
            RealmAction::Delete { realm_id } => {
                engine.delete_realm(&realm_id.parse()?)?;
                println!("Deleted realm: {}", realm_id);
            }
        },
        
        Commands::Task { action } => match action {
            TaskAction::Add { realm_id, title } => {
                let task_id = engine.add_task(&realm_id.parse()?, &title)?;
                println!("Added task: {}", task_id);
            }
            TaskAction::List { realm_id } => {
                for task in engine.get_tasks(&realm_id.parse()?) {
                    let check = if task.completed { "✓" } else { " " };
                    println!("[{}] {} ({})", check, task.title, task.id);
                }
            }
            TaskAction::Toggle { realm_id, task_id } => {
                engine.toggle_task(&realm_id.parse()?, &task_id.parse()?)?;
                println!("Toggled task: {}", task_id);
            }
            TaskAction::Delete { realm_id, task_id } => {
                engine.delete_task(&realm_id.parse()?, &task_id.parse()?)?;
                println!("Deleted task: {}", task_id);
            }
        },
        
        Commands::Invite { action } => match action {
            InviteAction::Create { realm_id } => {
                let invite = engine.create_invite(&realm_id.parse()?).await?;
                println!("Invite: {}", invite.to_string());
                // TODO: Display QR code
            }
            InviteAction::Join { ticket } => {
                let invite = InviteTicket::from_str(&ticket)?;
                engine.join_via_invite(invite).await?;
                println!("Joined realm successfully");
            }
        },
        
        Commands::Serve => {
            println!("Node ID: {}", engine.node_id());
            println!("Listening for connections...");
            engine.serve().await?;
        }
        
        Commands::Sync { node_id, realm_id } => {
            println!("Syncing with {}...", node_id);
            engine.sync_with(&node_id.parse()?, &realm_id.parse()?).await?;
            println!("Sync complete");
        }
        
        Commands::Info => {
            println!("Node ID: {}", engine.node_id());
            println!("Data dir: {}", cli.data_dir.display());
            println!("Realms: {}", engine.list_realms().len());
        }
    }
    
    Ok(())
}
```

---

## 5. Phase 2: Core Engine

### 5.1 Identity System

**Goal**: Quantum-secure hybrid signatures.

```rust
// crates/syncengine-core/src/identity.rs
use ed25519_dalek::{SigningKey, VerifyingKey};
use pqcrypto_dilithium::dilithium3::{self as mldsa65};

pub struct Identity {
    /// Classical signature key (fast, small)
    ed25519_secret: SigningKey,
    ed25519_public: VerifyingKey,
    
    /// Post-quantum signature key (large, future-proof)
    mldsa_secret: mldsa65::SecretKey,
    mldsa_public: mldsa65::PublicKey,
    
    /// DID derived from hybrid public key
    did: Did,
}

impl Identity {
    pub fn generate() -> Self {
        let ed25519_secret = SigningKey::generate(&mut rand::thread_rng());
        let ed25519_public = ed25519_secret.verifying_key();
        
        let (mldsa_public, mldsa_secret) = mldsa65::keypair();
        
        // DID = did:sync:z{base58(blake3(ed25519_pub || mldsa_pub))}
        let mut hasher = blake3::Hasher::new();
        hasher.update(ed25519_public.as_bytes());
        hasher.update(mldsa_public.as_bytes());
        let hash = hasher.finalize();
        
        let did = Did(format!("did:sync:z{}", bs58::encode(hash.as_bytes()).into_string()));
        
        Self {
            ed25519_secret,
            ed25519_public,
            mldsa_secret,
            mldsa_public,
            did,
        }
    }
    
    /// Sign message with both keys
    pub fn sign(&self, message: &[u8]) -> HybridSignature {
        let ed25519_sig = self.ed25519_secret.sign(message);
        let mldsa_sig = mldsa65::sign(message, &self.mldsa_secret);
        
        HybridSignature {
            ed25519: ed25519_sig.to_bytes(),
            mldsa: mldsa_sig.as_bytes().to_vec(),
        }
    }
    
    pub fn did(&self) -> &Did {
        &self.did
    }
}

#[derive(Serialize, Deserialize)]
pub struct HybridSignature {
    pub ed25519: [u8; 64],
    pub mldsa: Vec<u8>, // ~3293 bytes
}

impl HybridSignature {
    /// Verify with both keys (both must pass)
    pub fn verify(&self, message: &[u8], ed_pub: &VerifyingKey, mldsa_pub: &mldsa65::PublicKey) -> bool {
        let ed_sig = ed25519_dalek::Signature::from_bytes(&self.ed25519);
        let ed_ok = ed_pub.verify_strict(message, &ed_sig).is_ok();
        
        let mldsa_sig = mldsa65::Signature::from_bytes(&self.mldsa).unwrap();
        let mldsa_ok = mldsa65::verify(&mldsa_sig, message, mldsa_pub).is_ok();
        
        ed_ok && mldsa_ok
    }
}
```

### 5.2 Encryption Layer

**Goal**: Per-realm encryption with ChaCha20-Poly1305.

```rust
// crates/syncengine-core/src/crypto.rs
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};

pub struct RealmCrypto {
    cipher: ChaCha20Poly1305,
}

impl RealmCrypto {
    pub fn new(key: &[u8; 32]) -> Self {
        Self {
            cipher: ChaCha20Poly1305::new(key.into()),
        }
    }
    
    pub fn generate_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        key
    }
    
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = self.cipher.encrypt(nonce, plaintext)?;
        
        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);
        Ok(result)
    }
    
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if data.len() < 12 {
            return Err(CryptoError::InvalidCiphertext);
        }
        
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        self.cipher.decrypt(nonce, ciphertext)
            .map_err(|_| CryptoError::DecryptionFailed)
    }
}
```

### 5.3 Signed + Encrypted Sync Messages

```rust
// crates/syncengine-core/src/sync/protocol.rs

/// Message sent over the wire
#[derive(Serialize, Deserialize)]
pub struct SyncEnvelope {
    /// Sender's DID
    pub sender: Did,
    /// Encrypted payload (contains SyncPayload)
    pub ciphertext: Vec<u8>,
    /// Signature over ciphertext
    pub signature: HybridSignature,
}

#[derive(Serialize, Deserialize)]
pub struct SyncPayload {
    /// Automerge sync message
    pub sync_message: Vec<u8>,
    /// Timestamp
    pub timestamp: i64,
}

impl SyncEnvelope {
    pub fn create(
        identity: &Identity,
        crypto: &RealmCrypto,
        sync_message: &[u8],
    ) -> Result<Self, SyncError> {
        let payload = SyncPayload {
            sync_message: sync_message.to_vec(),
            timestamp: chrono::Utc::now().timestamp(),
        };
        
        let payload_bytes = postcard::to_allocvec(&payload)?;
        let ciphertext = crypto.encrypt(&payload_bytes)?;
        let signature = identity.sign(&ciphertext);
        
        Ok(Self {
            sender: identity.did().clone(),
            ciphertext,
            signature,
        })
    }
    
    pub fn open(
        &self,
        crypto: &RealmCrypto,
        // TODO: Verify against known member public keys
    ) -> Result<SyncPayload, SyncError> {
        let plaintext = crypto.decrypt(&self.ciphertext)?;
        let payload: SyncPayload = postcard::from_bytes(&plaintext)?;
        Ok(payload)
    }
}
```

---

## 6. Phase 3: Dioxus UI

### 6.1 App Entry Point

```rust
// src/main.rs
use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Initialize engine (runs once)
    let engine = use_context_provider(|| {
        Signal::new(
            sync_core::SyncEngine::open_default()
                .expect("Failed to initialize engine")
        )
    });
    
    // Global state
    let current_realm = use_signal(|| None::<sync_core::RealmId>);
    let connection_status = use_signal(|| ConnectionStatus::Offline);
    
    // Start background sync
    use_future(move || async move {
        let eng = engine.read();
        eng.start_background_sync().await;
    });
    
    rsx! {
        div { class: "sync-engine sacred-background",
            StatusBar { status: connection_status }
            
            div { class: "main-layout",
                RealmSelector {
                    on_select: move |id| current_realm.set(Some(id))
                }
                
                div { class: "content-area",
                    if let Some(realm_id) = current_realm() {
                        TaskList { realm_id }
                    } else {
                        WelcomeScreen {}
                    }
                }
            }
        }
    }
}
```

### 6.2 Realm Selector Component

```rust
// src/components/realm_selector.rs
use dioxus::prelude::*;
use sync_core::{RealmId, RealmInfo};

#[component]
pub fn RealmSelector(on_select: EventHandler<RealmId>) -> Element {
    let engine = use_context::<Signal<sync_core::SyncEngine>>();
    let realms = use_memo(move || engine.read().list_realms());
    let mut show_create = use_signal(|| false);
    
    rsx! {
        nav { class: "realm-selector",
            div { class: "realm-header",
                h2 { "Realms" }
                button {
                    class: "icon-button",
                    onclick: move |_| show_create.set(true),
                    "+"
                }
            }
            
            ul { class: "realm-list",
                for realm in realms.read().iter() {
                    RealmCard {
                        key: "{realm.id}",
                        realm: realm.clone(),
                        on_click: move |_| on_select.call(realm.id.clone())
                    }
                }
            }
            
            if show_create() {
                CreateRealmModal {
                    on_close: move |_| show_create.set(false),
                    on_create: move |name| {
                        engine.write().create_realm(&name);
                        show_create.set(false);
                    }
                }
            }
        }
    }
}

#[component]
fn RealmCard(realm: RealmInfo, on_click: EventHandler<()>) -> Element {
    rsx! {
        li {
            class: "realm-card",
            onclick: move |_| on_click.call(()),
            
            div { class: "realm-icon",
                if realm.is_shared {
                    span { class: "icon-connected" }
                } else {
                    span { class: "icon-private" }
                }
            }
            
            div { class: "realm-content",
                h3 { class: "realm-name", "{realm.name}" }
                p { class: "realm-meta",
                    if realm.is_shared {
                        "{realm.member_count} souls connected"
                    } else {
                        "private sanctuary"
                    }
                }
            }
            
            if realm.has_activity {
                div { class: "activity-pulse active" }
            }
        }
    }
}
```

### 6.3 Task List Component

```rust
// src/components/task_list.rs
use dioxus::prelude::*;
use sync_core::{RealmId, Task, TaskId};

#[component]
pub fn TaskList(realm_id: RealmId) -> Element {
    let engine = use_context::<Signal<sync_core::SyncEngine>>();
    
    let tasks = use_memo(move || {
        engine.read().get_tasks(&realm_id)
    });
    
    let mut new_task_title = use_signal(String::new);
    
    let add_task = move |_| {
        let title = new_task_title.read().clone();
        if !title.trim().is_empty() {
            engine.write().add_task(&realm_id, title.trim()).ok();
            new_task_title.set(String::new());
        }
    };
    
    rsx! {
        section { class: "task-panel",
            div { class: "task-list",
                for task in tasks.read().iter() {
                    TaskItem {
                        key: "{task.id}",
                        task: task.clone(),
                        on_toggle: move |id| {
                            engine.write().toggle_task(&realm_id, &id).ok();
                        },
                        on_delete: move |id| {
                            engine.write().delete_task(&realm_id, &id).ok();
                        }
                    }
                }
            }
            
            div { class: "new-task",
                input {
                    class: "task-input",
                    placeholder: "Add new intention...",
                    value: "{new_task_title}",
                    oninput: move |e| new_task_title.set(e.value()),
                    onkeydown: move |e| {
                        if e.key() == Key::Enter {
                            add_task(());
                        }
                    }
                }
                button {
                    class: "add-button",
                    onclick: add_task,
                    "+"
                }
            }
        }
    }
}

#[component]
fn TaskItem(
    task: Task,
    on_toggle: EventHandler<TaskId>,
    on_delete: EventHandler<TaskId>,
) -> Element {
    let task_id = task.id.clone();
    let task_id_del = task.id.clone();
    
    rsx! {
        div {
            class: "task-item",
            class: if task.completed { "completed" },
            
            button {
                class: "toggle-button",
                onclick: move |_| on_toggle.call(task_id.clone()),
                if task.completed { "◉" } else { "○" }
            }
            
            span { class: "task-title", "{task.title}" }
            
            button {
                class: "delete-button",
                onclick: move |_| on_delete.call(task_id_del.clone()),
                "×"
            }
        }
    }
}
```

---

## 7. Sacred Design System

### 7.1 Design Tokens

```css
/* assets/style.css */

:root {
  /* === SACRED COLORS === */
  
  /* Void (backgrounds) */
  --void-black: #0a0a0f;
  --deep-void: #12121a;
  --surface: #1a1a24;
  
  /* Moss Gold (primary actions, completion) */
  --moss-gold: #4ade80;
  --moss-gold-glow: rgba(74, 222, 128, 0.4);
  
  /* Temple Gold (highlights, sacred moments) */
  --temple-gold: #facc15;
  --temple-gold-glow: rgba(250, 204, 21, 0.3);
  
  /* Cyan Aether (connection, sync) */
  --cyan-aether: #22d3ee;
  --cyan-aether-glow: rgba(34, 211, 238, 0.35);
  
  /* Semantic */
  --danger-crimson: #ef4444;
  --warning-amber: #fbbf24;
  
  /* Glass surfaces */
  --glass-5: rgba(255, 255, 255, 0.03);
  --glass-10: rgba(255, 255, 255, 0.05);
  --glass-20: rgba(255, 255, 255, 0.08);
  
  /* Text */
  --ink-40: rgba(255, 255, 255, 0.4);
  --ink-60: rgba(255, 255, 255, 0.6);
  --ink-80: rgba(255, 255, 255, 0.8);
  --ink-100: rgba(255, 255, 255, 1.0);
  
  /* === FIBONACCI SPACING === */
  --space-1: 8px;
  --space-2: 13px;
  --space-3: 21px;
  --space-4: 34px;
  --space-5: 55px;
  --space-6: 89px;
  
  /* === TYPOGRAPHY (Fibonacci scale) === */
  --text-xs: 13px;
  --text-sm: 15px;
  --text-base: 17px;
  --text-lg: 21px;
  --text-xl: 34px;
  --text-2xl: 55px;
  
  /* Leading */
  --leading-tight: 1.2;
  --leading-golden: 1.618;
  --leading-relaxed: 1.8;
  
  /* === RADII (Fibonacci) === */
  --radius-sm: 8px;
  --radius-md: 13px;
  --radius-lg: 21px;
  
  /* === TRANSITIONS === */
  --ease-sacred: cubic-bezier(0.4, 0.0, 0.2, 1);
  --duration-fast: 150ms;
  --duration-normal: 300ms;
  --duration-slow: 500ms;
}

/* === BASE STYLES === */

* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

body {
  font-family: 'Inter Variable', -apple-system, BlinkMacSystemFont, system-ui, sans-serif;
  font-size: var(--text-base);
  line-height: var(--leading-golden);
  color: var(--ink-80);
  background: var(--void-black);
  -webkit-font-smoothing: antialiased;
}

/* === SACRED BACKGROUND === */

.sacred-background {
  background:
    radial-gradient(circle at 30% 20%, rgba(74, 222, 128, 0.03) 0%, transparent 50%),
    radial-gradient(circle at 70% 80%, rgba(250, 204, 21, 0.02) 0%, transparent 50%),
    var(--void-black);
  min-height: 100vh;
}

/* === LAYOUT === */

.sync-engine {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.main-layout {
  display: flex;
  flex: 1;
  overflow: hidden;
}

/* === REALM SELECTOR === */

.realm-selector {
  width: 320px;
  max-width: 38.2%; /* Golden ratio */
  background: var(--glass-5);
  border-right: 1px solid rgba(74, 222, 128, 0.1);
  display: flex;
  flex-direction: column;
}

.realm-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-3);
  border-bottom: 1px solid rgba(74, 222, 128, 0.1);
}

.realm-header h2 {
  font-size: var(--text-lg);
  font-weight: 500;
  color: var(--ink-60);
}

.realm-list {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-2);
  list-style: none;
}

/* === REALM CARD === */

.realm-card {
  height: 89px; /* Fibonacci */
  padding: var(--space-3);
  margin-bottom: var(--space-2);
  
  background: var(--glass-10);
  border: 1px solid rgba(74, 222, 128, 0.15);
  border-radius: var(--radius-md);
  
  display: flex;
  align-items: center;
  gap: var(--space-2);
  cursor: pointer;
  
  transition: all var(--duration-normal) var(--ease-sacred);
}

.realm-card:hover {
  transform: translateY(-2px);
  border-color: rgba(74, 222, 128, 0.3);
  box-shadow:
    inset 0 0 21px rgba(74, 222, 128, 0.05),
    0 0 2px rgba(250, 204, 21, 0.4),
    0 8px 34px rgba(0, 0, 0, 0.4);
}

.realm-icon {
  width: 34px;
  height: 34px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--moss-gold);
  font-size: var(--text-lg);
}

.realm-content {
  flex: 1;
  min-width: 0;
}

.realm-name {
  font-size: var(--text-base);
  font-weight: 500;
  color: var(--ink-80);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.realm-meta {
  font-size: var(--text-xs);
  color: var(--ink-40);
  margin-top: 4px;
}

.activity-pulse {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--moss-gold);
  opacity: 0;
  transition: opacity var(--duration-normal) var(--ease-sacred);
}

.activity-pulse.active {
  opacity: 1;
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { box-shadow: 0 0 8px var(--moss-gold-glow); }
  50% { box-shadow: 0 0 21px var(--moss-gold-glow); }
}

/* === TASK PANEL === */

.content-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.task-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: var(--space-4);
}

.task-list {
  flex: 1;
  overflow-y: auto;
}

/* === TASK ITEM === */

.task-item {
  height: 55px; /* Fibonacci */
  padding: 0 var(--space-3);
  margin-bottom: var(--space-1);
  
  background: var(--glass-5);
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  
  display: flex;
  align-items: center;
  gap: var(--space-2);
  
  transition: all var(--duration-fast) var(--ease-sacred);
}

.task-item:hover {
  background: var(--glass-10);
  border-color: rgba(74, 222, 128, 0.1);
}

.task-item.completed .task-title {
  text-decoration: line-through;
  color: var(--ink-40);
}

.toggle-button {
  width: 34px;
  height: 34px;
  background: none;
  border: none;
  color: var(--moss-gold);
  font-size: var(--text-lg);
  cursor: pointer;
  transition: transform var(--duration-fast) var(--ease-sacred);
}

.toggle-button:hover {
  transform: scale(1.1);
}

.task-title {
  flex: 1;
  font-size: var(--text-base);
  color: var(--ink-80);
}

.delete-button {
  width: 34px;
  height: 34px;
  background: none;
  border: none;
  color: var(--ink-40);
  font-size: var(--text-lg);
  cursor: pointer;
  opacity: 0;
  transition: all var(--duration-fast) var(--ease-sacred);
}

.task-item:hover .delete-button {
  opacity: 1;
}

.delete-button:hover {
  color: var(--danger-crimson);
}

/* === NEW TASK INPUT === */

.new-task {
  display: flex;
  gap: var(--space-2);
  padding-top: var(--space-3);
  border-top: 1px solid rgba(74, 222, 128, 0.1);
}

.task-input {
  flex: 1;
  height: 55px; /* Fibonacci */
  padding: 0 var(--space-3);
  
  background: var(--glass-10);
  border: 1px solid rgba(74, 222, 128, 0.2);
  border-radius: var(--radius-sm);
  
  font-family: inherit;
  font-size: var(--text-base);
  color: var(--cyan-aether);
  
  transition: all var(--duration-fast) var(--ease-sacred);
}

.task-input::placeholder {
  color: var(--ink-40);
}

.task-input:focus {
  outline: none;
  border-color: var(--moss-gold);
  box-shadow: 0 0 13px rgba(74, 222, 128, 0.2);
}

.add-button {
  width: 55px;
  height: 55px;
  
  background: var(--glass-10);
  border: 1px solid rgba(74, 222, 128, 0.2);
  border-radius: var(--radius-sm);
  
  font-size: var(--text-lg);
  color: var(--moss-gold);
  cursor: pointer;
  
  transition: all var(--duration-fast) var(--ease-sacred);
}

.add-button:hover {
  background: rgba(74, 222, 128, 0.1);
  border-color: var(--moss-gold);
  box-shadow: 0 0 13px var(--moss-gold-glow);
}

/* === STATUS BAR === */

.status-bar {
  height: 34px;
  padding: 0 var(--space-3);
  
  background: var(--deep-void);
  border-bottom: 1px solid rgba(74, 222, 128, 0.1);
  
  display: flex;
  align-items: center;
  justify-content: space-between;
  
  font-size: var(--text-xs);
  color: var(--ink-40);
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: var(--space-1);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.status-dot.online {
  background: var(--moss-gold);
  box-shadow: 0 0 8px var(--moss-gold-glow);
}

.status-dot.syncing {
  background: var(--cyan-aether);
  animation: pulse 1s ease-in-out infinite;
}

.status-dot.offline {
  background: var(--ink-40);
}

/* === ICON BUTTON === */

.icon-button {
  width: 34px;
  height: 34px;
  
  background: none;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  
  font-size: var(--text-lg);
  color: var(--ink-60);
  cursor: pointer;
  
  transition: all var(--duration-fast) var(--ease-sacred);
}

.icon-button:hover {
  color: var(--moss-gold);
  border-color: rgba(74, 222, 128, 0.2);
  background: var(--glass-10);
}

/* === FOCUS STATES (Accessibility) === */

*:focus-visible {
  outline: 2px solid var(--moss-gold);
  outline-offset: 2px;
}

/* === REDUCED MOTION === */

@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

### 7.2 Visual Language Summary

| Element | Specification |
|---------|---------------|
| **Primary Color** | Moss Gold `#4ade80` — completion, affirmation |
| **Secondary Color** | Temple Gold `#facc15` — highlights, sacred moments |
| **Connection Color** | Cyan Aether `#22d3ee` — sync status, connectivity |
| **Background** | Void Black `#0a0a0f` with subtle radial gradients |
| **Spacing Scale** | Fibonacci: 8, 13, 21, 34, 55, 89px |
| **Type Scale** | Fibonacci: 13, 15, 17, 21, 34, 55px |
| **Border Radius** | Fibonacci: 8, 13, 21px |
| **Task Item Height** | 55px (Fibonacci) |
| **Realm Card Height** | 89px (Fibonacci) |
| **Touch Target Min** | 34×34px |
| **Golden Ratio** | φ = 1.618 for proportions and line-height |

---

## 8. Testing Strategy

### 8.1 Test Categories

```
tests/
├── unit/                    # Fast, isolated tests
│   ├── realm_test.rs       # Realm CRUD operations
│   ├── task_test.rs        # Task CRUD operations
│   ├── crypto_test.rs      # Encryption/decryption
│   ├── identity_test.rs    # Signature generation/verification
│   └── storage_test.rs     # Persistence operations
│
├── integration/             # Multi-component tests
│   ├── p2p_sync_test.rs    # Two-node synchronization
│   ├── invite_flow_test.rs # Full invite lifecycle
│   └── offline_test.rs     # Offline → online sync
│
└── e2e/                     # Full system tests (CLI)
    └── cli_scenarios.rs     # Complete user workflows
```

### 8.2 P2P Integration Tests (Gossip-Based)

```rust
// tests/integration/p2p_sync_test.rs

/// Test helper: create two nodes subscribed to same topic
async fn create_gossip_pair() -> (TestNode, TestNode, TopicId) {
    let node_a = TestNode::new().await;
    let node_b = TestNode::new().await;
    
    let topic = TopicId::from_bytes(rand::random());
    
    // A subscribes first
    node_a.subscribe(topic, vec![]).await.unwrap();
    
    // B joins with A as bootstrap
    let addr_a = node_a.node_addr().await;
    node_b.subscribe(topic, vec![addr_a]).await.unwrap();
    
    // Wait for connection
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    (node_a, node_b, topic)
}

/// Test helper: create three nodes in same gossip swarm
async fn create_gossip_trio() -> (TestNode, TestNode, TestNode, TopicId) {
    let node_a = TestNode::new().await;
    let node_b = TestNode::new().await;
    let node_c = TestNode::new().await;
    
    let topic = TopicId::from_bytes(rand::random());
    
    // A is the seed node
    node_a.subscribe(topic, vec![]).await.unwrap();
    let addr_a = node_a.node_addr().await;
    
    // B and C join via A
    node_b.subscribe(topic, vec![addr_a.clone()]).await.unwrap();
    node_c.subscribe(topic, vec![addr_a]).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    (node_a, node_b, node_c, topic)
}

#[tokio::test]
async fn test_basic_gossip_sync() {
    let (node_a, node_b, topic) = create_gossip_pair().await;
    
    // A creates document with task
    node_a.engine.create_document(&topic);
    node_a.engine.add_task(&topic, "Task 1").unwrap();
    node_a.engine.broadcast_changes(&topic).await.unwrap();
    
    // Wait for gossip delivery
    tokio::time::sleep(Duration::from_millis(300)).await;
    
    // Verify B received it
    let tasks = node_b.engine.get_tasks(&topic);
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Task 1");
}

#[tokio::test]
async fn test_three_node_gossip_propagation() {
    // This is the CRITICAL test - proves gossip works beyond point-to-point
    let (node_a, node_b, node_c, topic) = create_gossip_trio().await;
    
    // Create initial document
    node_a.engine.create_document(&topic);
    
    // A adds task
    node_a.engine.add_task(&topic, "From A").unwrap();
    node_a.engine.broadcast_changes(&topic).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(300)).await;
    
    // Both B and C should have it
    assert_eq!(node_b.engine.get_tasks(&topic).len(), 1);
    assert_eq!(node_c.engine.get_tasks(&topic).len(), 1);
    
    // C adds task (C might not be directly connected to A)
    node_c.engine.add_task(&topic, "From C").unwrap();
    node_c.engine.broadcast_changes(&topic).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(300)).await;
    
    // All three should have both tasks
    assert_eq!(node_a.engine.get_tasks(&topic).len(), 2);
    assert_eq!(node_b.engine.get_tasks(&topic).len(), 2);
    assert_eq!(node_c.engine.get_tasks(&topic).len(), 2);
}

#[tokio::test]
async fn test_concurrent_gossip_edits() {
    let (node_a, node_b, topic) = create_gossip_pair().await;
    
    node_a.engine.create_document(&topic);
    node_b.engine.create_document(&topic);
    
    // Both add tasks simultaneously
    let handle_a = tokio::spawn({
        let engine = node_a.engine.clone();
        let topic = topic;
        async move {
            engine.add_task(&topic, "From A").unwrap();
            engine.broadcast_changes(&topic).await.unwrap();
        }
    });
    
    let handle_b = tokio::spawn({
        let engine = node_b.engine.clone();
        let topic = topic;
        async move {
            engine.add_task(&topic, "From B").unwrap();
            engine.broadcast_changes(&topic).await.unwrap();
        }
    });
    
    handle_a.await.unwrap();
    handle_b.await.unwrap();
    
    // Wait for cross-propagation
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Both should have both tasks (CRDT merge)
    let tasks_a = node_a.engine.get_tasks(&topic);
    let tasks_b = node_b.engine.get_tasks(&topic);
    
    assert_eq!(tasks_a.len(), 2);
    assert_eq!(tasks_b.len(), 2);
    
    // Same content
    let titles_a: HashSet<_> = tasks_a.iter().map(|t| &t.title).collect();
    let titles_b: HashSet<_> = tasks_b.iter().map(|t| &t.title).collect();
    assert_eq!(titles_a, titles_b);
}

#[tokio::test]
async fn test_late_joiner_catches_up() {
    let (node_a, node_b, topic) = create_gossip_pair().await;
    
    // A and B add tasks
    node_a.engine.create_document(&topic);
    node_a.engine.add_task(&topic, "Task 1").unwrap();
    node_a.engine.add_task(&topic, "Task 2").unwrap();
    node_a.engine.broadcast_changes(&topic).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // C joins later
    let node_c = TestNode::new().await;
    let addr_a = node_a.node_addr().await;
    node_c.subscribe(topic, vec![addr_a]).await.unwrap();
    
    // Wait for C to sync
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // C should have all tasks
    let tasks = node_c.engine.get_tasks(&topic);
    assert_eq!(tasks.len(), 2);
}

#[tokio::test]
async fn test_invite_flow_with_gossip() {
    let node_a = TestNode::new().await;
    let node_b = TestNode::new().await;
    
    // A creates realm (with gossip topic)
    let realm_id = node_a.engine.create_realm("Team Tasks").await.unwrap();
    node_a.engine.add_task(&realm_id, "Existing task").await.unwrap();
    
    // A creates invite
    let invite = node_a.engine.create_invite(&realm_id).await.unwrap();
    
    // B joins via invite
    node_b.engine.join_via_invite(invite).await.unwrap();
    
    // Wait for sync via gossip
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // B should have the task
    let tasks = node_b.engine.get_tasks(&realm_id);
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Existing task");
    
    // Now B adds a task
    node_b.engine.add_task(&realm_id, "New from B").await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(300)).await;
    
    // A should receive it via gossip
    assert_eq!(node_a.engine.get_tasks(&realm_id).len(), 2);
}
```

### 8.3 CLI Scenario Tests

```rust
// tests/e2e/cli_scenarios.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_realm_create_and_list() {
    let temp_dir = tempfile::tempdir().unwrap();
    
    // Create realm
    Command::cargo_bin("syncengine-cli")
        .unwrap()
        .args(["--data-dir", temp_dir.path().to_str().unwrap()])
        .args(["realm", "create", "Test Realm"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created realm:"));
    
    // List realms
    Command::cargo_bin("syncengine-cli")
        .unwrap()
        .args(["--data-dir", temp_dir.path().to_str().unwrap()])
        .args(["realm", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Test Realm"));
}

#[test]
fn test_task_lifecycle() {
    let temp_dir = tempfile::tempdir().unwrap();
    let data_dir = temp_dir.path().to_str().unwrap();
    
    // Create realm and capture ID
    let output = Command::cargo_bin("syncengine-cli")
        .unwrap()
        .args(["--data-dir", data_dir])
        .args(["realm", "create", "Task Test"])
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let realm_id = stdout.split(':').last().unwrap().trim();
    
    // Add task
    Command::cargo_bin("syncengine-cli")
        .unwrap()
        .args(["--data-dir", data_dir])
        .args(["task", "add", realm_id, "Buy milk"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added task:"));
    
    // List tasks
    Command::cargo_bin("syncengine-cli")
        .unwrap()
        .args(["--data-dir", data_dir])
        .args(["task", "list", realm_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Buy milk"))
        .stdout(predicate::str::contains("[ ]")); // Not completed
}
```

### 8.4 TDD Workflow

For each feature:

1. **Write failing test first**
   ```bash
   cargo test test_feature_name -- --nocapture
   # Should fail: "not yet implemented" or similar
   ```

2. **Implement minimum code to pass**
   ```bash
   cargo test test_feature_name
   # Should pass
   ```

3. **Refactor if needed**
   ```bash
   cargo test  # All tests should still pass
   cargo clippy  # No warnings
   ```

4. **Move to next test**

---

## 9. Security Model

### 9.1 Threat Model

| Threat | Mitigation |
|--------|------------|
| Eavesdropping on sync | ChaCha20-Poly1305 encryption per realm |
| Message tampering | Hybrid signatures on all sync messages |
| Replay attacks | Timestamps + nonce in every message |
| Relay server sees content | All content encrypted before transmission |
| Quantum computer breaks Ed25519 | ML-DSA-65 signature required alongside Ed25519 |
| Stolen device | Keys encrypted at rest (Phase 2+) |
| Invite interception | Realm key in invite grants access |

### 9.2 What Relay Servers See

| Data | Visible to Relay |
|------|------------------|
| Realm names | ❌ (hashed) |
| Task content | ❌ (encrypted) |
| Participant DIDs | ⚠️ (hashed, linkable) |
| Message sizes | ✅ (can infer activity) |
| Connection timing | ✅ (metadata) |
| IP addresses | ✅ (network layer) |

### 9.3 Key Hierarchy

```
Root Identity (never leaves device)
├── Ed25519 Secret Key
├── ML-DSA-65 Secret Key
└── Derives →
    ├── Device Keys (for delegation)
    └── Recovery Key (SPHINCS+ via BIP39 mnemonic)

Per-Realm Keys
├── Symmetric Key (ChaCha20-Poly1305)
│   └── Shared via invite ticket
└── Topic Key (blake3 hash for gossip)
```

---

## 10. API Reference

### 10.1 SyncEngine Public API

```rust
// crates/syncengine-core/src/lib.rs

/// Main engine - the only struct Dioxus needs to interact with
pub struct SyncEngine { /* ... */ }

impl SyncEngine {
    // === Lifecycle ===
    
    /// Open or create engine at default location
    pub async fn open_default() -> Result<Self, EngineError>;
    
    /// Open or create engine at specific path
    pub async fn open(path: &Path) -> Result<Self, EngineError>;
    
    /// Get this node's Iroh ID
    pub fn node_id(&self) -> NodeId;
    
    // === Realm Management ===
    
    /// Create a new private realm (also subscribes to gossip topic)
    pub async fn create_realm(&mut self, name: &str) -> Result<RealmId, EngineError>;
    
    /// List all realms (both private and shared)
    pub fn list_realms(&self) -> Vec<RealmInfo>;
    
    /// Get realm details
    pub fn get_realm(&self, id: &RealmId) -> Option<RealmInfo>;
    
    /// Delete a realm (leaves gossip topic for shared realms)
    pub async fn delete_realm(&mut self, id: &RealmId) -> Result<(), EngineError>;
    
    // === Task Management ===
    
    /// Add task to realm (auto-broadcasts via gossip if shared)
    pub async fn add_task(&mut self, realm_id: &RealmId, title: &str) -> Result<TaskId, EngineError>;
    
    /// Get all tasks in realm
    pub fn get_tasks(&self, realm_id: &RealmId) -> Vec<Task>;
    
    /// Toggle task completion (auto-broadcasts)
    pub async fn toggle_task(&mut self, realm_id: &RealmId, task_id: &TaskId) -> Result<(), EngineError>;
    
    /// Delete task (auto-broadcasts)
    pub async fn delete_task(&mut self, realm_id: &RealmId, task_id: &TaskId) -> Result<(), EngineError>;
    
    // === Sharing (Gossip-based) ===
    
    /// Create invite for realm (makes it shared, returns ticket with bootstrap peers)
    pub async fn create_invite(&mut self, realm_id: &RealmId) -> Result<InviteTicket, EngineError>;
    
    /// Join realm via invite ticket (joins gossip swarm, syncs history)
    pub async fn join_via_invite(&mut self, invite: InviteTicket) -> Result<RealmId, EngineError>;
    
    /// Get peers currently connected in a realm's gossip swarm
    pub fn get_realm_peers(&self, realm_id: &RealmId) -> Vec<PeerInfo>;
    
    // === Background Operations ===
    
    /// The engine automatically:
    /// - Maintains gossip connections for all shared realms
    /// - Broadcasts local changes
    /// - Receives and applies remote changes
    /// - Reconnects to peers on network changes
    
    /// Get current connection status
    pub fn connection_status(&self) -> ConnectionStatus;
    
    // === Events (for UI reactivity) ===
    
    /// Subscribe to realm changes (called when remote changes arrive via gossip)
    pub fn on_realm_change(&self, callback: impl Fn(RealmId) + Send + 'static);
    
    /// Subscribe to connection status changes
    pub fn on_connection_change(&self, callback: impl Fn(ConnectionStatus) + Send + 'static);
    
    /// Subscribe to peer join/leave events
    pub fn on_peer_change(&self, callback: impl Fn(RealmId, PeerEvent) + Send + 'static);
}

// === Supporting Types ===

#[derive(Clone, Debug)]
pub struct RealmInfo {
    pub id: RealmId,
    pub name: String,
    pub is_shared: bool,
    pub peer_count: usize,      // Active peers in gossip swarm
    pub task_count: usize,
    pub last_sync: Option<i64>, // Last time we received gossip
}

#[derive(Clone, Debug)]
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub completed: bool,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConnectionStatus {
    Offline,
    Connecting,
    Online { 
        relay_connected: bool,
        total_peers: usize,  // Across all realms
    },
}

#[derive(Clone, Debug)]
pub struct PeerInfo {
    pub node_id: NodeId,
    pub connected_since: i64,
    pub last_seen: i64,
}

#[derive(Clone, Debug)]
pub enum PeerEvent {
    Joined(NodeId),
    Left(NodeId),
}
```

---

## Summary: Claude Code Instructions

### Starting Point

1. **Study `iroh-automerge-repo` in the adjacent directory:**
   ```bash
   cd ../iroh-examples/iroh-automerge-repo
   cargo run
   # Run in two terminals to see sync in action
   ```

2. **Understand the key patterns:**
   - **Gossip topics** = realm discovery/broadcast
   - **TopicId** = 32-byte identifier (we derive from RealmId)
   - **Bootstrap peers** = how new nodes join the swarm
   - **Automerge sync** = per-document, runs over gossip messages

3. **Also study `iroh-gossip` patterns:**
   ```bash
   # Look at the chat example for topic subscription
   cd ../iroh-examples/iroh-gossip-chat
   less src/main.rs
   ```

4. **Port working code from the old project:**
   ```bash
   # These modules are tested and working - adapt them
   ls ../syncengine-tauri/src-tauri/src/
   # identity.rs, crypto.rs, storage.rs, types.rs
   ```

### Phase 1 Deliverables (Priority)

1. **Working CLI** that can:
   - Create realms (with gossip topic subscription)
   - Add/list/toggle/delete tasks (with gossip broadcast)
   - Generate and consume invite tickets (with bootstrap peers)
   - Stay connected to realm's gossip swarm

2. **Integration tests** proving:
   - Two nodes can join same gossip topic
   - Changes broadcast via gossip reach all subscribers
   - Three+ nodes all receive updates (not just point-to-point)
   - Invites include working bootstrap peer info

### First Steps (Do This First)

```bash
# 1. Create the new project structure
cd workspace
cargo new syncengine
cd syncengine
mkdir -p crates/syncengine-core/src/sync crates/syncengine-cli/src tests

# 2. Check iroh-examples for current dependency versions
cat ../iroh-examples/iroh-automerge-repo/Cargo.toml
# Use the same iroh and iroh-gossip versions they use!

# 3. Set up workspace Cargo.toml (update versions from step 2)
cat > Cargo.toml << 'EOF'
[workspace]
resolver = "2"
members = ["crates/*"]

[workspace.dependencies]
# Check ../iroh-examples/*/Cargo.toml for current versions!
automerge = "0.5"
iroh = "0.95"           # ← Verify against iroh-examples
iroh-gossip = "0.95"    # ← Verify against iroh-examples
redb = "2.0"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
postcard = { version = "1", features = ["alloc"] }
blake3 = "1"
bs58 = "0.5"
anyhow = "1"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
EOF

# 4. Copy working code from old project
cp ../syncengine-tauri/src-tauri/src/identity.rs crates/syncengine-core/src/
cp ../syncengine-tauri/src-tauri/src/crypto.rs   crates/syncengine-core/src/
cp ../syncengine-tauri/src-tauri/src/storage.rs  crates/syncengine-core/src/
cp ../syncengine-tauri/src-tauri/src/types.rs    crates/syncengine-core/src/

# 5. Copy this spec as CLAUDE.md
cp /path/to/SYNCHRONICITY_V2_SPEC.md CLAUDE.md

# 6. Run iroh-automerge-repo to understand it
cd ../iroh-examples/iroh-automerge-repo
cargo run

# 7. Start with the first test (Milestone 1)
cd ../syncengine
# Write test_gossip_echo in tests/p2p_integration.rs
# Make it pass
```

**IMPORTANT**: The iroh ecosystem evolves rapidly. Always check `../iroh-examples/iroh-automerge-repo/Cargo.toml` for the working dependency versions before starting.

### Architecture Rules

- All sync logic in `syncengine-core` crate (reusable by CLI and Dioxus)
- Use `iroh-gossip` for message broadcast, not point-to-point connections
- Each realm = one gossip topic + one Automerge document
- Invite tickets contain TopicId + bootstrap peers + encryption key
- TDD: write tests before implementation

### Key Difference from Simple iroh-automerge

The simple `iroh-automerge` example uses point-to-point QUIC streams. We use `iroh-automerge-repo` patterns instead:

```
iroh-automerge (DON'T USE):
  Node A ←──stream──→ Node B
  
iroh-automerge-repo (USE THIS):
  Node A ──┐
           ├── Gossip Topic ──→ All subscribers receive
  Node B ──┤
  Node C ──┘
```

### Success Criteria

```bash
# Terminal 1: Create realm and serve
$ syncengine-cli realm create "Shared List"
Created realm: realm_abc...

$ syncengine-cli task add realm_abc "Buy milk"
$ syncengine-cli invite create realm_abc
sync-invite:xyz789...

# Terminal 2: Join via invite
$ syncengine-cli invite join sync-invite:xyz789...
Joined "Shared List" (1 task)

$ syncengine-cli task list realm_abc
[ ] Buy milk

# Add task from Terminal 2
$ syncengine-cli task add realm_abc "Buy eggs"

# Terminal 1 sees it automatically (gossip broadcast)
$ syncengine-cli task list realm_abc
[ ] Buy milk
[ ] Buy eggs  ← Received via gossip!

# Terminal 3: Join the same realm (tests 3-node gossip)
$ syncengine-cli invite join sync-invite:xyz789...
Joined "Shared List" (2 tasks)
[ ] Buy milk
[ ] Buy eggs

$ syncengine-cli task add realm_abc "Buy bread"

# Both Terminal 1 and 2 receive it via gossip
```

When this works with **three nodes**, gossip is proven. Then build Dioxus UI on top.

---

*Document version: 2.0*
*Last updated: January 2026*
*Foundation: iroh-automerge-repo + iroh-gossip*
*Target: Claude Code implementation*

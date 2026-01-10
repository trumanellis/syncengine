# Architecture Decision Records

> Document version: 1.0
> Last updated: January 2026

This document captures key architectural decisions for Synchronicity Engine's core library.

---

## Table of Contents

1. [ADR-001: Gossip Topology (Mesh)](#adr-001-gossip-topology-mesh)
2. [ADR-002: Automerge Sync Protocol](#adr-002-automerge-sync-protocol)
3. [ADR-003: Encryption Strategy](#adr-003-encryption-strategy)
4. [ADR-004: TopicId Derivation](#adr-004-topicid-derivation)
5. [ADR-005: Invite Security Model](#adr-005-invite-security-model)
6. [ADR-006: Message Signing](#adr-006-message-signing)

---

## ADR-001: Gossip Topology (Mesh)

### Status
Accepted

### Context
Synchronicity Engine needs to sync data between multiple peers without a central server. Two primary patterns exist in the iroh ecosystem:

1. **Point-to-point streams** (iroh-automerge-repo style): Direct QUIC connections between known peers
2. **Gossip broadcast** (iroh-gossip style): Topic-based pub/sub with automatic peer discovery

### Decision
Use **mesh gossip topology** via `iroh-gossip` for realm synchronization.

### Rationale

| Factor | Point-to-Point | Gossip (Chosen) |
|--------|----------------|-----------------|
| Peer discovery | Manual | Automatic via topic |
| Scaling | O(n^2) connections | O(n) per node |
| Late joiner sync | Requires known peer | Any topic member |
| Offline resilience | Requires specific peer online | Any peer can relay |
| NAT traversal | Per-connection | Per-topic |

Gossip is superior for:
- Dynamic group membership (users join/leave realms freely)
- Censorship resistance (no single peer is critical)
- Offline-first design (changes propagate when any path exists)

### Consequences
- Must implement custom Automerge-over-gossip protocol
- Cannot directly use `samod` library (designed for point-to-point)
- Need to handle gossip message ordering and deduplication
- Higher message overhead (broadcast vs targeted)

### Implementation Notes
```rust
// Each realm = one gossip topic
let topic = realm_id.to_topic_id();
let (sender, receiver) = gossip.subscribe(topic, bootstrap_peers).await?;

// Broadcast changes to all topic subscribers
sender.broadcast(sync_message).await?;
```

---

## ADR-002: Automerge Sync Protocol

### Status
Accepted

### Context
Automerge provides multiple sync strategies:
1. **Full document sync**: Send entire document state
2. **Incremental sync protocol**: Exchange sync messages to converge
3. **Change-based broadcast**: Send individual changes as they occur

The `samod` library implements the official automerge-repo protocol but requires bidirectional streams.

### Decision
Implement **hybrid change broadcast with on-demand full sync**.

### Protocol Design

```
┌─────────────────────────────────────────────────────────────────┐
│                    Gossip Message Types                         │
├─────────────────────────────────────────────────────────────────┤
│  1. Changes       - Broadcast new changes (common case)         │
│  2. SyncRequest   - Request sync from specific peer             │
│  3. SyncMessage   - Automerge sync protocol message (targeted)  │
│  4. Heads         - Announce document heads (peer discovery)    │
└─────────────────────────────────────────────────────────────────┘
```

### Rationale
- **Changes broadcast**: Efficient for real-time updates (small, frequent)
- **SyncRequest/SyncMessage**: Handles late joiners needing full history
- **Heads announcement**: Allows peers to detect if they're behind

### Wire Format
```rust
#[derive(Serialize, Deserialize)]
pub enum GossipSyncMessage {
    /// Broadcast new changes (no specific recipient)
    Changes {
        /// Serialized Automerge changes
        changes: Vec<u8>,
        /// Heads after applying these changes
        heads: Vec<ChangeHash>,
    },
    /// Request sync from any peer with these heads
    SyncRequest {
        /// Requester's current heads
        heads: Vec<ChangeHash>,
    },
    /// Targeted sync message (respects automerge sync protocol)
    SyncMessage {
        /// Intended recipient (optimization, others ignore)
        to: Option<PublicKey>,
        /// Automerge sync message bytes
        message: Vec<u8>,
    },
    /// Periodic heads announcement for peer discovery
    Heads {
        heads: Vec<ChangeHash>,
    },
}
```

### Consequences
- More complex than pure point-to-point sync
- Requires deduplication of changes (same change via multiple paths)
- Need to implement "catch-up" logic for late joiners
- Can't use `samod` directly; must wrap Automerge ourselves

---

## ADR-003: Encryption Strategy

### Status
Accepted

### Context
The system needs two types of encryption:
1. **In-transit**: Protect data while traveling over network
2. **At-rest**: Protect data stored on disk

### Decision

| Layer | Encryption | Key Management |
|-------|------------|----------------|
| Transport (QUIC) | TLS 1.3 via iroh | Automatic, per-connection |
| Application (gossip messages) | ChaCha20-Poly1305 | Per-realm symmetric key |
| Storage | ChaCha20-Poly1305 | Derived from user password (Phase 2+) |

### Rationale

**Why application-layer encryption?**
- Relay servers (if used) see only ciphertext
- Compromised peer can't read other realms
- Realm keys can be rotated independent of transport

**Why ChaCha20-Poly1305?**
- Fast on devices without AES hardware acceleration
- AEAD (authenticated encryption with associated data)
- Well-studied, no known vulnerabilities
- Used by WireGuard, TLS 1.3

### Implementation
```rust
pub struct RealmCrypto {
    cipher: ChaCha20Poly1305,
}

impl RealmCrypto {
    /// Encrypt message for gossip broadcast
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let nonce = generate_random_nonce(); // 12 bytes
        let ciphertext = self.cipher.encrypt(&nonce, plaintext)?;

        // Prepend nonce to ciphertext
        let mut result = nonce.to_vec();
        result.extend(ciphertext);
        Ok(result)
    }
}
```

### Consequences
- All sync messages encrypted before gossip broadcast
- Invite tickets must include realm key
- Key rotation requires re-keying mechanism (Phase 2+)
- Storage encryption adds complexity (deferred to Phase 2)

---

## ADR-004: TopicId Derivation

### Status
Accepted

### Context
`iroh-gossip` uses `TopicId` (32 bytes) to identify gossip topics. We need to derive `TopicId` from `RealmId` deterministically.

### Decision
Use **blake3 keyed hash with domain separation**.

### Implementation
```rust
impl RealmId {
    /// Domain separation key (32 bytes, null-padded)
    const TOPIC_DOMAIN: &'static [u8; 32] = b"syncengine/realm/topic/v1\0\0\0\0\0\0";

    /// Derive TopicId from RealmId
    pub fn to_topic_id(&self) -> TopicId {
        let hash = blake3::keyed_hash(Self::TOPIC_DOMAIN, self.as_bytes());
        TopicId::from_bytes(*hash.as_bytes())
    }
}
```

### Rationale

**Why keyed hash instead of direct copy?**
1. **Domain separation**: Prevents collision with other iroh-gossip applications
2. **Version support**: Can change derivation in future versions
3. **No information leakage**: TopicId doesn't reveal RealmId directly

**Why blake3?**
- Already a dependency (used elsewhere in codebase)
- Fast, modern, secure
- Native keyed hash support

### Collision Analysis
- 32-byte output space = 2^256 possible topics
- Birthday bound: ~2^128 realms before expected collision
- Practically impossible to collide accidentally

### Consequences
- Cannot reverse TopicId to RealmId (by design)
- Must store RealmId -> TopicId mapping if needed
- Topic format is versioned for future changes

---

## ADR-005: Invite Security Model

### Status
Proposed (needs implementation)

### Context
Invite tickets enable joining realms. Current spec includes:
- TopicId (to find gossip swarm)
- Realm key (for encryption)
- Bootstrap peers (for initial connection)

### Security Concerns Identified

1. **No authentication**: Anyone with invite can join
2. **No revocation**: Once shared, invite is valid until expiry
3. **Replay attacks**: Invite can be reused by interceptor
4. **Information leakage**: Bootstrap peer IPs visible

### Decision
Implement **signed, auditable invites** with optional single-use.

### Recommended Structure
```rust
#[derive(Serialize, Deserialize)]
pub struct InviteTicket {
    /// Protocol version for compatibility
    pub version: u8,
    /// Unique identifier for this invite (for tracking/revocation)
    pub invite_id: [u8; 16],
    /// Gossip topic for this realm
    pub topic: TopicId,
    /// Realm encryption key
    pub realm_key: [u8; 32],
    /// Bootstrap peers (at least one required)
    pub bootstrap_peers: Vec<NodeAddr>,
    /// Human-readable realm name (for UI)
    pub realm_name: Option<String>,
    /// Creation timestamp (Unix epoch seconds)
    pub created_at: i64,
    /// Expiration timestamp (None = no expiry)
    pub expires_at: Option<i64>,
    /// Maximum number of uses (None = unlimited)
    pub max_uses: Option<u32>,
    /// Creator's hybrid signature over all above fields
    pub creator_signature: HybridSignature,
    /// Creator's DID (for signature verification)
    pub creator_did: Did,
}
```

### Verification Steps
1. Check `version` is supported
2. Verify `creator_signature` against `creator_did`
3. Check `created_at <= now <= expires_at`
4. Check usage count if `max_uses` set
5. Verify at least one bootstrap peer is reachable

### Future Considerations
- Invite revocation list (shared via gossip)
- Capability-based invites (read-only, time-limited, etc.)
- Key derivation from invite + recipient identity

### Consequences
- Larger invite tickets (~500+ bytes vs ~200 bytes)
- Requires identity system (Phase 2) for signing
- Must track invite usage for max_uses enforcement

---

## ADR-006: Message Signing

### Status
Accepted

### Context
Gossip messages can come from any peer. Without signatures:
- Messages can be spoofed
- Malicious peers can inject fake data
- No attribution for changes

### Decision
All gossip messages are **signed and verified** before processing.

### Implementation Pattern (from browser-chat example)
```rust
#[derive(Serialize, Deserialize)]
pub struct SignedMessage {
    /// Sender's public key
    pub from: PublicKey,
    /// Serialized and versioned message content
    pub data: Vec<u8>,
    /// Ed25519 signature over data
    pub signature: Signature,
}

impl SignedMessage {
    pub fn sign_and_encode(
        secret_key: &SecretKey,
        message: impl Serialize,
        timestamp: u64,
    ) -> Result<Vec<u8>, Error> {
        let wire = WireMessage::V0 { timestamp, message };
        let data = postcard::to_stdvec(&wire)?;
        let signature = secret_key.sign(&data);
        let signed = Self {
            from: secret_key.public(),
            data,
            signature,
        };
        Ok(postcard::to_stdvec(&signed)?)
    }

    pub fn verify_and_decode<T: DeserializeOwned>(
        bytes: &[u8],
    ) -> Result<(PublicKey, u64, T), Error> {
        let signed: Self = postcard::from_bytes(bytes)?;
        signed.from.verify(&signed.data, &signed.signature)?;
        let wire: WireMessage<T> = postcard::from_bytes(&signed.data)?;
        let WireMessage::V0 { timestamp, message } = wire;
        Ok((signed.from, timestamp, message))
    }
}

#[derive(Serialize, Deserialize)]
pub enum WireMessage<T> {
    V0 { timestamp: u64, message: T },
}
```

### Rationale
- **Public key in message**: Allows verification without prior key exchange
- **Timestamp**: Enables replay detection and message ordering
- **Version envelope**: Future-proof wire format

### Consequences
- ~64 bytes overhead per message (signature)
- Verification on every received message (CPU cost)
- Iroh's Ed25519 keys used (not hybrid quantum - that's for identity, not transport)

---

## Appendix: Patterns from iroh-examples

### From browser-chat (RECOMMENDED)
- `SignedMessage` pattern for gossip messages
- `ChatTicket` as example of serializable invite
- Presence heartbeat for peer discovery
- Event stream pattern for UI integration

### From iroh-automerge-repo (REFERENCE ONLY)
- Uses `samod` library - NOT directly applicable (point-to-point only)
- QUIC bidirectional streams for sync
- `IrohRepo` as protocol handler pattern
- `LengthDelimitedCodec` for framing

### Key Difference
```
iroh-automerge-repo: Node A <──QUIC stream──> Node B
browser-chat:        Node A ──┐
                              ├── Gossip Topic ──> All subscribers
                     Node B ──┘

Synchronicity Engine uses browser-chat pattern, NOT iroh-automerge-repo pattern.
```

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01 | Architecture Review Agent | Initial ADRs |

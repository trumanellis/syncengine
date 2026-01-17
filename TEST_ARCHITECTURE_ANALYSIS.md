# Contact Exchange Test Architecture Analysis

## Current Test Landscape

### Test Files Overview

| File | Type | Network? | Purpose |
|------|------|----------|---------|
| `contact_manager.rs` (unit tests) | Unit | ❌ No | Test ContactManager in isolation |
| `contact_integration.rs` | Integration | ❌ No | Test SyncEngine API without network |
| `contact_e2e_test.rs` | E2E | ✅ Yes | Test full QUIC communication between nodes |

### Current Test Results

```
contact_manager.rs (unit tests):
  ✅ 45 tests PASSED
  ❌ 2 tests FAILED:
     - test_send_contact_request (line 977)
     - test_accept_contact_request (line 1003)

contact_integration.rs:
  ✅ 7 tests PASSED
  ❌ 1 test FAILED:
     - test_accept_contact_request (line 60)

contact_e2e_test.rs:
  ✅ ALL tests PASSED (3 tests)
     - test_two_engines_exchange_contacts_over_quic
     - test_contact_request_declined_over_quic
     - test_contact_topic_and_key_derivation_match
```

---

## Failure Root Causes

### Failure 1: `contact_manager.rs::test_send_contact_request` (line 977-1000)

**Problem**: Creates ONE manager, generates invite from it, then tries to send request to ITSELF.

```rust
let (manager, _temp) = create_test_manager().await;  // ONE manager
let invite_code = manager.generate_invite(profile.clone(), 24).unwrap();
let invite = manager.decode_invite(&invite_code).unwrap();

// Tries to connect to self!
manager.send_contact_request(invite.clone(), our_profile).await.unwrap();
```

**Error**: `Network("Failed to connect to inviter: Connecting to ourself is not supported")`

**Why**: The invite contains the manager's own NodeId. Iroh's QUIC layer correctly rejects self-connections.

---

### Failure 2: `contact_manager.rs::test_accept_contact_request` (line 1003-1029)

**Problem**: Uses fake `NodeAddrBytes::new([0u8; 32])` with no valid addressing information.

```rust
let pending = PendingContact {
    // ...
    node_addr: NodeAddrBytes::new([0u8; 32]),  // ← Invalid address
    // ...
};

// Tries to send QUIC message to invalid address
manager.accept_contact_request(&invite_id).await.unwrap();
```

**Error**: `Network("Failed to connect to requester: No addressing information available")`

**Why**: `accept_contact_request()` calls `send_contact_response()` and `send_contact_accepted()`, which attempt QUIC connections to the peer's address. The zeroed bytes have no relay URL, no direct addresses, and an invalid public key.

---

### Failure 3: `contact_integration.rs::test_accept_contact_request` (line 60-106)

**Problem**: Same as Failure 2 - uses `NodeAddrBytes::new([0u8; 32])`.

```rust
node_addr: syncengine_core::invite::NodeAddrBytes::new([0u8; 32]),  // Line 82
```

**Error**: Same as Failure 2

**Note**: `test_send_contact_request` in this file PASSES because it creates TWO separate engines (Alice and Bob), so it's not connecting to itself. However, it doesn't call `start_networking()`, so the actual QUIC connection likely fails gracefully or the test doesn't verify the network path.

---

## Test Architecture Principles

### Three Layers of Testing

```
┌────────────────────────────────────────────────────────┐
│  Layer 3: E2E Tests (contact_e2e_test.rs)            │
│  - Full SyncEngine instances with networking          │
│  - Actual QUIC connections over network               │
│  - Test complete flow including network propagation   │
│  - Slower, but validates real-world behavior          │
├────────────────────────────────────────────────────────┤
│  Layer 2: Integration Tests (contact_integration.rs) │
│  - SyncEngine API without networking                  │
│  - Storage operations and state transitions           │
│  - Event emission and subscription                    │
│  - Fast, focused on API contracts                     │
├────────────────────────────────────────────────────────┤
│  Layer 1: Unit Tests (contact_manager.rs)            │
│  - ContactManager in isolation                        │
│  - Pure logic (invite generation, validation, etc.)   │
│  - No network operations                              │
│  - Very fast, focused on single methods              │
└────────────────────────────────────────────────────────┘
```

### The Problem: Blurred Boundaries

The failing tests are **unit/integration tests trying to do network operations**:

- They test the network code path (send_contact_request, accept_contact_request)
- But they don't set up proper network infrastructure
- They use invalid fixtures (self-connection, fake addresses)

This is like testing an HTTP client without running a server.

---

## Solution: Three Approaches

### Approach 1: Move to E2E Tests ✅ RECOMMENDED

**Strategy**: Remove network tests from unit/integration layers, rely on E2E tests for network validation.

**Implementation**:
1. Delete or `#[ignore]` the failing unit tests in `contact_manager.rs`
2. Fix `contact_integration.rs::test_accept_contact_request` by NOT calling `accept_contact()` (which does network operations)
3. Rely on existing E2E tests for network validation

**Pros**:
- Clean separation of concerns
- Tests are in the right place
- E2E tests already cover these flows

**Cons**:
- Less granular network testing
- Slower to run network tests

---

### Approach 2: Add Proper Network Setup to Unit Tests ⚠️ COMPLEX

**Strategy**: Make the failing unit tests into proper E2E tests by adding `start_networking()`.

**Implementation**:
1. In `contact_manager.rs::test_send_contact_request`:
   ```rust
   // Create TWO separate managers with different endpoints
   let (manager_alice, _temp_alice) = create_test_manager().await;
   manager_alice.gossip_sync.endpoint().start_networking(...);

   let (manager_bob, _temp_bob) = create_test_manager().await;
   manager_bob.gossip_sync.endpoint().start_networking(...);

   // Generate invite from Alice
   let invite = manager_alice.generate_invite(...);

   // Bob sends request to Alice
   manager_bob.send_contact_request(invite, ...).await.unwrap();
   ```

2. Similar for `accept_contact_request` - create two managers, exchange invites, accept.

**Pros**:
- Tests the actual network code
- More coverage at unit level

**Cons**:
- Complex setup for unit tests
- Duplicates E2E test coverage
- Slower test execution
- Requires port management for parallel tests

---

### Approach 3: Mock Network Layer 🔧 PRAGMATIC

**Strategy**: Test storage/logic without network operations.

**Implementation**:

For `test_send_contact_request`:
```rust
#[tokio::test]
async fn test_send_contact_request_storage_only() {
    let (manager, _temp) = create_test_manager().await;

    // Create valid invite from DIFFERENT fake DID
    let invite = create_fake_invite_from_different_peer();
    let our_profile = create_test_profile("Alice");

    // Instead of calling send_contact_request (which does network):
    // Manually create the pending contact (simulating what send_contact_request does)
    let pending = PendingContact {
        invite_id: invite.invite_id,
        peer_did: invite.inviter_did.clone(),
        profile: invite.profile_snapshot.clone(),
        node_addr: invite.node_addr,
        state: ContactState::OutgoingPending,
        created_at: chrono::Utc::now().timestamp(),
    };

    manager.storage.save_pending(&pending).unwrap();

    // Verify storage worked
    let loaded = manager.storage.load_pending(&invite.invite_id).unwrap();
    assert!(loaded.is_some());
}
```

For `test_accept_contact_request`:
```rust
#[tokio::test]
async fn test_accept_contact_request_state_transition() {
    let (manager, _temp) = create_test_manager().await;

    // Create incoming pending
    let pending = create_fake_incoming_pending();
    manager.storage.save_pending(&pending).unwrap();

    // Instead of calling accept_contact (which does network):
    // Manually transition to accepted state
    manager.storage.delete_pending(&pending.invite_id).unwrap();

    let contact = ContactInfo {
        peer_did: pending.peer_did.clone(),
        // ... fill in fields ...
        state: ContactState::MutuallyAccepted,
    };
    manager.storage.save_contact(&contact).unwrap();

    // Verify state transition
    assert!(manager.storage.load_pending(&pending.invite_id).unwrap().is_none());
    assert!(manager.storage.load_contact(&pending.peer_did).unwrap().is_some());
}
```

**Pros**:
- Tests run fast
- No network complexity
- Tests actual storage/logic concerns
- Doesn't duplicate E2E coverage

**Cons**:
- Doesn't test network code path
- Requires refactoring tests
- Some duplication of logic

---

## Recommended Solution

### Phase 1: Quick Fix (Immediate)

1. **Mark failing tests as `#[ignore]`** with a comment explaining they need proper multi-node setup:
   ```rust
   #[tokio::test]
   #[ignore = "Needs proper multi-node setup with separate endpoints"]
   async fn test_send_contact_request() { ... }
   ```

2. **Fix `contact_integration.rs::test_accept_contact_request`**:
   - Change to test ONLY storage operations (don't call `accept_contact()`)
   - Rename to `test_accept_contact_request_storage`

### Phase 2: Architectural Cleanup (Next Sprint)

1. **Create test utilities** in `contact_manager.rs`:
   ```rust
   // Helper to create multi-node test setup
   async fn create_two_node_setup() -> (ContactManager, ContactManager, TempDir, TempDir) {
       // ... setup two managers with networking ...
   }
   ```

2. **Add focused E2E tests** for specific scenarios:
   - `test_network_connection_failure_handling`
   - `test_simultaneous_contact_requests`
   - `test_network_timeout_behavior`

3. **Document test boundaries** in module comments:
   ```rust
   //! # Test Architecture
   //!
   //! Unit tests in this module do NOT perform network operations.
   //! For network integration tests, see `tests/contact_e2e_test.rs`.
   ```

---

## Implementation Plan

### Step 1: Immediate Fix

```bash
# Mark failing unit tests as ignored
vim crates/syncengine-core/src/sync/contact_manager.rs
# Add #[ignore] to test_send_contact_request and test_accept_contact_request

# Fix integration test
vim crates/syncengine-core/tests/contact_integration.rs
# Refactor test_accept_contact_request to not call accept_contact()
```

### Step 2: Verify

```bash
cargo test -p syncengine-core contact --lib
# Should show: 47 tests passed, 0 failed, 2 ignored

cargo test -p syncengine-core --test contact_integration
# Should show: 8 tests passed, 0 failed

cargo test -p syncengine-core --test contact_e2e_test
# Should show: 3 tests passed, 0 failed
```

### Step 3: Document

Add to `crates/syncengine-core/tests/README.md`:
```markdown
# Test Organization

## Unit Tests (src/**/*.rs)
- Test logic in isolation
- No network operations
- Use fake/mock data

## Integration Tests (tests/contact_integration.rs)
- Test SyncEngine API
- Storage and state management
- No actual network communication

## E2E Tests (tests/contact_e2e_test.rs)
- Full multi-node setup
- Real QUIC connections
- Network propagation delays
- Complete user flows
```

---

## Key Insights

### Why E2E Tests Pass But Unit Tests Fail

**E2E tests** (`contact_e2e_test.rs`) work because:
1. They create TWO separate `SyncEngine` instances
2. They call `start_networking()` on BOTH
3. They use real sleeps for network propagation
4. They test actual QUIC connections

**Unit tests** (`contact_manager.rs`) fail because:
1. They create ONE manager (self-connection)
2. They use fake addresses (`[0u8; 32]`)
3. They don't start networking infrastructure
4. They attempt network operations without proper setup

### The Test Pyramid

```
        /\
       /E2E\      ← Slow, comprehensive (3 tests)
      /______\
     /        \
    /Integration\ ← Medium speed, API focused (8 tests)
   /__________\
  /            \
 /   Unit Tests  \ ← Fast, isolated logic (45+ tests)
/________________\
```

**Current problem**: Some unit tests are trying to be E2E tests.

**Solution**: Respect the pyramid - each layer tests appropriate concerns.

---

## Next Actions

✅ **Option A (Recommended)**: Implement Phase 1 Quick Fix
- Mark 2 unit tests as `#[ignore]`
- Fix 1 integration test to not do network operations
- Document test architecture

⏸️ **Option B**: Implement Approach 3 (Mock Network)
- Refactor failing tests to test storage only
- More work but cleaner architecture

🚀 **Option C**: Implement Approach 2 (Full Multi-Node)
- Build proper multi-node test infrastructure
- Most comprehensive but most complex

**Recommendation**: Start with Option A for immediate unblocking, then plan Option B or C for next iteration.

# Synchronicity Engine — Worker Instructions

> **Role**: Feature implementation with TDD
> **Scope**: This repository only
> **You are**: A worker implementing a specific feature

---

## Worktrees for Feature Development

**Use git worktrees for significant new features.** This keeps `main` stable and allows parallel experimentation.

### When to Use a Worktree

| Use Worktree | Stay on Main |
|--------------|--------------|
| New UI components (e.g., new card types) | Bug fixes |
| Major refactors | CSS tweaks |
| Experimental features | Small enhancements |
| Multi-file architectural changes | Single-file edits |
| Anything you might want to abandon | Quick iterations |

### Creating a Worktree

```bash
# From the main syncengine directory
git worktree add ../syncengine-wt-<feature-name> -b feature/<feature-name>

# Example: new vertical card component
git worktree add ../syncengine-wt-vertical-cards -b feature/vertical-quest-cards
```

### Working in a Worktree

```bash
# Navigate to the worktree
cd ../syncengine-wt-<feature-name>

# Work normally - commits go to the feature branch
cargo build
cargo test
git add -A && git commit -m "feat: implement feature"
```

### Merging Back to Main

```bash
# From main directory
cd /path/to/syncengine
git checkout main
git merge feature/<feature-name>

# Clean up
git worktree remove ../syncengine-wt-<feature-name>
git branch -d feature/<feature-name>
```

### Listing Worktrees

```bash
git worktree list
```

**Recommendation:** When the user asks for a significant new feature, suggest creating a worktree first. This protects main from experimental code and makes it easy to compare approaches or abandon failed experiments.

---

## Development Workflow: TDD → CLI → UI

**The core principle:** All logic lives in `syncengine-core`. CLI and Dioxus are **thin wrappers** that call core functions.

```
┌─────────────────────────────────────────────────────────────────┐
│  1. CORE FUNCTIONS (syncengine-core crate)                      │
│     - ALL business logic lives here                             │
│     - Write failing unit test                                   │
│     - Implement minimum code to pass                            │
│     - Refactor                                                  │
│     - This is where the REAL code is                            │
├─────────────────────────────────────────────────────────────────┤
│  2. CLI WRAPPER (syncengine-cli crate)                          │
│     - Thin wrapper: parse args → call core function → print     │
│     - NO business logic in CLI                                  │
│     - E2E tests verify the wiring works                         │
├─────────────────────────────────────────────────────────────────┤
│  3. DIOXUS WRAPPER (Phase 3)                                    │
│     - Thin wrapper: UI event → call core function → update UI   │
│     - NO business logic in UI                                   │
│     - Same core functions as CLI                                │
│     - ★ MUST follow DESIGN_SYSTEM.md aesthetic                  │
└─────────────────────────────────────────────────────────────────┘
```

**Why this matters:**
- Core functions are unit-testable in isolation
- CLI and UI are interchangeable frontends to the same engine
- If it works in CLI, it will work in Dioxus (same function calls)
- 95% of your tests should be in `syncengine-core`

**The pattern:**
```rust
// syncengine-core/src/engine.rs — The REAL implementation
impl SyncEngine {
    pub async fn create_realm(&mut self, name: &str) -> Result<RealmId, SyncError> {
        // All the actual logic here
    }
}

// syncengine-cli/src/main.rs — Thin wrapper
Commands::RealmCreate { name } => {
    let id = engine.create_realm(&name).await?;  // Just call core
    println!("Created realm: {}", id);            // Just print result
}

// src/components/realm.rs (Dioxus) — Thin wrapper
let create_realm = move |_| {
    let id = engine.write().create_realm(&name).await.unwrap();  // Just call core
    // UI updates automatically via signals
};
```

---

## UI Design (Phase 3 Only)

**When working on Dioxus UI, you MUST follow `DESIGN_SYSTEM.md`.**

The aesthetic is **cyber-mystical terminal** — sacred geometry meets command line.

**Quick reference:**
- Background: `#0a0a0a` (void black)
- Gold `#d4af37`: Sacred terms, titles  
- Cyan `#00d4aa`: Tech terms, links, focus states
- Moss `#7cb87c`: Status dots, borders
- Fonts: `Cormorant Garamond` (titles), `JetBrains Mono` (body)

**Sacred language:**
| Don't say | Say instead |
|-----------|-------------|
| Create task | Manifest new intention |
| Connected | Field resonating |
| Loading | Synchronicities are forming |
| Login | Enter the Field |
| Task | Intention |
| Delete | Dissolve |

Read `DESIGN_SYSTEM.md` completely before writing any UI code.

---

## TDD Best Practices

### The Cycle

```bash
# 1. Write a failing test
cargo test test_feature_name -- --nocapture
# Should fail with: "not yet implemented" or assertion error

# 2. Write minimum code to pass
cargo test test_feature_name
# Should pass

# 3. Refactor if needed
cargo test  # ALL tests should still pass
cargo clippy  # No warnings

# 4. Commit
git add -A && git commit -m "feat: implement <feature>"
```

### Test File Organization

**95% of tests live in syncengine-core** (where the logic is):

```
crates/syncengine-core/                 # ← MOST TESTS HERE
├── src/
│   ├── lib.rs
│   ├── engine.rs         # #[cfg(test)] mod tests — engine unit tests
│   ├── realm.rs          # #[cfg(test)] mod tests — realm unit tests
│   └── sync/
│       └── gossip.rs     # #[cfg(test)] mod tests — gossip unit tests
│
└── tests/                # Integration tests (multi-module, still in core)
    └── p2p_integration.rs    # Two+ node sync tests

crates/syncengine-cli/
└── tests/                # ← MINIMAL: just wiring tests
    └── cli_scenarios.rs      # Verify CLI calls core correctly
```

**Rule of thumb:**
- Testing logic? → `syncengine-core`
- Testing that CLI parses args and calls the right function? → `syncengine-cli/tests`

### Unit Test Template

```rust
// At bottom of each source file
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_basic() {
        // Arrange
        let input = ...;
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected);
    }

    #[test]
    fn test_feature_edge_case() {
        // Test boundary conditions
    }

    #[test]
    fn test_feature_error_case() {
        // Test error handling
        let result = function_that_can_fail(bad_input);
        assert!(result.is_err());
    }
}
```

### Async Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_async_feature() {
        // Arrange
        let node = TestNode::new().await;
        
        // Act
        let result = node.do_async_thing().await;
        
        // Assert
        assert!(result.is_ok());
    }
}
```

### What Makes a Good Test

| ✅ Good | ❌ Bad |
|---------|--------|
| Tests one thing | Tests multiple behaviors |
| Descriptive name: `test_add_task_to_empty_realm` | Vague name: `test_task` |
| Clear arrange/act/assert | Logic scattered throughout |
| Fast (< 100ms for unit tests) | Slow (network calls, sleeps) |
| Deterministic | Flaky (timing-dependent) |
| Tests behavior, not implementation | Tests private internals |

---

## CLI Testing

### The CLI is a Thin Wrapper

The CLI should contain **zero business logic**. It only:
1. Parses command-line arguments
2. Calls a core function
3. Prints the result

```rust
// ❌ BAD: Logic in CLI
Commands::RealmCreate { name } => {
    // DON'T DO THIS - validation logic belongs in core
    if name.is_empty() {
        return Err(anyhow!("Name cannot be empty"));
    }
    let id = RealmId::new();
    storage.save_realm(&id, &name)?;
    println!("Created: {}", id);
}

// ✅ GOOD: CLI just calls core
Commands::RealmCreate { name } => {
    let id = engine.create_realm(&name).await?;  // Core handles everything
    println!("Created realm: {}", id);
}
```

### Adding a CLI Command

```rust
// crates/syncengine-cli/src/main.rs

#[derive(Subcommand)]
enum Commands {
    // Add new command here
    NewFeature {
        #[arg(short, long)]
        param: String,
    },
}

// In match block — just call core and print:
Commands::NewFeature { param } => {
    let result = engine.new_feature(&param).await?;
    println!("{}", result);
}
```

### CLI Integration Test

These tests verify the **wiring**, not the logic (logic is tested in core):

```rust
// tests/cli_scenarios.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_new_feature_cli() {
    let temp_dir = tempfile::tempdir().unwrap();
    
    Command::cargo_bin("syncengine-cli")
        .unwrap()
        .args(["--data-dir", temp_dir.path().to_str().unwrap()])
        .args(["new-feature", "--param", "value"])
        .assert()
        .success()
        .stdout(predicate::str::contains("expected output"));
}
```

### Manual CLI Testing

```bash
# Build and run
cargo run -p syncengine-cli -- <command>

# With debug output
RUST_LOG=debug cargo run -p syncengine-cli -- <command>

# Two terminals for P2P testing
# Terminal 1:
cargo run -p syncengine-cli -- serve

# Terminal 2:
cargo run -p syncengine-cli -- connect <node-id>
```

---

## Code Patterns

### Error Handling

Use `thiserror` for library errors, `anyhow` for CLI:

```rust
// crates/syncengine-core/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("Realm not found: {0}")]
    RealmNotFound(RealmId),
    
    #[error("Gossip error: {0}")]
    Gossip(#[from] iroh_gossip::Error),
    
    #[error("Storage error: {0}")]
    Storage(#[from] redb::Error),
}

// In CLI (main.rs)
use anyhow::Result;

fn main() -> Result<()> {
    // anyhow auto-converts SyncError
}
```

### Async Patterns

```rust
// Prefer explicit async over blocking
pub async fn do_thing(&self) -> Result<(), SyncError> {
    // Use tokio::spawn for background tasks
    tokio::spawn(async move {
        // Long-running work
    });
    
    // Use select for concurrent operations
    tokio::select! {
        result = operation_a() => { ... }
        result = operation_b() => { ... }
    }
}
```

### Logging

```rust
use tracing::{info, debug, warn, error};

pub async fn sync_realm(&self, realm_id: &RealmId) -> Result<()> {
    debug!(?realm_id, "Starting sync");
    
    match self.do_sync(realm_id).await {
        Ok(count) => {
            info!(?realm_id, count, "Sync complete");
            Ok(())
        }
        Err(e) => {
            error!(?realm_id, error = ?e, "Sync failed");
            Err(e)
        }
    }
}
```

---

## File Modification Rules

### Safe to Modify

- Anything in `crates/syncengine-core/src/sync/` (new code)
- `crates/syncengine-cli/src/main.rs` (add commands)
- Test files
- `Cargo.toml` (add dependencies)

### Modify Carefully

These were copied from the old project and mostly work:

- `identity.rs` — Only fix imports, don't change crypto logic
- `crypto.rs` — Only fix imports, don't change crypto logic
- `storage.rs` — Can extend, but don't break existing patterns
- `types.rs` — Can add types, don't remove existing ones

**If you need to change crypto/identity logic, escalate to coordinator.**

### Never Modify

- `.git/` (obviously)
- `../iroh-examples/` (reference only)
- `../syncengine-tauri/` (reference only)

---

## Dependency Rules

### Before Adding a Dependency

1. Check if `../iroh-examples/iroh-automerge-repo/Cargo.toml` uses it
2. Prefer workspace dependencies (defined in root `Cargo.toml`)
3. Match versions with iroh-examples to avoid conflicts

### Adding to Workspace

```toml
# Root Cargo.toml
[workspace.dependencies]
new-crate = "1.0"

# Crate Cargo.toml
[dependencies]
new-crate.workspace = true
```

---

## Common Issues

### "iroh API doesn't match examples"

Version mismatch. Check:
```bash
cat ../iroh-examples/iroh-automerge-repo/Cargo.toml | grep iroh
cat Cargo.toml | grep iroh
```

### "Test passes locally but fails in CI"

Usually timing. Add small delays for async tests:
```rust
tokio::time::sleep(Duration::from_millis(100)).await;
```

### "Automerge document is empty after sync"

You're creating separate documents. Make sure both nodes operate on the same document ID / topic.

### "Gossip messages not received"

1. Check both nodes subscribed to same TopicId
2. Verify bootstrap peers are correct
3. Add delay after subscribe before sending

---

## Commit Guidelines

### Message Format

```
<type>: <short description>

<optional body>

<optional footer>
```

Types:
- `feat:` — New feature
- `fix:` — Bug fix
- `test:` — Adding tests
- `refactor:` — Code change that doesn't add feature or fix bug
- `docs:` — Documentation only
- `chore:` — Build process, dependencies

### When to Commit

**DO NOT commit until the user has manually tested and confirmed the feature works.**

For UI changes especially, the code may compile but the feature may not work as intended. Wait for the user to:
1. Run the app
2. Test the feature visually
3. Confirm it's working

Only then should you commit. This prevents cluttering git history with broken or incomplete code.

**After user confirmation, commit when:**
- A feature is complete and tested
- Before switching to a different part of the feature
- Before any risky refactor

```bash
# Wait for user to test, then:
git add -A && git commit -m "feat: implement gossip echo"
```

---

## Escalation

**Stop and ask the coordinator when:**

1. Test fails 3+ times with different approaches
2. Need to modify `identity.rs` or `crypto.rs` logic
3. Iroh API doesn't match examples (version issue)
4. Unclear how feature should interact with other features
5. Need code from another worktree/feature branch
6. Architectural question (gossip vs direct, etc.)

**How to escalate:**
- Describe what you tried
- Include error messages
- Show relevant code
- Ask a specific question

---

## Quick Reference

### Run Tests
```bash
cargo test                           # All tests
cargo test test_name                 # Specific test
cargo test -- --nocapture           # Show println output
cargo test -p syncengine-core             # Just core crate
```

### Run CLI
```bash
cargo run -p syncengine-cli -- --help
cargo run -p syncengine-cli -- realm create "Test"
RUST_LOG=debug cargo run -p syncengine-cli -- serve
```

### Check Code Quality
```bash
cargo clippy                        # Lints
cargo fmt                           # Format
cargo doc --open                    # Generate docs
```

### See What Changed
```bash
git status
git diff
git log --oneline -10
```

---

## Feature Checklist Template

When starting a feature, copy this:

```markdown
## Feature: <Name>

### Core Functions
- [ ] Write test: `test_<feature>_basic`
- [ ] Implement: `fn <feature>() -> Result<T>`
- [ ] Write test: `test_<feature>_edge_case`
- [ ] Handle edge case
- [ ] Write test: `test_<feature>_error`
- [ ] Handle error case

### CLI Integration
- [ ] Add CLI command
- [ ] Write CLI test
- [ ] Test manually in terminal

### Documentation
- [ ] Add doc comments to public functions
- [ ] Update CHANGELOG if significant

### Cleanup
- [ ] `cargo clippy` passes
- [ ] `cargo fmt` applied
- [ ] All tests pass
- [ ] Committed with descriptive message
```

---

## Session Logging (REQUIRED)

**Every session MUST end with a detailed log entry in `LOGS.md`.**

This is non-negotiable. Future sessions (and humans) depend on understanding what was done, why, and what's left.

### Log Entry Format

```markdown
## YYYY-MM-DD HH:MM — <Brief Title>

### What Was Done
- Bullet points of completed work
- Include file paths changed (e.g., `engine.rs:2180-2191`)
- Note any tests added/modified/deleted

### Why
- Explain the reasoning behind changes
- Link to any bugs fixed or features implemented
- Note any architectural decisions made

### What's Left / Known Issues
- Unfinished work
- Known bugs introduced or discovered
- Tests that are flaky or skipped

### Key Code Changes
- Brief description of significant changes
- Before/after if behavior changed substantially
```

### When to Log

- **At the end of every session** — Even if interrupted
- **After significant milestones** — Major feature complete, bug fixed
- **Before context switches** — Switching to different feature/branch

### What Makes a Good Log

| ✅ Good | ❌ Bad |
|---------|--------|
| "Deleted `test_startup_sync_prioritizes_contacts` — test was outdated after contacts now auto-connect via gossip (see `startup_sync` lines 2180-2191)" | "Fixed test" |
| "Changed `peers_attempted` assertion from 2 to 1 because contacts are skipped in connection loop" | "Updated assertions" |
| "Bug: messages not appearing — root cause was packets stored but conversation query didn't include sent messages" | "Fixed message bug" |

### Log File Location

```
syncengine/LOGS.md
```

Append new entries at the **top** of the file (most recent first).

---

*You are a worker. Implement your assigned feature following TDD. Escalate blockers early. **Always log your work.***

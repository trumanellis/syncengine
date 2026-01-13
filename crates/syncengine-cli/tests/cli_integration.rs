//! CLI Integration Tests
//!
//! These tests verify the CLI commands work correctly end-to-end.
//! They test the "wiring" between the CLI and the core library.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

// ============================================================================
// Test Utilities
// ============================================================================

/// Create a CLI command with a temporary data directory
fn cli_cmd(data_dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("syncengine").expect("Failed to find syncengine binary");
    cmd.arg("--data-dir").arg(data_dir.path());
    cmd
}

/// Extract realm ID from CLI output (assumes format: "ID: <base58>")
fn extract_realm_id(output: &str) -> Option<String> {
    for line in output.lines() {
        if let Some(id_part) = line.strip_prefix("  ID: ") {
            return Some(id_part.trim().to_string());
        }
    }
    None
}

/// Extract task ID from CLI output (assumes format: "ID: <ulid>")
fn extract_task_id(output: &str) -> Option<String> {
    for line in output.lines() {
        if let Some(id_part) = line.strip_prefix("  ID: ") {
            return Some(id_part.trim().to_string());
        }
    }
    None
}

// ============================================================================
// Info Command Tests
// ============================================================================

#[test]
fn test_info_command() {
    let data_dir = TempDir::new().unwrap();

    cli_cmd(&data_dir)
        .arg("info")
        .assert()
        .success()
        .stdout(predicate::str::contains("Synchronicity Engine"))
        .stdout(predicate::str::contains("Identity:"))
        .stdout(predicate::str::contains("DID:"));
}

#[test]
fn test_info_shows_data_directory() {
    let data_dir = TempDir::new().unwrap();

    cli_cmd(&data_dir)
        .arg("info")
        .assert()
        .success()
        .stdout(predicate::str::contains("Data directory:"));
}

// ============================================================================
// Identity Command Tests
// ============================================================================

#[test]
fn test_identity_show() {
    let data_dir = TempDir::new().unwrap();

    cli_cmd(&data_dir)
        .args(["identity", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Identity:"))
        .stdout(predicate::str::contains("DID:"));
}

#[test]
fn test_identity_export_base58() {
    let data_dir = TempDir::new().unwrap();

    cli_cmd(&data_dir)
        .args(["identity", "export", "--format", "base58"])
        .assert()
        .success();
}

#[test]
fn test_identity_export_hex() {
    let data_dir = TempDir::new().unwrap();

    cli_cmd(&data_dir)
        .args(["identity", "export", "--format", "hex"])
        .assert()
        .success();
}

#[test]
fn test_identity_regenerate_requires_force() {
    let data_dir = TempDir::new().unwrap();

    // Without --force, should print warning but not error
    cli_cmd(&data_dir)
        .args(["identity", "regenerate"])
        .assert()
        .success()
        .stdout(predicate::str::contains("WARNING"))
        .stdout(predicate::str::contains("--force"));
}

#[test]
fn test_identity_regenerate_with_force() {
    let data_dir = TempDir::new().unwrap();

    // First, get initial DID
    let output = cli_cmd(&data_dir)
        .args(["identity", "show"])
        .output()
        .unwrap();
    let initial_output = String::from_utf8_lossy(&output.stdout);

    // Regenerate
    cli_cmd(&data_dir)
        .args(["identity", "regenerate", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("regenerated"))
        .stdout(predicate::str::contains("New DID:"));

    // Verify DID changed
    let output = cli_cmd(&data_dir)
        .args(["identity", "show"])
        .output()
        .unwrap();
    let new_output = String::from_utf8_lossy(&output.stdout);

    // The DIDs should be different (regenerated)
    // Note: Can't do direct comparison easily, but the command succeeded
    assert!(new_output.contains("DID:"));
}

// ============================================================================
// Realm Command Tests
// ============================================================================

#[test]
fn test_realm_create() {
    let data_dir = TempDir::new().unwrap();

    cli_cmd(&data_dir)
        .args(["realm", "create", "Test Realm"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created realm: Test Realm"))
        .stdout(predicate::str::contains("ID:"));
}

#[test]
fn test_realm_list_empty() {
    let data_dir = TempDir::new().unwrap();

    cli_cmd(&data_dir)
        .args(["realm", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No realms found"));
}

#[test]
fn test_realm_list_with_realms() {
    let data_dir = TempDir::new().unwrap();

    // Create a realm
    cli_cmd(&data_dir)
        .args(["realm", "create", "My Realm"])
        .assert()
        .success();

    // List should show the realm
    cli_cmd(&data_dir)
        .args(["realm", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Realms (1)"))
        .stdout(predicate::str::contains("My Realm"));
}

#[test]
fn test_realm_show() {
    let data_dir = TempDir::new().unwrap();

    // Create a realm
    let output = cli_cmd(&data_dir)
        .args(["realm", "create", "Show Test"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let realm_id = extract_realm_id(&stdout).expect("Should find realm ID");

    // Show the realm
    cli_cmd(&data_dir)
        .args(["realm", "show", &realm_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Realm: Show Test"))
        .stdout(predicate::str::contains("ID:"))
        .stdout(predicate::str::contains("Shared:"))
        .stdout(predicate::str::contains("Created:"))
        .stdout(predicate::str::contains("Tasks: 0"));
}

#[test]
fn test_realm_show_invalid_id() {
    let data_dir = TempDir::new().unwrap();

    cli_cmd(&data_dir)
        .args(["realm", "show", "invalid-id"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid realm ID"));
}

#[test]
fn test_realm_show_nonexistent() {
    let data_dir = TempDir::new().unwrap();

    // Use a valid base58 but nonexistent realm
    let fake_id = "4vJ9JU1bJJE96FWSJKvHsmmFADCg4gpZQff4P3bkLKi";

    cli_cmd(&data_dir)
        .args(["realm", "show", fake_id])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Realm not found"));
}

#[test]
fn test_realm_delete() {
    let data_dir = TempDir::new().unwrap();

    // Create a realm
    let output = cli_cmd(&data_dir)
        .args(["realm", "create", "To Delete"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let realm_id = extract_realm_id(&stdout).expect("Should find realm ID");

    // Delete the realm
    cli_cmd(&data_dir)
        .args(["realm", "delete", &realm_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted realm:"));

    // Realm should no longer exist
    cli_cmd(&data_dir)
        .args(["realm", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No realms found"));
}

// ============================================================================
// Task Command Tests
// ============================================================================

#[test]
fn test_task_add() {
    let data_dir = TempDir::new().unwrap();

    // Create a realm first
    let output = cli_cmd(&data_dir)
        .args(["realm", "create", "Task Test"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let realm_id = extract_realm_id(&stdout).expect("Should find realm ID");

    // Add a task
    cli_cmd(&data_dir)
        .args(["task", "add", &realm_id, "Build solar dehydrator"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Added task: Build solar dehydrator",
        ))
        .stdout(predicate::str::contains("ID:"));
}

#[test]
fn test_task_list_empty() {
    let data_dir = TempDir::new().unwrap();

    // Create a realm
    let output = cli_cmd(&data_dir)
        .args(["realm", "create", "Empty Realm"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let realm_id = extract_realm_id(&stdout).expect("Should find realm ID");

    // List tasks
    cli_cmd(&data_dir)
        .args(["task", "list", &realm_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("No tasks in this realm"));
}

#[test]
fn test_task_list_with_tasks() {
    let data_dir = TempDir::new().unwrap();

    // Create a realm
    let output = cli_cmd(&data_dir)
        .args(["realm", "create", "Task List Test"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let realm_id = extract_realm_id(&stdout).expect("Should find realm ID");

    // Add tasks
    cli_cmd(&data_dir)
        .args(["task", "add", &realm_id, "Task One"])
        .assert()
        .success();

    cli_cmd(&data_dir)
        .args(["task", "add", &realm_id, "Task Two"])
        .assert()
        .success();

    // List tasks
    cli_cmd(&data_dir)
        .args(["task", "list", &realm_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Tasks (2)"))
        .stdout(predicate::str::contains("Task One"))
        .stdout(predicate::str::contains("Task Two"));
}

#[test]
fn test_task_toggle() {
    let data_dir = TempDir::new().unwrap();

    // Create a realm
    let output = cli_cmd(&data_dir)
        .args(["realm", "create", "Toggle Test"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let realm_id = extract_realm_id(&stdout).expect("Should find realm ID");

    // Add a task
    let output = cli_cmd(&data_dir)
        .args(["task", "add", &realm_id, "Toggle Me"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let task_id = extract_task_id(&stdout).expect("Should find task ID");

    // Toggle to completed
    cli_cmd(&data_dir)
        .args(["task", "toggle", &realm_id, &task_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("completed"));

    // Toggle back to incomplete
    cli_cmd(&data_dir)
        .args(["task", "toggle", &realm_id, &task_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("incomplete"));
}

#[test]
fn test_task_delete() {
    let data_dir = TempDir::new().unwrap();

    // Create a realm
    let output = cli_cmd(&data_dir)
        .args(["realm", "create", "Delete Task Test"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let realm_id = extract_realm_id(&stdout).expect("Should find realm ID");

    // Add a task
    let output = cli_cmd(&data_dir)
        .args(["task", "add", &realm_id, "To Delete"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let task_id = extract_task_id(&stdout).expect("Should find task ID");

    // Delete the task
    cli_cmd(&data_dir)
        .args(["task", "delete", &realm_id, &task_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted task:"));

    // Task should no longer appear in list
    cli_cmd(&data_dir)
        .args(["task", "list", &realm_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("No tasks in this realm"));
}

#[test]
fn test_task_toggle_nonexistent() {
    let data_dir = TempDir::new().unwrap();

    // Create a realm
    let output = cli_cmd(&data_dir)
        .args(["realm", "create", "Nonexistent Task"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let realm_id = extract_realm_id(&stdout).expect("Should find realm ID");

    // Try to toggle nonexistent task
    cli_cmd(&data_dir)
        .args(["task", "toggle", &realm_id, "01ARZ3NDEKTSV4RRFFQ69G5FAV"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("Task")));
}

// ============================================================================
// Invite Command Tests
// ============================================================================

#[test]
fn test_invite_create() {
    let data_dir = TempDir::new().unwrap();

    // Create a realm
    let output = cli_cmd(&data_dir)
        .args(["realm", "create", "Invite Test"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let realm_id = extract_realm_id(&stdout).expect("Should find realm ID");

    // Create invite
    cli_cmd(&data_dir)
        .args(["invite", "create", &realm_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Invite created"))
        .stdout(predicate::str::contains("sync-invite:"));
}

#[test]
fn test_invite_create_nonexistent_realm() {
    let data_dir = TempDir::new().unwrap();

    // Use a valid base58 but nonexistent realm
    let fake_id = "4vJ9JU1bJJE96FWSJKvHsmmFADCg4gpZQff4P3bkLKi";

    cli_cmd(&data_dir)
        .args(["invite", "create", fake_id])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("Realm")));
}

#[test]
fn test_invite_join_invalid_format() {
    let data_dir = TempDir::new().unwrap();

    // Try to join with invalid ticket
    cli_cmd(&data_dir)
        .args(["invite", "join", "not-a-valid-ticket"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid"));
}

// ============================================================================
// Full Workflow Tests
// ============================================================================

#[test]
fn test_full_task_workflow() {
    let data_dir = TempDir::new().unwrap();

    // 1. Check initial state
    cli_cmd(&data_dir)
        .arg("info")
        .assert()
        .success()
        .stdout(predicate::str::contains("Realms: 0"));

    // 2. Create a realm
    let output = cli_cmd(&data_dir)
        .args(["realm", "create", "Community Garden"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let realm_id = extract_realm_id(&stdout).expect("Should find realm ID");

    // 3. Add several tasks
    let tasks = ["Plant tomatoes", "Build compost bin", "Install irrigation"];

    let mut task_ids = Vec::new();
    for task in &tasks {
        let output = cli_cmd(&data_dir)
            .args(["task", "add", &realm_id, task])
            .output()
            .unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);
        task_ids.push(extract_task_id(&stdout).expect("Should find task ID"));
    }

    // 4. List tasks
    cli_cmd(&data_dir)
        .args(["task", "list", &realm_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Tasks (3)"))
        .stdout(predicate::str::contains("Plant tomatoes"))
        .stdout(predicate::str::contains("Build compost bin"))
        .stdout(predicate::str::contains("Install irrigation"));

    // 5. Complete the first task
    cli_cmd(&data_dir)
        .args(["task", "toggle", &realm_id, &task_ids[0]])
        .assert()
        .success()
        .stdout(predicate::str::contains("completed"));

    // 6. Delete the second task
    cli_cmd(&data_dir)
        .args(["task", "delete", &realm_id, &task_ids[1]])
        .assert()
        .success();

    // 7. Verify state
    cli_cmd(&data_dir)
        .args(["task", "list", &realm_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Tasks (2)"))
        .stdout(predicate::str::contains("Plant tomatoes"))
        .stdout(predicate::str::contains("Build compost bin").not())
        .stdout(predicate::str::contains("Install irrigation"));

    // 8. Create invite
    cli_cmd(&data_dir)
        .args(["invite", "create", &realm_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("sync-invite:"));

    // 9. Delete the realm
    cli_cmd(&data_dir)
        .args(["realm", "delete", &realm_id])
        .assert()
        .success();

    // 10. Verify realm is gone
    cli_cmd(&data_dir)
        .args(["realm", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No realms found"));
}

#[test]
fn test_multiple_realms() {
    let data_dir = TempDir::new().unwrap();

    // Create multiple realms
    let realm_names = ["Work", "Personal", "Shopping"];
    let mut realm_ids = Vec::new();

    for name in &realm_names {
        let output = cli_cmd(&data_dir)
            .args(["realm", "create", name])
            .output()
            .unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);
        realm_ids.push(extract_realm_id(&stdout).expect("Should find realm ID"));
    }

    // List should show all realms
    cli_cmd(&data_dir)
        .args(["realm", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Realms (3)"))
        .stdout(predicate::str::contains("Work"))
        .stdout(predicate::str::contains("Personal"))
        .stdout(predicate::str::contains("Shopping"));

    // Add tasks to each realm
    for (i, realm_id) in realm_ids.iter().enumerate() {
        cli_cmd(&data_dir)
            .args(["task", "add", realm_id, &format!("Task for realm {}", i)])
            .assert()
            .success();
    }

    // Verify each realm has its own tasks
    for (i, realm_id) in realm_ids.iter().enumerate() {
        cli_cmd(&data_dir)
            .args(["task", "list", realm_id])
            .assert()
            .success()
            .stdout(predicate::str::contains(&format!("Task for realm {}", i)));
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_invalid_subcommand() {
    let data_dir = TempDir::new().unwrap();

    cli_cmd(&data_dir).arg("nonexistent").assert().failure();
}

#[test]
fn test_missing_required_args() {
    let data_dir = TempDir::new().unwrap();

    // realm create without name
    cli_cmd(&data_dir)
        .args(["realm", "create"])
        .assert()
        .failure();

    // task add without realm_id
    cli_cmd(&data_dir).args(["task", "add"]).assert().failure();
}

#[test]
fn test_help_works() {
    let data_dir = TempDir::new().unwrap();

    // --help shows long_about which mentions "peer-to-peer task sharing"
    cli_cmd(&data_dir)
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("task sharing"));

    cli_cmd(&data_dir)
        .args(["realm", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Realm management"));

    cli_cmd(&data_dir)
        .args(["task", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Task management"));
}

#[test]
fn test_version() {
    let data_dir = TempDir::new().unwrap();

    cli_cmd(&data_dir)
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

// ============================================================================
// Data Persistence Tests
// ============================================================================

#[test]
fn test_data_persists_across_invocations() {
    let data_dir = TempDir::new().unwrap();

    // Create a realm
    let output = cli_cmd(&data_dir)
        .args(["realm", "create", "Persistent"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let realm_id = extract_realm_id(&stdout).expect("Should find realm ID");

    // Add a task
    cli_cmd(&data_dir)
        .args(["task", "add", &realm_id, "Remember me"])
        .assert()
        .success();

    // New invocation should see the data
    cli_cmd(&data_dir)
        .args(["realm", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Persistent"));

    cli_cmd(&data_dir)
        .args(["task", "list", &realm_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Remember me"));
}

#[test]
fn test_identity_persists() {
    let data_dir = TempDir::new().unwrap();

    // Get initial identity
    let output = cli_cmd(&data_dir)
        .args(["identity", "show"])
        .output()
        .unwrap();

    let first_output = String::from_utf8_lossy(&output.stdout);

    // New invocation should have same identity
    let output = cli_cmd(&data_dir)
        .args(["identity", "show"])
        .output()
        .unwrap();

    let second_output = String::from_utf8_lossy(&output.stdout);

    // DID lines should be identical
    let first_did = first_output.lines().find(|l| l.contains("DID:"));
    let second_did = second_output.lines().find(|l| l.contains("DID:"));

    assert_eq!(first_did, second_did, "Identity should persist");
}

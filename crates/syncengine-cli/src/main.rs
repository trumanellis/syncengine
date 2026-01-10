//! Synchronicity Engine CLI
//!
//! Thin wrapper around syncengine-core functions for command-line usage.
//!
//! ## Usage
//!
//! ```bash
//! # Show node information
//! syncengine info
//!
//! # Create a new realm
//! syncengine realm create "My Tasks"
//!
//! # List all realms
//! syncengine realm list
//!
//! # Add a task to a realm
//! syncengine task add <realm_id> "Buy groceries"
//!
//! # List tasks in a realm
//! syncengine task list <realm_id>
//!
//! # Toggle task completion
//! syncengine task toggle <realm_id> <task_id>
//!
//! # Create an invite for a realm
//! syncengine invite create <realm_id>
//!
//! # Join a realm via invite
//! syncengine invite join <ticket>
//! ```

use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use clap::{Parser, Subcommand};
use syncengine_core::{RealmId, SyncEngine, TaskId};

/// Synchronicity Engine - P2P Task Sharing
#[derive(Parser)]
#[command(name = "syncengine")]
#[command(version = "0.1.0")]
#[command(about = "Synchronicity Engine - P2P Task Sharing")]
#[command(long_about = "A censorship-resistant, local-first, peer-to-peer task sharing application implementing a sacred gifting economy.")]
struct Cli {
    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Data directory (default: ~/.syncengine/data)
    #[arg(short, long, global = true)]
    data_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show node information
    Info,

    /// Identity management
    Identity {
        #[command(subcommand)]
        action: IdentityAction,
    },

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

    /// Start serving/syncing as a persistent P2P node
    Serve {
        /// Realm to sync (optional, can join/create realms via other commands)
        #[arg(short, long)]
        realm: Option<String>,
    },
}

#[derive(Subcommand)]
enum IdentityAction {
    /// Show identity info (DID and public key fingerprint)
    Show,
    /// Export public key (for backup/sharing)
    Export {
        /// Output format: base58, hex, or json
        #[arg(short, long, default_value = "base58")]
        format: String,
    },
    /// Generate new identity (WARNING: replaces existing)
    Regenerate {
        /// Confirm regeneration (required)
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
enum RealmAction {
    /// Create a new realm
    Create {
        /// Name of the realm
        name: String,
    },
    /// List all realms
    List,
    /// Show realm details
    Show {
        /// Realm ID (base58)
        realm_id: String,
    },
    /// Delete a realm
    Delete {
        /// Realm ID (base58)
        realm_id: String,
    },
}

#[derive(Subcommand)]
enum TaskAction {
    /// Add a task to a realm
    Add {
        /// Realm ID (base58)
        realm_id: String,
        /// Task title
        title: String,
    },
    /// List tasks in a realm
    List {
        /// Realm ID (base58)
        realm_id: String,
    },
    /// Toggle task completion
    Toggle {
        /// Realm ID (base58)
        realm_id: String,
        /// Task ID (ULID string)
        task_id: String,
    },
    /// Delete a task
    Delete {
        /// Realm ID (base58)
        realm_id: String,
        /// Task ID (ULID string)
        task_id: String,
    },
}

#[derive(Subcommand)]
enum InviteAction {
    /// Create an invite for a realm
    Create {
        /// Realm ID (base58)
        realm_id: String,
    },
    /// Join a realm via invite ticket
    Join {
        /// Invite ticket (sync-invite:...)
        ticket: String,
    },
}

fn setup_logging(verbosity: u8) {
    let filter = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(filter)),
        )
        .init();
}

/// Get the default data directory (~/.syncengine/data)
fn default_data_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".syncengine")
        .join("data")
}

/// Parse a realm ID from base58 string
fn parse_realm_id(s: &str) -> Result<RealmId> {
    RealmId::from_base58(s).map_err(|e| anyhow::anyhow!("Invalid realm ID '{}': {}", s, e))
}

/// Parse a task ID from ULID string
fn parse_task_id(s: &str) -> Result<TaskId> {
    TaskId::from_string(s).map_err(|e| anyhow::anyhow!("Invalid task ID '{}': {}", s, e))
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    setup_logging(cli.verbose);

    let data_dir = cli.data_dir.unwrap_or_else(default_data_dir);
    let mut engine = SyncEngine::new(&data_dir).await?;

    // Initialize identity on startup so DID is always available
    engine.init_identity()?;

    match cli.command {
        Commands::Info => {
            let info = engine.node_info().await?;

            println!("Synchronicity Engine v0.1.0");
            println!();
            println!("Identity:");
            if let Some(did) = info.did {
                println!("  DID: {}", did);
            } else {
                println!("  DID: (not initialized)");
            }
            println!();
            println!("Node:");
            if let Some(node_id) = info.node_id {
                println!("  ID: {}", node_id);
            } else {
                println!("  ID: (P2P not active)");
            }
            if let Some(relay) = info.relay_url {
                println!("  Relay: {}", relay);
            }
            println!();
            println!("Data directory: {}", info.data_dir.display());
            println!("Realms: {}", info.realm_count);
            println!();
            println!("Status: Local mode (P2P not active)");
        }

        Commands::Identity { action } => match action {
            IdentityAction::Show => {
                if let Some(did) = engine.did() {
                    println!("Identity:");
                    println!("  DID: {}", did);
                    if let Some(pk) = engine.public_key() {
                        let bytes = pk.to_bytes();
                        // Show first 8 bytes of Ed25519 key as fingerprint
                        let fingerprint = hex::encode(&bytes[..8]);
                        println!("  Ed25519 fingerprint: {}", fingerprint);
                        println!("  Public key size: {} bytes", bytes.len());
                    }
                } else {
                    println!("Identity not initialized.");
                }
            }

            IdentityAction::Export { format } => {
                if let Some(exported) = engine.export_public_key(&format) {
                    println!("{}", exported);
                } else {
                    anyhow::bail!("Identity not initialized");
                }
            }

            IdentityAction::Regenerate { force } => {
                if !force {
                    println!("WARNING: Regenerating identity is IRREVERSIBLE!");
                    println!();
                    println!("This will:");
                    println!("  - Generate a new keypair");
                    println!("  - Replace your existing identity");
                    println!("  - Invalidate any data signed with the old identity");
                    println!();
                    println!("To confirm, run: syncengine identity regenerate --force");
                } else {
                    engine.regenerate_identity()?;
                    let did = engine.did().expect("DID should exist after regeneration");
                    println!("Identity regenerated.");
                    println!("  New DID: {}", did);
                }
            }
        }

        Commands::Realm { action } => match action {
            RealmAction::Create { name } => {
                let id = engine.create_realm(&name).await?;
                println!("Created realm: {}", name);
                println!("  ID: {}", id.to_base58());
            }

            RealmAction::List => {
                let realms = engine.list_realms().await?;
                if realms.is_empty() {
                    println!("No realms found.");
                } else {
                    println!("Realms ({}):", realms.len());
                    println!();
                    for realm in realms {
                        let shared = if realm.is_shared { " [shared]" } else { "" };
                        println!("  {} {}{}", realm.id.to_base58(), realm.name, shared);
                    }
                }
            }

            RealmAction::Show { realm_id } => {
                let id = parse_realm_id(&realm_id)?;
                match engine.get_realm(&id).await? {
                    Some(realm) => {
                        println!("Realm: {}", realm.name);
                        println!("  ID: {}", realm.id.to_base58());
                        println!("  Shared: {}", if realm.is_shared { "Yes" } else { "No" });
                        println!(
                            "  Created: {}",
                            chrono::DateTime::from_timestamp(realm.created_at, 0)
                                .map(|dt| dt.to_rfc3339())
                                .unwrap_or_else(|| realm.created_at.to_string())
                        );

                        // Open realm to list tasks
                        engine.open_realm(&id).await?;
                        let tasks = engine.list_tasks(&id)?;
                        println!("  Tasks: {}", tasks.len());
                    }
                    None => {
                        anyhow::bail!("Realm not found: {}", realm_id);
                    }
                }
            }

            RealmAction::Delete { realm_id } => {
                let id = parse_realm_id(&realm_id)?;
                engine.delete_realm(&id).await?;
                println!("Deleted realm: {}", realm_id);
            }
        },

        Commands::Task { action } => match action {
            TaskAction::Add { realm_id, title } => {
                let id = parse_realm_id(&realm_id)?;
                let task_id = engine.add_task(&id, &title).await?;
                println!("Added task: {}", title);
                println!("  ID: {}", task_id.to_string_repr());
            }

            TaskAction::List { realm_id } => {
                let id = parse_realm_id(&realm_id)?;
                engine.open_realm(&id).await?;
                let tasks = engine.list_tasks(&id)?;

                if tasks.is_empty() {
                    println!("No tasks in this realm.");
                } else {
                    println!("Tasks ({}):", tasks.len());
                    println!();
                    for task in tasks {
                        let status = if task.completed { "✓" } else { "○" };
                        println!("  {} {} {}", status, task.id.to_string_repr(), task.title);
                    }
                }
            }

            TaskAction::Toggle { realm_id, task_id } => {
                let rid = parse_realm_id(&realm_id)?;
                let tid = parse_task_id(&task_id)?;
                engine.toggle_task(&rid, &tid).await?;

                // Show new state
                if let Some(task) = engine.get_task(&rid, &tid)? {
                    let status = if task.completed {
                        "completed"
                    } else {
                        "incomplete"
                    };
                    println!("Toggled task: {} -> {}", task.title, status);
                }
            }

            TaskAction::Delete { realm_id, task_id } => {
                let rid = parse_realm_id(&realm_id)?;
                let tid = parse_task_id(&task_id)?;
                engine.delete_task(&rid, &tid).await?;
                println!("Deleted task: {}", task_id);
            }
        },

        Commands::Invite { action } => match action {
            InviteAction::Create { realm_id } => {
                let id = parse_realm_id(&realm_id)?;
                let ticket = engine.create_invite(&id).await?;
                println!("Invite created:");
                println!();
                println!("{}", ticket);
                println!();
                println!("Share this link to invite others to your realm.");
            }

            InviteAction::Join { ticket } => {
                let realm_id = engine.join_realm(&ticket).await?;
                if let Some(realm) = engine.get_realm(&realm_id).await? {
                    println!("Joined realm: {}", realm.name);
                    println!("  ID: {}", realm_id.to_base58());
                } else {
                    println!("Joined realm: {}", realm_id.to_base58());
                }
            }
        },

        Commands::Serve { realm } => {
            println!("Starting Synchronicity Engine...");
            println!();

            // Display identity
            let did = engine.did().unwrap();
            println!("Identity:");
            println!("  DID: {}", did);
            println!();

            // Start gossip networking
            engine.start_networking().await?;
            let info = engine.node_info().await?;

            println!("Node:");
            if let Some(node_id) = &info.node_id {
                println!("  ID: {}", node_id);
            }
            if let Some(relay) = &info.relay_url {
                println!("  Relay: {}", relay);
            }
            println!();

            // If realm specified, start syncing that realm
            if let Some(realm_id_str) = &realm {
                let realm_id = parse_realm_id(realm_id_str)?;
                engine.open_realm(&realm_id).await?;
                engine.start_sync(&realm_id).await?;

                // Get realm name for display
                if let Some(realm_info) = engine.get_realm(&realm_id).await? {
                    println!("Syncing realm: {} ({})", realm_info.name, realm_id_str);
                } else {
                    println!("Syncing realm: {}", realm_id_str);
                }
                println!();
            }

            println!("Data directory: {}", info.data_dir.display());
            println!();
            println!("Node is running. Press Ctrl+C to stop.");
            println!();

            // Run event loop with periodic status updates
            let status_interval = Duration::from_secs(60);
            let mut last_status = std::time::Instant::now();

            loop {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {
                        println!();
                        println!("Received shutdown signal...");
                        break;
                    }
                    _ = tokio::time::sleep(Duration::from_secs(1)) => {
                        // Check if we should print status
                        if last_status.elapsed() >= status_interval {
                            last_status = std::time::Instant::now();

                            // Print status update
                            let realms = engine.list_realms().await?;
                            let syncing_count = realms.iter()
                                .filter(|r| engine.is_realm_syncing(&r.id))
                                .count();

                            println!(
                                "[Status] {} realm(s), {} syncing",
                                realms.len(),
                                syncing_count
                            );
                        }
                    }
                }
            }

            println!("Shutting down...");
            engine.shutdown().await?;
            println!("Goodbye.");
        }
    }

    Ok(())
}

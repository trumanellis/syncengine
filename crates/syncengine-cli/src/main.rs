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
//!
//! # Generate a contact invitation
//! syncengine contact generate-invite
//!
//! # Accept a contact invitation
//! syncengine contact accept <invite_code>
//!
//! # List all contacts
//! syncengine contact list
//!
//! # List pending contact requests
//! syncengine contact pending
//! ```

use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use anyhow::Result;
use tokio::io::AsyncBufReadExt;
use clap::{Parser, Subcommand};
use syncengine_core::{Did, PeerStatus, RealmId, SyncEngine, TaskId};

/// Synchronicity Engine - P2P Task Sharing
#[derive(Parser)]
#[command(name = "syncengine")]
#[command(version = "0.1.0")]
#[command(about = "Synchronicity Engine - P2P Task Sharing")]
#[command(
    long_about = "A censorship-resistant, local-first, peer-to-peer task sharing application implementing a sacred gifting economy."
)]
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

    /// Peer management
    Peers {
        #[command(subcommand)]
        action: PeersAction,
    },

    /// Contact exchange management
    Contact {
        #[command(subcommand)]
        cmd: ContactCommands,
    },

    /// Profile management
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },

    /// Packet layer commands (Indra's Network)
    Packet {
        #[command(subcommand)]
        action: PacketAction,
    },

    /// Chat commands for direct messaging
    Chat {
        #[command(subcommand)]
        action: ChatAction,
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

#[derive(Subcommand)]
enum PeersAction {
    /// List all discovered peers
    List {
        /// Filter by status (online, offline, unknown)
        #[arg(short, long)]
        status: Option<String>,
    },
    /// Show status of specific peer
    Status {
        /// Peer endpoint ID (hex format)
        endpoint_id: String,
    },
    /// Manually attempt to connect to a peer
    Connect {
        /// Peer endpoint ID (hex format)
        endpoint_id: String,
    },
    /// Set local nickname for a peer
    SetNickname {
        /// Peer endpoint ID (hex format)
        endpoint_id: String,
        /// Nickname to set
        nickname: String,
    },
}

#[derive(Subcommand)]
enum ContactCommands {
    /// Generate a contact invitation code
    GenerateInvite {
        /// Hours until invite expires (default: 24)
        #[arg(short, long, default_value = "24")]
        expiry_hours: u8,
    },

    /// Accept a contact invitation
    Accept {
        /// The invitation code or invite ID to accept
        invite_code: String,
    },

    /// List all contacts
    List,

    /// List pending contact requests
    Pending,
}

#[derive(Subcommand)]
enum ProfileAction {
    /// Show your profile
    Show,

    /// Set profile fields
    Set {
        /// Display name
        #[arg(short, long)]
        name: Option<String>,

        /// Subtitle/tagline
        #[arg(short, long)]
        tagline: Option<String>,

        /// Bio (markdown)
        #[arg(short, long)]
        bio: Option<String>,

        /// Avatar image file path
        #[arg(short, long)]
        avatar: Option<PathBuf>,
    },

    /// Get a peer's profile by DID
    Get {
        /// DID of the peer
        did: String,
    },

    /// Profile pinning commands
    Pins {
        #[command(subcommand)]
        action: ProfilePinAction,
    },
}

#[derive(Subcommand)]
enum ProfilePinAction {
    /// List all pinned profiles
    List,

    /// Manually pin a profile by DID
    Pin {
        /// DID of the profile to pin
        did: String,
    },

    /// Unpin a profile
    Unpin {
        /// DID of the profile to unpin
        did: String,
    },

    /// Sign and broadcast our own profile
    Announce,
}

/// Packet layer commands for Indra's Network
#[derive(Subcommand)]
enum PacketAction {
    /// Show own packet log
    Log,

    /// Send a test packet (heartbeat)
    SendHeartbeat,

    /// Show mirror of another profile
    Mirror {
        /// DID of the profile to show
        did: String,
    },

    /// List all mirrored profiles
    Mirrors,

    /// Show profile keys info
    Keys,
}

/// Chat commands for direct messaging
#[derive(Subcommand)]
enum ChatAction {
    /// List all conversations
    List,

    /// Show conversation with a contact
    Show {
        /// Contact's DID
        did: String,

        /// Number of messages to show (default: all)
        #[arg(short, long)]
        limit: Option<usize>,
    },

    /// Send a message to a contact
    Send {
        /// Contact's DID
        did: String,

        /// Message content
        message: String,
    },

    /// Interactive chat mode with a contact
    Interactive {
        /// Contact's DID
        did: String,
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

/// Parse a peer endpoint ID from hex string
fn parse_endpoint_id(s: &str) -> Result<iroh::PublicKey> {
    let bytes = hex::decode(s).map_err(|e| anyhow::anyhow!("Invalid hex format: {}", e))?;
    if bytes.len() != 32 {
        anyhow::bail!("Endpoint ID must be 32 bytes (got {})", bytes.len());
    }
    let mut array = [0u8; 32];
    array.copy_from_slice(&bytes);
    iroh::PublicKey::from_bytes(&array).map_err(|e| anyhow::anyhow!("Invalid public key: {}", e))
}

/// Parse peer status from string
fn parse_peer_status(s: &str) -> Result<PeerStatus> {
    match s.to_lowercase().as_str() {
        "online" => Ok(PeerStatus::Online),
        "offline" => Ok(PeerStatus::Offline),
        "unknown" => Ok(PeerStatus::Unknown),
        _ => anyhow::bail!(
            "Invalid status '{}'. Must be one of: online, offline, unknown",
            s
        ),
    }
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
        },

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

        Commands::Peers { action } => match action {
            PeersAction::List { status } => {
                let peers = if let Some(status_str) = status {
                    let status = parse_peer_status(&status_str)?;
                    engine.peer_registry().list_by_status(status)?
                } else {
                    engine.peer_registry().list_all()?
                };

                if peers.is_empty() {
                    println!("No peers found.");
                } else {
                    println!("Discovered peers ({}):", peers.len());
                    println!();
                    for peer in peers {
                        let endpoint_id_hex = hex::encode(peer.endpoint_id);
                        let nickname = peer.nickname.unwrap_or_else(|| "(unnamed)".to_string());
                        let status = peer.status;
                        let realms = peer.shared_realms.len();

                        println!(
                            "  {} - {} [{}] ({} shared realms)",
                            &endpoint_id_hex[..16],
                            nickname,
                            status,
                            realms
                        );
                        println!("    Full ID: {}", endpoint_id_hex);
                        if !peer.shared_realms.is_empty() {
                            println!("    Shared realms:");
                            for realm_id in &peer.shared_realms {
                                println!("      - {}", realm_id.to_base58());
                            }
                        }
                        println!();
                    }
                }
            }

            PeersAction::Status { endpoint_id } => {
                let peer_id = parse_endpoint_id(&endpoint_id)?;
                match engine.peer_registry().get(&peer_id)? {
                    Some(peer) => {
                        let nickname = peer.nickname.unwrap_or_else(|| "(unnamed)".to_string());
                        println!("Peer: {}", nickname);
                        println!("  Endpoint ID: {}", hex::encode(peer.endpoint_id));
                        println!("  Status: {}", peer.status);
                        println!("  Last seen: {} (Unix timestamp)", peer.last_seen);
                        println!("  Shared realms: {}", peer.shared_realms.len());
                        if !peer.shared_realms.is_empty() {
                            for realm_id in &peer.shared_realms {
                                println!("    - {}", realm_id.to_base58());
                            }
                        }
                    }
                    None => {
                        anyhow::bail!("Peer not found: {}", endpoint_id);
                    }
                }
            }

            PeersAction::Connect { endpoint_id } => {
                let peer_id = parse_endpoint_id(&endpoint_id)?;

                // Ensure networking is started
                if !engine.is_networking_active() {
                    engine.start_networking().await?;
                }

                println!("Attempting to connect to peer {}...", &endpoint_id[..16]);

                // Manually trigger reconnection for this specific peer
                // We'll do this by temporarily getting the peer, updating to Unknown,
                // and then calling the reconnection logic
                if engine.peer_registry().get(&peer_id)?.is_some() {
                    engine.attempt_reconnect_inactive_peers().await?;

                    // Check the result
                    if let Some(peer) = engine.peer_registry().get(&peer_id)? {
                        match peer.status {
                            PeerStatus::Online => println!("Successfully connected to peer."),
                            PeerStatus::Offline => println!("Failed to connect to peer."),
                            PeerStatus::Unknown => println!("Connection status unknown."),
                        }
                    }
                } else {
                    anyhow::bail!("Peer not found in registry: {}", endpoint_id);
                }
            }

            PeersAction::SetNickname {
                endpoint_id,
                nickname,
            } => {
                let peer_id = parse_endpoint_id(&endpoint_id)?;
                engine
                    .peer_registry()
                    .update_nickname(&peer_id, &nickname)?;
                println!(
                    "Set nickname '{}' for peer {}",
                    nickname,
                    &endpoint_id[..16]
                );
            }
        },

        Commands::Contact { cmd } => match cmd {
            ContactCommands::GenerateInvite { expiry_hours } => {
                let invite = engine.generate_contact_invite(expiry_hours).await?;
                println!("Contact invitation generated:");
                println!();
                println!("{}", invite);
                println!();
                println!("Share this link to invite others to your contact list.");
                println!();
                println!("Expires in: {} hours", expiry_hours);
            }

            ContactCommands::Accept { invite_code } => {
                // Try to decode the invite code as hex (invite_id)
                let invite_id = match hex::decode(&invite_code) {
                    Ok(bytes) if bytes.len() == 16 => {
                        let mut id = [0u8; 16];
                        id.copy_from_slice(&bytes);
                        id
                    }
                    _ => {
                        anyhow::bail!("Invalid invite code format. Expected 32-character hex string (16 bytes).");
                    }
                };

                engine.accept_contact(&invite_id).await?;
                println!("Successfully accepted contact invitation!");
                println!();
                println!("You are now connected with this peer. Messages can now be exchanged.");
            }

            ContactCommands::List => {
                let contacts = engine.list_contacts()?;

                if contacts.is_empty() {
                    println!("No contacts in your list.");
                } else {
                    println!("Contacts ({}):", contacts.len());
                    println!();
                    for contact in contacts {
                        let status = contact.status;
                        let favorite = if contact.is_favorite { " ★" } else { "" };
                        println!("  {} {}{}", contact.profile.display_name, status, favorite);
                        println!("    DID: {}", contact.peer_did);
                        println!("    Connected: {}", contact.accepted_at);
                        println!("    Last seen: {} (Unix timestamp)", contact.last_seen);
                        if let Some(subtitle) = &contact.profile.subtitle {
                            println!("    {} ", subtitle);
                        }
                        println!();
                    }
                }
            }

            ContactCommands::Pending => {
                let (incoming, outgoing) = engine.list_pending_contacts()?;

                if incoming.is_empty() && outgoing.is_empty() {
                    println!("No pending contact requests.");
                } else {
                    if !incoming.is_empty() {
                        println!("Incoming requests ({}):", incoming.len());
                        println!();
                        for pending in &incoming {
                            println!("  {} ({})", pending.profile.display_name, pending.state);
                            println!("    DID: {}", pending.peer_did);
                            println!("    Requested: {} (Unix timestamp)", pending.created_at);
                            if let Some(subtitle) = &pending.profile.subtitle {
                                println!("    {} ", subtitle);
                            }
                            if !pending.profile.bio.is_empty() {
                                println!("    \"{}\"", pending.profile.bio);
                            }
                            println!();
                        }
                    }

                    if !outgoing.is_empty() {
                        println!("Outgoing requests ({}):", outgoing.len());
                        println!();
                        for pending in &outgoing {
                            println!("  {} ({})", pending.profile.display_name, pending.state);
                            println!("    DID: {}", pending.peer_did);
                            println!("    Sent: {} (Unix timestamp)", pending.created_at);
                            if let Some(subtitle) = &pending.profile.subtitle {
                                println!("    {} ", subtitle);
                            }
                            println!();
                        }
                    }
                }
            }
        },

        Commands::Profile { action } => match action {
            ProfileAction::Show => {
                engine.init_identity()?;
                let profile = engine.get_own_profile()?;

                println!("Your Profile:");
                println!("  Peer ID: {}", profile.peer_id);
                println!("  Display Name: {}", profile.display_name);
                if let Some(subtitle) = &profile.subtitle {
                    println!("  Tagline: {}", subtitle);
                }
                if let Some(avatar) = &profile.avatar_blob_id {
                    println!("  Avatar: {}", avatar);
                }
                println!("  Bio: {}", if profile.bio.is_empty() { "(empty)" } else { &profile.bio });
                println!("  Updated: {}", profile.updated_at);

                // Show if we have a signed/pinned version
                if let Some(pin) = engine.get_own_pinned_profile()? {
                    println!();
                    println!("  ✓ Profile is signed and pinned");
                    println!("  DID: {}", pin.did);
                }
            }

            ProfileAction::Set {
                name,
                tagline,
                bio,
                avatar,
            } => {
                engine.init_identity()?;
                let mut profile = engine.get_own_profile()?;

                let mut changed = false;

                if let Some(n) = name {
                    profile.display_name = n;
                    changed = true;
                }
                if let Some(t) = tagline {
                    profile.subtitle = Some(t);
                    changed = true;
                }
                if let Some(b) = bio {
                    profile.bio = b;
                    changed = true;
                }
                if let Some(avatar_path) = avatar {
                    // Read and upload avatar
                    let avatar_data = std::fs::read(&avatar_path)
                        .map_err(|e| anyhow::anyhow!("Failed to read avatar file: {}", e))?;

                    let blob_id = engine.upload_avatar(avatar_data).await
                        .map_err(|e| anyhow::anyhow!("Failed to upload avatar: {}", e))?;

                    profile.avatar_blob_id = Some(blob_id.clone());
                    changed = true;
                    println!("Avatar uploaded: {}", blob_id);
                }

                if changed {
                    profile.updated_at = chrono::Utc::now().timestamp();
                    engine.save_profile(&profile)?;
                    println!("Profile updated!");

                    // Also sign and pin the updated profile
                    engine.sign_and_pin_own_profile()?;
                    println!("Profile signed and pinned.");
                } else {
                    println!("No changes made.");
                }
            }

            ProfileAction::Get { did } => {
                match engine.get_pinned_profile(&did)? {
                    Some(pin) => {
                        let p = &pin.signed_profile.profile;
                        println!("Profile for {}:", did);
                        println!("  Display Name: {}", p.display_name);
                        if let Some(subtitle) = &p.subtitle {
                            println!("  Tagline: {}", subtitle);
                        }
                        if let Some(avatar) = &p.avatar_blob_id {
                            println!("  Avatar: {}", avatar);
                        }
                        println!("  Bio: {}", if p.bio.is_empty() { "(empty)" } else { &p.bio });
                        println!();
                        println!("  Relationship: {:?}", pin.relationship);
                        println!("  Pinned At: {}", pin.pinned_at);
                        println!("  ✓ Signature Valid: {}", pin.signed_profile.verify());
                    }
                    None => {
                        println!("No pinned profile found for DID: {}", did);
                        println!("(Profiles are pinned automatically from contacts)");
                    }
                }
            }

            ProfileAction::Pins { action } => match action {
                ProfilePinAction::List => {
                    let pins = engine.list_pinned_profiles()?;

                    if pins.is_empty() {
                        println!("No pinned profiles.");
                    } else {
                        println!("Pinned Profiles ({}):", pins.len());
                        println!();
                        for pin in pins {
                            let is_own = if pin.is_own() { " (own)" } else { "" };
                            println!(
                                "  {} - {}{}",
                                &pin.did[..20.min(pin.did.len())],
                                pin.signed_profile.profile.display_name,
                                is_own
                            );
                            println!("    Relationship: {:?}", pin.relationship);
                            println!("    Pinned: {} (Unix timestamp)", pin.pinned_at);
                            if let Some(hash) = &pin.avatar_hash {
                                println!("    Avatar Hash: {}", hex::encode(&hash[..8]));
                            }
                            println!();
                        }
                    }
                }

                ProfilePinAction::Pin { did } => {
                    // Manual pinning requires we already have a signed profile
                    // This would typically come from receiving an announcement
                    // For manual pinning, we'd need to request the profile from the network
                    println!("Manual pinning by DID not yet implemented.");
                    println!("Profiles are automatically pinned when:");
                    println!("  - A contact is accepted");
                    println!("  - A contact announces their profile");
                    println!();
                    println!("DID provided: {}", did);
                }

                ProfilePinAction::Unpin { did } => {
                    match engine.unpin_profile(&did) {
                        Ok(()) => {
                            println!("Unpinned profile: {}", did);
                        }
                        Err(e) => {
                            println!("Failed to unpin: {}", e);
                        }
                    }
                }

                ProfilePinAction::Announce => {
                    engine.init_identity()?;
                    let signed = engine.sign_and_pin_own_profile()?;
                    println!("Profile signed and ready to announce:");
                    println!("  DID: {}", signed.did());
                    println!("  Name: {}", signed.profile.display_name);
                    println!();
                    println!("Note: Profile will be broadcast when connected to the network.");
                    println!("Use 'syncengine serve' to start the P2P network.");
                }
            },
        },

        // ═══════════════════════════════════════════════════════════════════════
        // Packet Layer Commands (Indra's Network)
        // ═══════════════════════════════════════════════════════════════════════

        Commands::Packet { action } => match action {
            PacketAction::Keys => {
                engine.init_profile_keys()?;
                let did = engine.profile_did().unwrap();
                let seq = engine.log_head_sequence();

                println!("Profile Keys (Indra's Network):");
                println!("  DID: {}", did);
                println!("  Log Head Sequence: {}", seq);
            }

            PacketAction::Log => {
                engine.init_profile_keys()?;
                let seq = engine.log_head_sequence();
                let did = engine.profile_did().unwrap();

                println!("Own Packet Log:");
                println!("  Owner: {}", did);
                println!("  Head Sequence: {}", seq);
                println!();

                if seq == 0 {
                    if let Some(log) = engine.my_log() {
                        if log.head_sequence().is_none() {
                            println!("  (empty log)");
                        } else {
                            // Show first packet
                            if let Some(entry) = log.get(0) {
                                let env = &entry.envelope;
                                println!("  [0] {} - {}", env.timestamp, if env.is_global() { "global" } else { "sealed" });
                            }
                        }
                    }
                } else {
                    if let Some(log) = engine.my_log() {
                        for seq_num in 0..=seq {
                            if let Some(entry) = log.get(seq_num) {
                                let env = &entry.envelope;
                                let kind = if env.is_global() { "global" } else { "sealed" };
                                println!("  [{}] ts={} {}", seq_num, env.timestamp, kind);
                            }
                        }
                    }
                }
            }

            PacketAction::SendHeartbeat => {
                engine.init_profile_keys()?;
                let payload = syncengine_core::PacketPayload::Heartbeat {
                    timestamp: chrono::Utc::now().timestamp_millis(),
                };
                let address = syncengine_core::PacketAddress::Global;

                let seq = engine.create_and_broadcast_packet(payload, address).await?;
                let did = engine.profile_did().unwrap();

                println!("Sent heartbeat packet:");
                println!("  Sender: {}", did);
                println!("  Sequence: {}", seq);
                println!("  Type: global (public, broadcast to peers)");
            }

            PacketAction::Mirror { did } => {
                let did = Did::from_str(&did)
                    .map_err(|e| anyhow::anyhow!("Invalid DID: {}", e))?;

                match engine.mirror_head(&did) {
                    Some(seq) => {
                        println!("Mirror for {}:", did);
                        println!("  Head Sequence: {}", seq);
                        println!();

                        // Show packets
                        let packets = engine.mirror_packets_since(&did, 0)?;
                        if packets.is_empty() {
                            println!("  (no packets)");
                        } else {
                            for env in packets {
                                let kind = if env.is_global() { "global" } else { "sealed" };
                                println!("  [{}] ts={} {}", env.sequence, env.timestamp, kind);
                            }
                        }
                    }
                    None => {
                        println!("No mirror found for {}", did);
                        println!("(No packets received from this profile yet)");
                    }
                }
            }

            PacketAction::Mirrors => {
                println!("Mirrored Profiles:");
                println!();

                // Get all mirrored DIDs
                match engine.list_mirrored_dids() {
                    Ok(dids) => {
                        if dids.is_empty() {
                            println!("  (no mirrors)");
                        } else {
                            for did in dids {
                                let head = engine.mirror_head(&did).unwrap_or(0);
                                println!("  {} (head: {})", did, head);
                            }
                        }
                    }
                    Err(e) => {
                        println!("  Error: {}", e);
                    }
                }
            }
        },

        // ═══════════════════════════════════════════════════════════════════════
        // Chat Commands
        // ═══════════════════════════════════════════════════════════════════════

        Commands::Chat { action } => match action {
            ChatAction::List => {
                engine.init_profile_keys()?;

                println!("Conversations:");
                println!();

                let conversations = engine.list_conversations()?;
                if conversations.is_empty() {
                    println!("  (no conversations yet)");
                    println!();
                    println!("  Send a message to start a conversation:");
                    println!("  syncengine chat send <did> \"Hello!\"");
                } else {
                    for convo in conversations {
                        let preview = convo.preview(50).unwrap_or_else(|| "(no messages)".to_string());
                        let time = convo.last_message()
                            .map(|m| m.relative_time())
                            .unwrap_or_else(|| "".to_string());
                        let unread = convo.unread_count();
                        let unread_badge = if unread > 0 {
                            format!(" [{}]", unread)
                        } else {
                            String::new()
                        };

                        println!("  {} ({}){}",
                            convo.display_name(),
                            time,
                            unread_badge
                        );
                        println!("    \"{}\"", preview);
                        println!("    DID: {}", convo.contact_did);
                        println!();
                    }
                }
            }

            ChatAction::Show { did, limit } => {
                engine.init_profile_keys()?;

                let convo = engine.get_conversation(&did)?;

                println!("Conversation with {}:", convo.display_name());
                println!("DID: {}", convo.contact_did);
                println!("Messages: {}", convo.len());
                println!();

                let messages = convo.messages();
                let to_show: &[_] = if let Some(n) = limit {
                    let start = messages.len().saturating_sub(n);
                    &messages[start..]
                } else {
                    messages
                };

                if to_show.is_empty() {
                    println!("  (no messages)");
                } else {
                    for msg in to_show {
                        let sender = if msg.is_mine { "You" } else { &msg.display_sender() };
                        let time = msg.relative_time();
                        println!("  [{} - {}]", sender, time);
                        println!("    {}", msg.content);
                        println!();
                    }
                }
            }

            ChatAction::Send { did, message } => {
                engine.init_profile_keys()?;

                match engine.send_message(&did, &message).await {
                    Ok(seq) => {
                        println!("Message sent!");
                        println!("  Sequence: {}", seq);
                        println!("  To: {}", did);
                        println!("  Content: {}", message);
                    }
                    Err(e) => {
                        // Check if this is the "recipient key lookup not implemented" error
                        if e.to_string().contains("Recipient public key lookup not yet implemented") {
                            println!("Cannot send message: Contact key exchange not complete.");
                            println!();
                            println!("To send encrypted messages, you need to:");
                            println!("  1. Exchange contact invites with the recipient");
                            println!("  2. Both parties must accept the contact request");
                            println!("  3. Wait for key exchange to complete");
                            println!();
                            println!("Use 'syncengine contact generate-invite' to create an invite.");
                        } else {
                            return Err(e.into());
                        }
                    }
                }
            }

            ChatAction::Interactive { did } => {
                engine.init_profile_keys()?;
                engine.start_networking().await?;

                // Get contact name
                let peer = engine.get_peer_by_did(&did)?;
                let contact_name = peer
                    .and_then(|p| p.profile.as_ref().map(|pr| pr.display_name.clone()))
                    .unwrap_or_else(|| did.chars().take(12).collect());

                println!("Interactive chat with {} (Ctrl+C to exit)", contact_name);
                println!("DID: {}", did);
                println!("{}", "─".repeat(50));

                // Show recent history
                let convo = engine.get_conversation(&did)?;
                let recent: Vec<_> = convo.messages().iter().rev().take(10).collect();
                for msg in recent.into_iter().rev() {
                    let sender = if msg.is_mine { "You" } else { &contact_name };
                    println!("{}: {}", sender, msg.content);
                }

                if !convo.is_empty() {
                    println!("{}", "─".repeat(50));
                }

                println!("Type a message and press Enter to send.");
                println!();

                // Track last seen sequence for new message detection
                let mut last_seq = convo.last_received_sequence();

                // Set up stdin reader
                let stdin = tokio::io::stdin();
                let reader = tokio::io::BufReader::new(stdin);
                let mut lines = tokio::io::AsyncBufReadExt::lines(reader);

                // Interactive loop
                loop {
                    tokio::select! {
                        // Check for new messages every second
                        _ = tokio::time::sleep(Duration::from_secs(1)) => {
                            match engine.get_new_messages(&did, last_seq) {
                                Ok(new_msgs) => {
                                    for msg in new_msgs {
                                        println!("{}: {}", contact_name, msg.content);
                                        if msg.sequence > last_seq {
                                            last_seq = msg.sequence;
                                        }
                                    }
                                }
                                Err(_) => {} // Ignore errors in polling
                            }
                        }
                        // Read user input
                        line = lines.next_line() => {
                            match line {
                                Ok(Some(text)) => {
                                    let text = text.trim();
                                    if !text.is_empty() {
                                        match engine.send_message(&did, text).await {
                                            Ok(_seq) => {
                                                println!("You: {}", text);
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to send: {}", e);
                                            }
                                        }
                                    }
                                }
                                Ok(None) => {
                                    // EOF - stdin closed
                                    println!();
                                    println!("Input closed, exiting...");
                                    break;
                                }
                                Err(e) => {
                                    eprintln!("Read error: {}", e);
                                }
                            }
                        }
                        // Handle Ctrl+C
                        _ = tokio::signal::ctrl_c() => {
                            println!();
                            println!("Exiting chat...");
                            break;
                        }
                    }
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

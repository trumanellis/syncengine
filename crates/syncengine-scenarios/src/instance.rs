//! Instance lifecycle management for SyncEngine processes.
//!
//! Tracks launched instances, their state, and provides methods to control them.

#![allow(dead_code)]

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Represents the state of an instance
#[derive(Debug, Clone, PartialEq)]
pub enum InstanceState {
    /// Instance is running
    Running,
    /// Instance was killed
    Killed,
    /// Instance exited normally
    Exited(i32),
    /// Instance crashed
    Crashed,
}

/// Information about a running instance
#[derive(Debug)]
pub struct InstanceInfo {
    pub name: String,
    pub profile: String,
    pub state: InstanceState,
    pub process: Option<Child>,
    pub data_dir: PathBuf,
    pub position: u8,
}

/// Manages all running instances
#[derive(Debug)]
pub struct InstanceManager {
    instances: HashMap<String, InstanceInfo>,
    binary_path: PathBuf,
    data_base: PathBuf,
    bootstrap_base: PathBuf,
    next_position: u8,
}

impl InstanceManager {
    /// Create a new instance manager
    pub fn new() -> Result<Self> {
        Self::new_with_options(false)
    }

    /// Create a new instance manager with options
    ///
    /// # Arguments
    /// * `fresh` - If true, delete all existing data directories for a clean start
    pub fn new_with_options(fresh: bool) -> Result<Self> {
        // Find the binary
        let binary_path = Self::find_binary()?;

        // Get base data directory
        let data_base = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("syncengine-scenarios");

        // Bootstrap directory for cross-instance connections
        let bootstrap_base = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("syncengine-bootstrap");

        // If fresh start requested, delete existing data
        if fresh {
            if data_base.exists() {
                tracing::info!(path = %data_base.display(), "Clearing scenario data directory");
                std::fs::remove_dir_all(&data_base)?;
            }
            if bootstrap_base.exists() {
                tracing::info!(path = %bootstrap_base.display(), "Clearing bootstrap directory");
                std::fs::remove_dir_all(&bootstrap_base)?;
            }
        }

        // Ensure directories exist
        std::fs::create_dir_all(&data_base)?;
        std::fs::create_dir_all(&bootstrap_base)?;

        Ok(Self {
            instances: HashMap::new(),
            binary_path,
            data_base,
            bootstrap_base,
            next_position: 0,
        })
    }

    /// Create a new instance manager for testing (doesn't require binary)
    #[cfg(test)]
    pub fn new_for_testing(data_base: PathBuf, bootstrap_base: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&data_base)?;
        std::fs::create_dir_all(&bootstrap_base)?;

        Ok(Self {
            instances: HashMap::new(),
            binary_path: PathBuf::from("/fake/binary"),
            data_base,
            bootstrap_base,
            next_position: 0,
        })
    }

    /// Register a fake instance for testing (doesn't spawn a process)
    #[cfg(test)]
    pub fn register_fake_instance(&mut self, name: &str, profile: &str) {
        let data_dir = self.data_base.join(format!("instance-{}", name));
        let _ = std::fs::create_dir_all(&data_dir);

        self.instances.insert(
            name.to_string(),
            InstanceInfo {
                name: name.to_string(),
                profile: profile.to_string(),
                state: InstanceState::Running,
                process: None,
                data_dir,
                position: self.next_position,
            },
        );
        self.next_position += 1;
    }

    /// Find the SyncEngine binary
    fn find_binary() -> Result<PathBuf> {
        // Look for release binary first, then debug
        let candidates = [
            "target/release/syncengine-desktop",
            "target/debug/syncengine-desktop",
            "../target/release/syncengine-desktop",
            "../target/debug/syncengine-desktop",
        ];

        for candidate in candidates {
            let path = PathBuf::from(candidate);
            if path.exists() {
                return Ok(path.canonicalize()?);
            }
        }

        anyhow::bail!(
            "Could not find syncengine-desktop binary. Run 'cargo build --release' first."
        )
    }

    /// Launch a new instance with auto-connect support
    pub fn launch(&mut self, name: &str, profile: &str) -> Result<()> {
        self.launch_with_connect(name, profile, None, None)
    }

    /// Launch a new instance with optional auto-connect peers and expected total
    ///
    /// # Arguments
    /// * `name` - Instance name
    /// * `profile` - Profile name for the instance
    /// * `connect_peers` - Optional list of peer names to auto-connect to
    /// * `total_expected` - Optional expected total instance count for proper window tiling.
    ///   When provided (e.g., from declarative scenarios), enables correct tiling from the start.
    ///   When None, falls back to dynamic calculation based on currently launched instances.
    pub fn launch_with_connect(
        &mut self,
        name: &str,
        profile: &str,
        connect_peers: Option<Vec<String>>,
        total_expected: Option<u8>,
    ) -> Result<()> {
        if self.instances.contains_key(name) {
            anyhow::bail!("Instance '{}' already exists", name);
        }

        let data_dir = self.data_base.join(format!("instance-{}", name));
        std::fs::create_dir_all(&data_dir)?;

        let position = self.next_position;
        self.next_position += 1;
        // Use expected total if provided, otherwise fall back to dynamic calculation
        let total = total_expected.unwrap_or(self.instances.len() as u8 + 1);

        // Convert numeric position to string format expected by the binary
        let position_str = match (position, total) {
            (_, 1) => "maximized".to_string(),
            (0, 2) => "left".to_string(),
            (1, 2) => "right".to_string(),
            (0, _) => "left".to_string(),
            (p, t) if p == t - 1 => "right".to_string(),
            _ => "center".to_string(),
        };

        tracing::info!(
            name = %name,
            profile = %profile,
            position = %position_str,
            "Launching instance"
        );

        let mut cmd = Command::new(&self.binary_path);
        cmd.arg("--name")
            .arg(name)
            .arg("--position")
            .arg(&position_str)
            .arg("--total-windows")
            .arg(total.to_string())
            .arg("--init-profile-name")
            .arg(profile)
            .env("SYNCENGINE_DATA_DIR", &data_dir);

        // Pass through JSONL logging environment variables if set
        // This allows scenario-spawned instances to log to the same directory
        if let Ok(logs_dir) = std::env::var("SYNCENGINE_LOGS_DIR") {
            cmd.env("SYNCENGINE_LOGS_DIR", &logs_dir);
            cmd.env("SYNCENGINE_INSTANCE", name);
            tracing::debug!(name = %name, logs_dir = %logs_dir, "Passing JSONL logging to instance");
        }

        // Inherit RUST_LOG for consistent log levels
        if let Ok(rust_log) = std::env::var("RUST_LOG") {
            cmd.env("RUST_LOG", rust_log);
        }

        cmd.stdout(Stdio::null())
            .stderr(Stdio::null());

        // Add auto-connect peers if provided
        if let Some(peers) = connect_peers {
            if !peers.is_empty() {
                cmd.arg("--init-connect").arg(peers.join(","));
            }
        }

        tracing::debug!(
            name = %name,
            binary = %self.binary_path.display(),
            data_dir = %data_dir.display(),
            "Spawning instance process"
        );

        let child = cmd.spawn().context("Failed to spawn instance")?;

        self.instances.insert(
            name.to_string(),
            InstanceInfo {
                name: name.to_string(),
                profile: profile.to_string(),
                state: InstanceState::Running,
                process: Some(child),
                data_dir,
                position,
            },
        );

        Ok(())
    }

    /// Get all instance names (for auto-connect)
    pub fn get_all_names(&self) -> Vec<String> {
        self.instances.keys().cloned().collect()
    }

    /// Kill an instance
    pub fn kill(&mut self, name: &str) -> Result<()> {
        let info = self
            .instances
            .get_mut(name)
            .context(format!("Instance '{}' not found", name))?;

        if let Some(ref mut child) = info.process {
            tracing::info!(name = %name, "Killing instance");
            child.kill().context("Failed to kill process")?;
            info.state = InstanceState::Killed;
        }

        Ok(())
    }

    /// Restart a killed instance (reconnects to all other running instances)
    pub fn restart(&mut self, name: &str) -> Result<()> {
        let info = self
            .instances
            .get(name)
            .context(format!("Instance '{}' not found", name))?;

        if info.state == InstanceState::Running {
            anyhow::bail!("Instance '{}' is already running", name);
        }

        let profile = info.profile.clone();
        let data_dir = info.data_dir.clone();
        let position = info.position;

        // Get all other running instances for auto-connect
        let connect_peers: Vec<String> = self
            .instances
            .iter()
            .filter(|(n, i)| *n != name && i.state == InstanceState::Running)
            .map(|(n, _)| n.clone())
            .collect();

        // Remove old entry
        self.instances.remove(name);

        // Relaunch
        let total = self.instances.len() as u8 + 1;

        // Convert numeric position to string format expected by the binary
        let position_str = match (position, total) {
            (_, 1) => "maximized".to_string(),
            (0, 2) => "left".to_string(),
            (1, 2) => "right".to_string(),
            (0, _) => "left".to_string(),
            (p, t) if p == t - 1 => "right".to_string(),
            _ => "center".to_string(),
        };

        tracing::info!(
            name = %name,
            profile = %profile,
            position = %position_str,
            connect_peers = ?connect_peers,
            "Restarting instance"
        );

        let mut cmd = Command::new(&self.binary_path);
        cmd.arg("--name")
            .arg(name)
            .arg("--position")
            .arg(&position_str)
            .arg("--total-windows")
            .arg(total.to_string())
            .arg("--init-profile-name")
            .arg(&profile)
            .env("SYNCENGINE_DATA_DIR", &data_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        // Add auto-connect to other running instances
        if !connect_peers.is_empty() {
            cmd.arg("--init-connect").arg(connect_peers.join(","));
        }

        let child = cmd.spawn().context("Failed to spawn instance")?;

        self.instances.insert(
            name.to_string(),
            InstanceInfo {
                name: name.to_string(),
                profile,
                state: InstanceState::Running,
                process: Some(child),
                data_dir,
                position,
            },
        );

        Ok(())
    }

    /// Get all instance names
    pub fn list_instances(&self) -> Vec<String> {
        self.instances.keys().cloned().collect()
    }

    /// Get a random running instance name
    pub fn random_instance(&self) -> Option<String> {
        use rand::seq::IteratorRandom;
        let mut rng = rand::rng();

        self.instances
            .iter()
            .filter(|(_, info)| info.state == InstanceState::Running)
            .map(|(name, _)| name.clone())
            .choose(&mut rng)
    }

    /// Check if an instance is running
    pub fn is_running(&self, name: &str) -> bool {
        self.instances
            .get(name)
            .map(|info| info.state == InstanceState::Running)
            .unwrap_or(false)
    }

    /// Get instance data directory (for bootstrap invites)
    pub fn get_data_dir(&self, name: &str) -> Option<PathBuf> {
        self.instances.get(name).map(|info| info.data_dir.clone())
    }

    /// Kill all instances
    pub fn kill_all(&mut self) {
        for (name, info) in self.instances.iter_mut() {
            if let Some(ref mut child) = info.process {
                tracing::info!(name = %name, "Killing instance");
                let _ = child.kill();
                info.state = InstanceState::Killed;
            }
        }
    }

    /// Check if all instances have exited (manually quit or killed)
    /// Updates state for any instances that have exited since last check
    pub fn all_instances_exited(&mut self) -> bool {
        if self.instances.is_empty() {
            return false; // No instances launched yet
        }

        let mut all_exited = true;
        for (name, info) in self.instances.iter_mut() {
            if info.state == InstanceState::Running {
                if let Some(ref mut child) = info.process {
                    // Check if the process has exited without blocking
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            // Process has exited
                            let exit_code = status.code().unwrap_or(-1);
                            tracing::info!(name = %name, exit_code, "Instance exited");
                            info.state = InstanceState::Exited(exit_code);
                        }
                        Ok(None) => {
                            // Process is still running
                            all_exited = false;
                        }
                        Err(e) => {
                            tracing::warn!(name = %name, error = ?e, "Failed to check instance status");
                            all_exited = false;
                        }
                    }
                }
            }
        }

        all_exited
    }

    /// Get bootstrap directory for cross-instance connections
    /// This must match the path used in app.rs for invite file exchange
    pub fn bootstrap_dir(&self) -> PathBuf {
        self.bootstrap_base.clone()
    }
}

impl Drop for InstanceManager {
    fn drop(&mut self) {
        self.kill_all();
    }
}

/// Thread-safe wrapper around InstanceManager
pub type SharedInstanceManager = Arc<RwLock<InstanceManager>>;

pub fn create_shared_manager() -> Result<SharedInstanceManager> {
    create_shared_manager_with_options(false)
}

pub fn create_shared_manager_with_options(fresh: bool) -> Result<SharedInstanceManager> {
    Ok(Arc::new(RwLock::new(InstanceManager::new_with_options(fresh)?)))
}

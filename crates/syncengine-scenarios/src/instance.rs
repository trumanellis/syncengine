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
    next_position: u8,
}

impl InstanceManager {
    /// Create a new instance manager
    pub fn new() -> Result<Self> {
        // Find the binary
        let binary_path = Self::find_binary()?;

        // Get base data directory
        let data_base = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("syncengine-scenarios");

        // Ensure base directory exists
        std::fs::create_dir_all(&data_base)?;

        Ok(Self {
            instances: HashMap::new(),
            binary_path,
            data_base,
            next_position: 0,
        })
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
        self.launch_with_connect(name, profile, None)
    }

    /// Launch a new instance with optional auto-connect peers
    pub fn launch_with_connect(
        &mut self,
        name: &str,
        profile: &str,
        connect_peers: Option<Vec<String>>,
    ) -> Result<()> {
        if self.instances.contains_key(name) {
            anyhow::bail!("Instance '{}' already exists", name);
        }

        let data_dir = self.data_base.join(format!("instance-{}", name));
        std::fs::create_dir_all(&data_dir)?;

        let position = self.next_position;
        self.next_position += 1;
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
            .env("SYNCENGINE_DATA_DIR", &data_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        // Add auto-connect peers if provided
        if let Some(peers) = connect_peers {
            if !peers.is_empty() {
                cmd.arg("--init-connect").arg(peers.join(","));
            }
        }

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

    /// Get bootstrap directory for cross-instance connections
    pub fn bootstrap_dir(&self) -> PathBuf {
        self.data_base.join("bootstrap")
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
    Ok(Arc::new(RwLock::new(InstanceManager::new()?)))
}

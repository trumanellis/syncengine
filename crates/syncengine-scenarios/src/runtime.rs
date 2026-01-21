//! Lua runtime setup and scenario execution.

#![allow(dead_code)]

use anyhow::{Context, Result};
use mlua::{Lua, Table};
use std::path::{Path, PathBuf};

use crate::api::create_context_table;
use crate::instance::{create_shared_manager, SharedInstanceManager};
use crate::scheduler::{create_shared_scheduler, run_scheduler_loop, SharedScheduler};

/// The scenario runtime that manages Lua execution
pub struct ScenarioRuntime {
    lua: Lua,
    scenarios_dir: PathBuf,
    instances: SharedInstanceManager,
    scheduler: SharedScheduler,
}

impl ScenarioRuntime {
    /// Create a new scenario runtime
    pub fn new(scenarios_dir: PathBuf) -> Result<Self> {
        let lua = Lua::new();
        let instances = create_shared_manager()?;
        let scheduler = create_shared_scheduler();

        // Set up Lua package.path to find modules in scenarios directory
        // This allows scenarios to use: require("lib.helpers")
        let scenarios_path = scenarios_dir.to_string_lossy();
        let package_path_setup = format!(
            r#"package.path = "{0}/?.lua;{0}/?/init.lua;" .. package.path"#,
            scenarios_path
        );
        lua.load(&package_path_setup)
            .exec()
            .context("Failed to set package.path")?;

        Ok(Self {
            lua,
            scenarios_dir,
            instances,
            scheduler,
        })
    }

    /// Run a scenario file
    pub async fn run_scenario(&mut self, path: &Path) -> Result<()> {
        let scenario_code = std::fs::read_to_string(path)
            .context(format!("Failed to read scenario file: {:?}", path))?;

        // Create context table
        let ctx = create_context_table(&self.lua, self.instances.clone(), self.scheduler.clone())?;

        // Create the scenario() function that scenarios call
        let instances = self.instances.clone();
        let scenario_fn = self.lua.create_function(move |lua, config: Table| {
            // Extract scenario configuration
            let name: String = config.get("name").unwrap_or_else(|_| "unnamed".to_string());
            let description: String = config
                .get("description")
                .unwrap_or_else(|_| "No description".to_string());

            tracing::info!(
                name = %name,
                description = %description,
                "Running scenario"
            );

            // Check topology - if mesh, we need to launch with auto-connect
            let is_mesh = config
                .get::<String>("topology")
                .map(|t| t == "mesh")
                .unwrap_or(false);

            // Collect all instance names first (for mesh auto-connect)
            let mut all_instance_names: Vec<String> = Vec::new();
            if let Ok(instances_table) = config.get::<Table>("instances") {
                for pair in instances_table.pairs::<i32, Table>() {
                    if let Ok((_, instance)) = pair {
                        let inst_name: String =
                            instance.get("name").unwrap_or_else(|_| "unknown".to_string());
                        all_instance_names.push(inst_name);
                    }
                }
            }

            // Handle instances array if present
            if let Ok(instances_table) = config.get::<Table>("instances") {
                // Calculate total expected instances for proper window tiling
                let total_expected = Some(all_instance_names.len() as u8);

                for pair in instances_table.pairs::<i32, Table>() {
                    if let Ok((_, instance)) = pair {
                        let inst_name: String =
                            instance.get("name").unwrap_or_else(|_| "unknown".to_string());
                        let profile: String = instance
                            .get("profile")
                            .unwrap_or_else(|_| capitalize(&inst_name));

                        // For mesh topology, connect to all other instances
                        let connect_peers = if is_mesh {
                            Some(
                                all_instance_names
                                    .iter()
                                    .filter(|n| *n != &inst_name)
                                    .cloned()
                                    .collect(),
                            )
                        } else {
                            None
                        };

                        // Launch instance with auto-connect and expected total for tiling
                        let instances_clone = instances.clone();
                        tokio::task::block_in_place(|| {
                            let rt = tokio::runtime::Handle::current();
                            rt.block_on(async {
                                let mut mgr = instances_clone.write().await;
                                if let Err(e) = mgr.launch_with_connect(&inst_name, &profile, connect_peers, total_expected) {
                                    tracing::error!(name = %inst_name, error = %e, "Failed to launch");
                                }
                            });
                        });

                        // Small delay between launches
                        std::thread::sleep(std::time::Duration::from_millis(500));
                    }
                }
            }

            // Log topology info
            if is_mesh {
                tracing::info!(instances = ?all_instance_names, "Created mesh topology with auto-connect");
            }

            // Call on_start if present
            if let Ok(on_start) = config.get::<mlua::Function>("on_start") {
                // Get ctx from globals
                let ctx: Table = lua.globals().get("ctx")?;
                if let Err(e) = on_start.call::<()>(ctx) {
                    tracing::error!(error = %e, "on_start callback failed");
                }
            }

            // Call on_running if present (for ongoing behaviors like chaos)
            if let Ok(on_running) = config.get::<mlua::Function>("on_running") {
                let ctx: Table = lua.globals().get("ctx")?;
                if let Err(e) = on_running.call::<()>(ctx) {
                    tracing::error!(error = %e, "on_running callback failed");
                }
            }

            Ok(())
        })?;

        // Set globals
        self.lua.globals().set("scenario", scenario_fn)?;
        self.lua.globals().set("ctx", ctx)?;

        // Add helper functions to globals
        let capitalize_fn = self.lua.create_function(|_, s: String| Ok(capitalize(&s)))?;
        self.lua.globals().set("capitalize", capitalize_fn)?;

        let random_fn = self.lua.create_function(|_, (min, max): (i32, i32)| {
            use rand::Rng;
            let mut rng = rand::rng();
            Ok(rng.random_range(min..=max))
        })?;
        self.lua.globals().set("random", random_fn)?;

        // Execute scenario
        self.lua
            .load(&scenario_code)
            .set_name(path.file_name().unwrap().to_str().unwrap())
            .exec()
            .context("Failed to execute scenario")?;

        // Run scheduler loop to handle after() and every() callbacks
        run_scheduler_loop(&self.lua, self.scheduler.clone()).await?;

        // Wait a bit for instances to settle, then prompt user
        tracing::info!("Scenario running. Press Ctrl+C to stop all instances.");

        // Keep running until interrupted
        tokio::signal::ctrl_c().await?;

        tracing::info!("Shutting down scenario...");

        // Kill all instances
        {
            let mut mgr = self.instances.write().await;
            mgr.kill_all();
        }

        Ok(())
    }
}

/// Capitalize first letter
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

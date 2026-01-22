//! Scenario Runner for SyncEngine
//!
//! Runs Lua-based test scenarios that orchestrate multiple SyncEngine instances.
//!
//! Usage:
//!   syncengine-scenario <scenario-name>
//!   syncengine-scenario mesh-3
//!   syncengine-scenario --fresh offline-relay  # Clean start, delete old data

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use syncengine_scenarios::runtime::ScenarioRuntime;

/// SyncEngine Scenario Runner
#[derive(Parser, Debug)]
#[command(name = "syncengine-scenario")]
#[command(about = "Run Lua-based test scenarios for SyncEngine")]
struct Args {
    /// Scenario name (without .lua extension)
    /// Looks for scenarios/<name>.lua
    scenario: String,

    /// Scenarios directory (default: ./scenarios)
    #[arg(short, long)]
    scenarios_dir: Option<PathBuf>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Fresh start: delete all existing instance data before running
    /// Use this for clean test runs without leftover contacts/messages
    #[arg(short, long)]
    fresh: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Check for JSONL logging environment variables
    let logs_dir = std::env::var("SYNCENGINE_LOGS_DIR").ok();
    let instance_name = std::env::var("SYNCENGINE_INSTANCE")
        .unwrap_or_else(|_| format!("scenario-{}", args.scenario));

    // Initialize tracing
    let filter = if args.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    };

    // Set up tracing with optional JSONL layer
    if let Some(ref logs_dir) = logs_dir {
        match syncengine_core::logging::JsonlLayer::new(logs_dir, &instance_name) {
            Ok(jsonl_layer) => {
                let subscriber = tracing_subscriber::registry()
                    .with(jsonl_layer)
                    .with(
                        tracing_subscriber::fmt::layer()
                            .with_target(false),
                    )
                    .with(filter);
                tracing::subscriber::set_global_default(subscriber)
                    .expect("Failed to set global subscriber");
                tracing::info!(
                    logs_dir = %logs_dir,
                    instance = %instance_name,
                    "JSONL logging enabled for scenario"
                );
            }
            Err(e) => {
                // Fall back to console-only logging
                tracing_subscriber::fmt()
                    .with_env_filter(filter)
                    .with_target(false)
                    .init();
                tracing::warn!("Failed to initialize JSONL logging: {}", e);
            }
        }
    } else {
        // Standard console logging
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .init();
    }

    // Determine scenarios directory
    let scenarios_dir = args.scenarios_dir.unwrap_or_else(|| PathBuf::from("scenarios"));

    // Build scenario file path
    let scenario_file = scenarios_dir.join(format!("{}.lua", args.scenario));

    if !scenario_file.exists() {
        anyhow::bail!(
            "Scenario '{}' not found at {:?}",
            args.scenario,
            scenario_file
        );
    }

    if args.fresh {
        tracing::info!("Fresh start requested - will delete existing instance data");
    }
    tracing::info!("Loading scenario: {}", args.scenario);

    // Create and run the scenario runtime
    let mut runtime = ScenarioRuntime::new_with_options(scenarios_dir, args.fresh)?;

    // Load and execute the scenario
    runtime
        .run_scenario(&scenario_file)
        .await
        .context("Failed to run scenario")?;

    tracing::info!("Scenario '{}' completed", args.scenario);

    Ok(())
}

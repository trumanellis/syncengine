//! Scenario Runner for SyncEngine
//!
//! Runs Lua-based test scenarios that orchestrate multiple SyncEngine instances.
//!
//! Usage:
//!   syncengine-scenario <scenario-name>
//!   syncengine-scenario mesh-3
//!   syncengine-scenario chaos

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

mod api;
mod instance;
mod runtime;
mod scheduler;

use runtime::ScenarioRuntime;

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
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    let filter = if args.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

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

    tracing::info!("Loading scenario: {}", args.scenario);

    // Create and run the scenario runtime
    let mut runtime = ScenarioRuntime::new(scenarios_dir)?;

    // Load and execute the scenario
    runtime
        .run_scenario(&scenario_file)
        .await
        .context("Failed to run scenario")?;

    tracing::info!("Scenario '{}' completed", args.scenario);

    Ok(())
}

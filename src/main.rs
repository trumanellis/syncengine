#![allow(non_snake_case)]

mod app;
mod components;
pub mod context;
mod pages;
mod theme;

use std::path::PathBuf;
use std::sync::OnceLock;

use clap::Parser;
use dioxus::desktop::{Config, WindowBuilder};

/// Global data directory, set from command line
static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Get the data directory (set from command line or default)
pub fn get_data_dir() -> PathBuf {
    DATA_DIR.get().cloned().unwrap_or_else(|| {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("syncengine")
    })
}

/// Synchronicity Engine - P2P Task Sharing
#[derive(Parser, Debug)]
#[command(name = "syncengine-desktop")]
#[command(about = "Synchronicity Engine - Local-first P2P task sharing")]
struct Args {
    /// Data directory for storage (use different dirs for multiple instances)
    #[arg(short, long)]
    data_dir: Option<PathBuf>,

    /// Instance number (shorthand for --data-dir with suffix)
    #[arg(short, long)]
    instance: Option<u8>,
}

fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    // Determine data directory
    let data_dir = if let Some(dir) = args.data_dir {
        dir
    } else if let Some(instance) = args.instance {
        let base = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("syncengine");
        if instance == 1 {
            base
        } else {
            base.with_file_name(format!("syncengine-{}", instance))
        }
    } else {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("syncengine")
    };

    // Store data directory globally
    let _ = DATA_DIR.set(data_dir.clone());

    // Get screen dimensions for half-width window
    // Default to reasonable size, will be adjusted by window manager
    let window_width = 700.0;  // Half of typical 1400px screen
    let window_height = 900.0; // Nearly full height

    // Window title with instance indicator
    let title = if let Some(instance) = args.instance {
        format!("Synchronicity Engine (Instance {})", instance)
    } else {
        "Synchronicity Engine".to_string()
    };

    tracing::info!("Starting with data dir: {:?}", data_dir);

    // Configure desktop window
    let config = Config::new()
        .with_window(
            WindowBuilder::new()
                .with_title(&title)
                .with_inner_size(dioxus::desktop::LogicalSize::new(window_width, window_height))
                .with_resizable(true)
        );

    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(app::App);
}

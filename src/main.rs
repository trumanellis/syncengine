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

    /// Instance name (creates data dir: syncengine-<name>)
    #[arg(short, long)]
    name: Option<String>,

    /// Instance number (shorthand for --name with number)
    #[arg(short, long)]
    instance: Option<u8>,
}

fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    // Determine data directory and display name
    let (data_dir, display_name) = if let Some(dir) = args.data_dir {
        (dir.clone(), dir.file_name().and_then(|n| n.to_str()).unwrap_or("custom").to_string())
    } else if let Some(ref name) = args.name {
        let base = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(format!("syncengine-{}", name));
        (base, name.clone())
    } else if let Some(instance) = args.instance {
        let base = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."));
        if instance == 1 {
            (base.join("syncengine"), format!("Instance {}", instance))
        } else {
            (base.join(format!("syncengine-{}", instance)), format!("Instance {}", instance))
        }
    } else {
        let base = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("syncengine");
        (base, String::new())
    };

    // Store data directory globally
    let _ = DATA_DIR.set(data_dir.clone());

    // Window size: half width, nearly full height
    let window_width = 700.0;
    let window_height = 900.0;

    // Window title with instance name
    let title = if !display_name.is_empty() {
        format!("Synchronicity Engine - {}", display_name)
    } else {
        "Synchronicity Engine".to_string()
    };

    tracing::info!("Starting '{}' with data dir: {:?}", display_name, data_dir);

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

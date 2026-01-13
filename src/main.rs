#![allow(non_snake_case)]

mod app;
mod components;
pub mod context;
mod pages;
mod theme;

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use clap::{Parser, ValueEnum};
use dioxus::desktop::{Config, LogicalPosition, WindowBuilder};

/// Get the primary screen dimensions on macOS
fn get_screen_size() -> (f64, f64) {
    // Try to get screen size using osascript on macOS
    if let Ok(output) = Command::new("osascript")
        .args(["-e", "tell application \"Finder\" to get bounds of window of desktop"])
        .output()
    {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            // Output format: "0, 0, 1440, 900" (x1, y1, x2, y2)
            let parts: Vec<&str> = stdout.trim().split(", ").collect();
            if parts.len() == 4 {
                if let (Ok(width), Ok(height)) = (
                    parts[2].parse::<f64>(),
                    parts[3].parse::<f64>(),
                ) {
                    return (width, height);
                }
            }
        }
    }
    // Fallback to common MacBook resolution
    (1440.0, 900.0)
}

/// Global data directory, set from command line
static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Window position on screen
#[derive(Debug, Clone, ValueEnum)]
enum WindowPosition {
    Maximized,
    Left,
    Center,
    Right,
}

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

    /// Instance name (creates data dir: instance-<name>)
    #[arg(short, long)]
    name: Option<String>,

    /// Instance number (shorthand for --name with number)
    #[arg(short, long)]
    instance: Option<u8>,

    /// Window position (maximized, left, center, or right)
    #[arg(short, long, value_enum)]
    position: Option<WindowPosition>,

    /// Total number of windows (for calculating split width)
    #[arg(short, long, default_value = "1")]
    total_windows: u8,
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
            .join(format!("instance-{}", name));
        (base, name.clone())
    } else if let Some(instance) = args.instance {
        let base = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."));
        if instance == 1 {
            (base.join("syncengine"), format!("Instance {}", instance))
        } else {
            (base.join(format!("instance-{}", instance)), format!("Instance {}", instance))
        }
    } else {
        let base = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("syncengine");
        (base, String::new())
    };

    // Store data directory globally
    let _ = DATA_DIR.set(data_dir.clone());

    // Get screen dimensions and calculate window size
    let (screen_width, screen_height) = get_screen_size();

    // Calculate window width based on total windows
    let window_width = match args.position {
        Some(WindowPosition::Maximized) => screen_width,
        _ => screen_width / args.total_windows as f64,
    };
    let window_height = screen_height - 25.0;

    // Window title with instance name
    let title = if !display_name.is_empty() {
        format!("Synchronicity Engine - {}", display_name)
    } else {
        "Synchronicity Engine".to_string()
    };

    tracing::info!(
        "Starting '{}' with data dir: {:?}, screen: {}x{}, window: {}x{}, total_windows: {}",
        display_name, data_dir, screen_width, screen_height, window_width, window_height, args.total_windows
    );

    // Determine window position based on position enum and window width
    let window_x = match args.position {
        Some(WindowPosition::Maximized) => 0,
        Some(WindowPosition::Left) => 0,
        Some(WindowPosition::Center) => window_width as i32,
        Some(WindowPosition::Right) => {
            if args.total_windows == 2 {
                window_width as i32
            } else {
                (window_width * 2.0) as i32
            }
        }
        None => 0,
    };

    // Configure desktop window
    let mut window_builder = WindowBuilder::new()
        .with_title(&title)
        .with_inner_size(dioxus::desktop::LogicalSize::new(window_width, window_height))
        .with_resizable(true);

    // Set position if specified (y=25 accounts for menu bar)
    if args.position.is_some() {
        window_builder = window_builder.with_position(LogicalPosition::new(window_x, 25));
    }

    let config = Config::new().with_window(window_builder);

    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(app::App);
}

#![allow(non_snake_case)]

mod app;
mod components;
pub mod context;
mod pages;
mod theme;

use dioxus::desktop::{Config, WindowBuilder};

fn main() {
    tracing_subscriber::fmt::init();

    // Configure desktop window
    let config = Config::new()
        .with_window(
            WindowBuilder::new()
                .with_title("Synchronicity Engine")
                .with_inner_size(dioxus::desktop::LogicalSize::new(1200.0, 800.0))
                .with_resizable(true)
        );

    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(app::App);
}

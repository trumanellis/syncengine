#![allow(non_snake_case)]

mod app;
mod components;
mod pages;
mod theme;

fn main() {
    tracing_subscriber::fmt::init();
    dioxus::launch(app::App);
}

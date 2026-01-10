use dioxus::prelude::*;

use crate::pages::{Field, Landing};
use crate::theme::GLOBAL_STYLES;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[route("/")]
    Landing {},
    #[route("/field")]
    Field {},
}

#[component]
pub fn App() -> Element {
    rsx! {
        style { {GLOBAL_STYLES} }
        Router::<Route> {}
    }
}

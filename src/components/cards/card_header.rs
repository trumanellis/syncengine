//! Card Header Component
//!
//! Displays title, subtitle, and optional link in card headers.

use dioxus::prelude::*;

/// Card header with title, subtitle, and optional link
///
/// # Examples
///
/// ```rust
/// rsx! {
///     CardHeader {
///         title: "Alice Wonderland",
///         subtitle: Some("Architect of Dreams".to_string()),
///         link: Some("sync.local/alice".to_string()),
///     }
/// }
/// ```
#[component]
pub fn CardHeader(
    /// Main title (large, gold, serif)
    title: String,
    /// Optional subtitle (small, uppercase, moss)
    #[props(default = None)]
    subtitle: Option<String>,
    /// Optional link (cyan, monospace)
    #[props(default = None)]
    link: Option<String>,
) -> Element {
    rsx! {
        div { class: "card-header",
            // Title
            h2 { class: "card-header__title",
                "{title}"
            }

            // Subtitle
            if let Some(sub) = subtitle {
                div { class: "card-header__subtitle",
                    "{sub}"
                }
            }

            // Link
            if let Some(link_text) = link {
                a {
                    class: "card-header__link",
                    href: "#",
                    onclick: move |e| {
                        e.prevent_default();
                        // TODO: Navigation or copy to clipboard
                    },
                    "{link_text}"
                }
            }
        }
    }
}

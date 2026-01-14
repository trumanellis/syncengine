//! Golden Card Primitive
//!
//! Base component enforcing golden ratio (1:1.618) aspect ratios.

use dioxus::prelude::*;

/// Card orientation determines aspect ratio and internal layout
#[derive(Clone, Copy, PartialEq)]
pub enum CardOrientation {
    /// Portrait: 1:1.618 (height > width)
    /// Grid: 38.2% (left) | 61.8% (right)
    Portrait,
    /// Landscape: 1.618:1 (width > height)
    /// Grid: 38.2% (top) / 61.8% (bottom)
    Landscape,
}

/// Golden Rectangle Card with enforced aspect ratio
///
/// # Examples
///
/// ```rust
/// rsx! {
///     GoldenCard {
///         orientation: CardOrientation::Portrait,
///         interactive: true,
///         div { class: "card-image-area", /* Image */ }
///         div { class: "card-content", /* Content */ }
///     }
/// }
/// ```
#[component]
pub fn GoldenCard(
    /// Card orientation (Portrait or Landscape)
    orientation: CardOrientation,
    /// Enable hover effects and cursor pointer
    #[props(default = false)]
    interactive: bool,
    /// Card contents (typically two divs: image area + content area)
    children: Element,
) -> Element {
    // Calculate aspect ratio
    let (aspect_w, aspect_h) = match orientation {
        CardOrientation::Portrait => (1.0, 1.618),
        CardOrientation::Landscape => (1.618, 1.0),
    };

    // CSS classes
    let orientation_class = match orientation {
        CardOrientation::Portrait => "golden-card--portrait",
        CardOrientation::Landscape => "golden-card--landscape",
    };

    let interactive_class = if interactive { "interactive" } else { "" };

    rsx! {
        div {
            class: "golden-card {orientation_class} {interactive_class}",
            style: "aspect-ratio: {aspect_w} / {aspect_h};",

            // Interior grid layout (38.2% / 61.8% split)
            div { class: "golden-card__interior",
                {children}
            }
        }
    }
}

//! Seed of Life Sacred Geometry Component
//!
//! Renders the Seed of Life pattern as an ambient background element.
//! This sacred geometry pattern consists of seven overlapping circles
//! arranged in sixfold symmetry.

use dioxus::prelude::*;

/// Properties for the SeedOfLife component
#[derive(Clone, PartialEq, Props)]
pub struct SeedOfLifeProps {
    /// Size of the SVG in pixels (default: 600)
    #[props(default = 600)]
    pub size: u32,
    /// Opacity of the pattern (default: 0.15)
    #[props(default = 0.15)]
    pub opacity: f32,
    /// Stroke color (default: gold #d4af37)
    #[props(default = "#d4af37".to_string())]
    pub stroke_color: String,
}

/// Renders the Seed of Life sacred geometry pattern as an inline SVG
///
/// # Design Notes
///
/// The Seed of Life consists of:
/// - One center circle
/// - Six surrounding circles, each passing through the center
/// - All circles have the same radius
///
/// # Example
///
/// ```rust,ignore
/// rsx! {
///     div { class: "relative",
///         SeedOfLife { size: 600, opacity: 0.15 }
///         // Content goes on top
///     }
/// }
/// ```
#[component]
pub fn SeedOfLife(props: SeedOfLifeProps) -> Element {
    let size = props.size;
    let opacity = props.opacity;
    let stroke = &props.stroke_color;

    rsx! {
        div {
            class: "seed-of-life-container",
            style: "position: absolute; inset: 0; display: flex; align-items: center; justify-content: center; pointer-events: none; overflow: hidden;",
            svg {
                view_box: "0 0 200 200",
                width: "{size}",
                height: "{size}",
                style: "opacity: {opacity};",
                "aria-hidden": "true",
                g {
                    fill: "none",
                    stroke: "{stroke}",
                    stroke_width: "0.5",
                    // Center circle
                    circle { cx: "100", cy: "100", r: "30" }
                    // Six surrounding circles (hexagonal arrangement)
                    // Top
                    circle { cx: "100", cy: "70", r: "30" }
                    // Top-right
                    circle { cx: "126", cy: "85", r: "30" }
                    // Bottom-right
                    circle { cx: "126", cy: "115", r: "30" }
                    // Bottom
                    circle { cx: "100", cy: "130", r: "30" }
                    // Bottom-left
                    circle { cx: "74", cy: "115", r: "30" }
                    // Top-left
                    circle { cx: "74", cy: "85", r: "30" }
                }
            }
        }
    }
}

/// A background wrapper that includes the Seed of Life pattern
///
/// This component creates a positioned container with the sacred geometry
/// pattern as a background layer.
///
/// # Example
///
/// ```rust,ignore
/// rsx! {
///     SeedOfLifeBackground {
///         div { class: "content",
///             // Your content here
///         }
///     }
/// }
/// ```
#[component]
pub fn SeedOfLifeBackground(children: Element) -> Element {
    rsx! {
        div {
            class: "seed-of-life-bg-wrapper",
            style: "position: relative; min-height: 100%;",
            SeedOfLife {}
            div {
                style: "position: relative; z-index: 1;",
                {children}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_props() {
        let props = SeedOfLifeProps {
            size: 600,
            opacity: 0.15,
            stroke_color: "#d4af37".to_string(),
        };
        assert_eq!(props.size, 600);
        assert!((props.opacity - 0.15).abs() < f32::EPSILON);
        assert_eq!(props.stroke_color, "#d4af37");
    }
}

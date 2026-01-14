//! Markdown Editor and Renderer
//!
//! Edit/preview toggle and read-only markdown display.

use dioxus::prelude::*;
use pulldown_cmark::{html, Options, Parser};

/// Markdown editor with live preview
///
/// # Examples
///
/// ```rust
/// let mut content = use_signal(|| "# Hello\nWorld".to_string());
///
/// rsx! {
///     MarkdownEditor {
///         content: content(),
///         on_change: move |new_content| content.set(new_content),
///     }
/// }
/// ```
#[component]
pub fn MarkdownEditor(
    /// Current markdown content
    content: ReadOnlySignal<String>,
    /// Callback when content changes
    on_change: EventHandler<String>,
) -> Element {
    let mut preview_mode = use_signal(|| false);

    // Convert markdown to HTML for preview
    let html_preview = use_memo(move || {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);

        let content_str = content();
        let parser = Parser::new_ext(&content_str, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        html_output
    });

    rsx! {
        div { class: "markdown-editor",
            // Toolbar
            div { class: "markdown-toolbar",
                button {
                    class: if !preview_mode() { "active" } else { "" },
                    onclick: move |_| preview_mode.set(false),
                    "Edit"
                }
                button {
                    class: if preview_mode() { "active" } else { "" },
                    onclick: move |_| preview_mode.set(true),
                    "Preview"
                }
            }

            // Editor or preview
            if preview_mode() {
                div {
                    class: "markdown-preview card-markdown",
                    dangerous_inner_html: "{html_preview()}",
                }
            } else {
                textarea {
                    class: "markdown-textarea",
                    value: "{content()}",
                    oninput: move |e| on_change.call(e.value()),
                    placeholder: "Write your thoughts in markdown...",
                }
            }
        }
    }
}

/// Read-only markdown renderer with optional collapse
///
/// # Examples
///
/// ```rust
/// rsx! {
///     MarkdownRenderer {
///         content: "# Title\nSome content".to_string(),
///         collapsible: true,
///         collapsed: false,
///     }
/// }
/// ```
#[component]
pub fn MarkdownRenderer(
    /// Markdown content to render
    content: ReadOnlySignal<String>,
    /// Enable collapse functionality
    #[props(default = false)]
    collapsible: bool,
    /// Initial collapsed state
    #[props(default = false)]
    collapsed: bool,
) -> Element {
    let mut is_collapsed = use_signal(|| collapsed);

    // Convert markdown to HTML
    let html_content = use_memo(move || {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);

        let content_str = content();
        let parser = Parser::new_ext(&content_str, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        html_output
    });

    let collapsed_class = if is_collapsed() {
        "card-markdown--collapsed"
    } else {
        ""
    };

    rsx! {
        div { class: "markdown-renderer",
            div {
                class: "card-markdown {collapsed_class}",
                dangerous_inner_html: "{html_content()}",
            }

            // Collapse toggle
            if collapsible {
                button {
                    class: "markdown-toggle",
                    onclick: move |_| is_collapsed.set(!is_collapsed()),
                    if is_collapsed() {
                        "Expand"
                    } else {
                        "Collapse"
                    }
                }
            }
        }
    }
}

//! Markdown Editor with Live Preview
//!
//! Split-pane editor with markdown toolbar and real-time preview

use dioxus::prelude::*;
use crate::components::cards::MarkdownRenderer;

/// Markdown Editor Component
///
/// Features:
/// - Toolbar with common markdown formatting buttons
/// - Split view: editor on left, preview on right
/// - Keyboard shortcuts for formatting
#[component]
pub fn MarkdownEditor(
    /// Current markdown content
    value: Signal<String>,
    /// Placeholder text
    #[props(default = "Write your description...".to_string())]
    placeholder: String,
    /// Minimum height in pixels
    #[props(default = 300)]
    min_height: u32,
) -> Element {
    let mut show_preview = use_signal(|| true);

    // Toolbar actions
    let insert_bold = move |_: MouseEvent| {
        let current = value.read().clone();
        let new_text = format!("{}**bold text**", current);
        value.set(new_text);
    };

    let insert_italic = move |_: MouseEvent| {
        let current = value.read().clone();
        let new_text = format!("{}*italic text*", current);
        value.set(new_text);
    };

    let insert_heading = move |_: MouseEvent| {
        let current = value.read().clone();
        let new_text = if current.is_empty() {
            "## Heading\n\n".to_string()
        } else {
            format!("{}\n## Heading\n\n", current)
        };
        value.set(new_text);
    };

    let insert_link = move |_: MouseEvent| {
        let current = value.read().clone();
        let new_text = format!("{}[link text](url)", current);
        value.set(new_text);
    };

    let insert_list = move |_: MouseEvent| {
        let current = value.read().clone();
        let new_text = if current.is_empty() {
            "- Item 1\n- Item 2\n- Item 3\n".to_string()
        } else {
            format!("{}\n- Item 1\n- Item 2\n- Item 3\n", current)
        };
        value.set(new_text);
    };

    let insert_code = move |_: MouseEvent| {
        let current = value.read().clone();
        let new_text = format!("{}```\ncode here\n```\n", current);
        value.set(new_text);
    };

    let insert_quote = move |_: MouseEvent| {
        let current = value.read().clone();
        let new_text = if current.is_empty() {
            "> Quote\n".to_string()
        } else {
            format!("{}\n> Quote\n", current)
        };
        value.set(new_text);
    };

    rsx! {
        div { class: "markdown-editor",
            // Toolbar
            div { class: "md-toolbar",
                div { class: "md-toolbar-group",
                    button {
                        class: "md-btn",
                        onclick: insert_bold,
                        title: "Bold",
                        "B"
                    }
                    button {
                        class: "md-btn",
                        onclick: insert_italic,
                        title: "Italic",
                        "I"
                    }
                    button {
                        class: "md-btn",
                        onclick: insert_heading,
                        title: "Heading",
                        "H"
                    }
                }
                div { class: "md-toolbar-group",
                    button {
                        class: "md-btn",
                        onclick: insert_link,
                        title: "Link",
                        "🔗"
                    }
                    button {
                        class: "md-btn",
                        onclick: insert_list,
                        title: "List",
                        "≡"
                    }
                    button {
                        class: "md-btn",
                        onclick: insert_code,
                        title: "Code Block",
                        "</>"
                    }
                    button {
                        class: "md-btn",
                        onclick: insert_quote,
                        title: "Quote",
                        "\""
                    }
                }
                div { class: "md-toolbar-group md-toolbar-toggle",
                    button {
                        class: if show_preview() { "md-btn md-btn--active" } else { "md-btn" },
                        onclick: move |_| show_preview.toggle(),
                        title: "Toggle Preview",
                        if show_preview() { "👁️" } else { "📝" }
                    }
                }
            }

            // Editor and preview
            div {
                class: "md-content",
                style: "min-height: {min_height}px",

                // Editor pane
                div { class: if show_preview() { "md-pane md-pane--editor" } else { "md-pane md-pane--full" },
                    textarea {
                        class: "md-textarea",
                        value: "{value}",
                        oninput: move |e| value.set(e.value()),
                        placeholder: "{placeholder}",
                        spellcheck: true
                    }
                }

                // Preview pane
                if show_preview() {
                    div { class: "md-pane md-pane--preview",
                        if value.read().is_empty() {
                            div { class: "md-preview-empty",
                                "Preview will appear here..."
                            }
                        } else {
                            {
                                let content = use_memo(move || value());
                                rsx! {
                                    MarkdownRenderer {
                                        content: content,
                                        collapsible: false,
                                        collapsed: false,
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Quick reference
            details { class: "md-help",
                summary { class: "md-help-toggle", "Markdown Reference" }
                div { class: "md-help-content",
                    table { class: "md-ref-table",
                        tbody {
                            tr {
                                td { code { "**bold**" } }
                                td { strong { "bold" } }
                            }
                            tr {
                                td { code { "*italic*" } }
                                td { em { "italic" } }
                            }
                            tr {
                                td { code { "## Heading" } }
                                td { "Heading (level 2)" }
                            }
                            tr {
                                td { code { "[text](url)" } }
                                td { "Link" }
                            }
                            tr {
                                td { code { "- item" } }
                                td { "Bullet list" }
                            }
                            tr {
                                td { code { "> quote" } }
                                td { "Block quote" }
                            }
                            tr {
                                td { code { "```code```" } }
                                td { "Code block" }
                            }
                        }
                    }
                }
            }
        }
    }
}

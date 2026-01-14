//! Golden Rectangle Card System
//!
//! Components implementing the golden ratio (φ = 1.618) design system.

mod golden_card;
mod card_header;
mod card_gallery;
mod profile_card;
mod quest_card;
mod markdown_editor;

pub use golden_card::{CardOrientation, GoldenCard};
pub use card_header::CardHeader;
pub use card_gallery::CardGallery;
pub use profile_card::ProfileCard;
pub use quest_card::QuestCard;
pub use markdown_editor::{MarkdownEditor, MarkdownRenderer};

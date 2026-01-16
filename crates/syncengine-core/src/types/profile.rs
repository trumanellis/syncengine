//! User Profile Type - Rich identity information for peers
//!
//! Stores profile data including display name, avatar, bio, and featured quests.

use serde::{Deserialize, Serialize};

/// User profile with rich identity information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserProfile {
    /// Peer's public key / node ID
    pub peer_id: String,

    /// Display name shown in UI
    pub display_name: String,

    /// Optional subtitle (e.g., role, tagline)
    pub subtitle: Option<String>,

    /// Custom short link (e.g., "alice" → sync.local/alice)
    pub profile_link: Option<String>,

    /// Iroh blob hash for avatar image
    pub avatar_blob_id: Option<String>,

    /// Markdown-formatted biography
    pub bio: String,

    /// Quest IDs featured in profile gallery
    pub top_quests: Vec<String>,

    /// Unix timestamp when profile was created
    pub created_at: i64,

    /// Unix timestamp of last update
    pub updated_at: i64,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            peer_id: String::new(),
            display_name: "Anonymous User".to_string(),
            subtitle: None,
            profile_link: None,
            avatar_blob_id: None,
            bio: String::new(),
            top_quests: vec![],
            created_at: 0,
            updated_at: 0,
        }
    }
}

impl UserProfile {
    /// Create a new profile with just peer ID and display name
    pub fn new(peer_id: String, display_name: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            peer_id,
            display_name,
            created_at: now,
            updated_at: now,
            ..Default::default()
        }
    }

    /// Update the profile's timestamp
    pub fn touch(&mut self) {
        self.updated_at = chrono::Utc::now().timestamp();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_profile() {
        let profile = UserProfile::default();
        assert_eq!(profile.display_name, "Anonymous User");
        assert!(profile.bio.is_empty());
        assert!(profile.avatar_blob_id.is_none());
    }

    #[test]
    fn test_new_profile() {
        let profile = UserProfile::new("test-peer-id".to_string(), "Alice".to_string());
        assert_eq!(profile.peer_id, "test-peer-id");
        assert_eq!(profile.display_name, "Alice");
        assert!(profile.created_at > 0);
        assert_eq!(profile.created_at, profile.updated_at);
    }

    #[test]
    fn test_touch_updates_timestamp() {
        let mut profile = UserProfile::new("test".to_string(), "Test".to_string());
        let original_time = profile.updated_at;

        // Sleep for >1 second since Unix timestamps have 1-second granularity
        std::thread::sleep(std::time::Duration::from_millis(1001));
        profile.touch();

        assert!(profile.updated_at > original_time);
    }
}

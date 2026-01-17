// Blindodon - An accessibility-first Mastodon client
// Copyright (C) 2025 Blindodon Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Timeline model and configuration

use serde::{Deserialize, Serialize};

/// Type of timeline
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TimelineType {
    /// Home timeline (posts from followed accounts)
    Home,
    /// Local timeline (posts from the instance)
    Local,
    /// Federated timeline (posts from all known instances)
    Federated,
    /// Notifications
    Notifications,
    /// Direct messages
    Direct,
    /// Posts from a specific user
    User { user_id: String },
    /// Posts with a specific hashtag
    Hashtag { tag: String },
    /// Posts from a list
    List { list_id: String },
    /// Bookmarked posts
    Bookmarks,
    /// Favorited posts
    Favourites,
    /// Trending posts
    Trending,
    /// Search results
    Search { query: String },
}

impl TimelineType {
    /// Get a display name for this timeline type
    pub fn display_name(&self) -> String {
        match self {
            TimelineType::Home => "Home".to_string(),
            TimelineType::Local => "Local".to_string(),
            TimelineType::Federated => "Federated".to_string(),
            TimelineType::Notifications => "Notifications".to_string(),
            TimelineType::Direct => "Direct Messages".to_string(),
            TimelineType::User { user_id } => format!("User: {}", user_id),
            TimelineType::Hashtag { tag } => format!("#{}", tag),
            TimelineType::List { list_id } => format!("List: {}", list_id),
            TimelineType::Bookmarks => "Bookmarks".to_string(),
            TimelineType::Favourites => "Favourites".to_string(),
            TimelineType::Trending => "Trending".to_string(),
            TimelineType::Search { query } => format!("Search: {}", query),
        }
    }
}

/// Settings for a specific timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineSettings {
    /// Unique identifier for this timeline instance
    pub id: String,

    /// Type of timeline
    pub timeline_type: TimelineType,

    /// Whether auto-refresh is enabled
    pub auto_refresh: bool,

    /// Refresh interval in seconds
    pub refresh_interval_secs: u64,

    /// Number of posts to fetch per refresh
    pub posts_per_fetch: u32,

    /// Whether sound notifications are enabled
    pub sound_enabled: bool,

    /// Whether desktop notifications are enabled
    pub desktop_notifications: bool,

    /// Hide boosts/reblogs
    pub hide_boosts: bool,

    /// Hide replies
    pub hide_replies: bool,

    /// Hide posts with only media
    pub hide_media_only: bool,

    /// Display density
    pub display_density: DisplayDensity,

    /// Remember scroll position
    pub persist_position: bool,
}

impl Default for TimelineSettings {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timeline_type: TimelineType::Home,
            auto_refresh: true,
            refresh_interval_secs: 60,
            posts_per_fetch: 20,
            sound_enabled: true,
            desktop_notifications: false,
            hide_boosts: false,
            hide_replies: false,
            hide_media_only: false,
            display_density: DisplayDensity::Normal,
            persist_position: true,
        }
    }
}

/// Display density for timeline
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DisplayDensity {
    Compact,
    #[default]
    Normal,
    Comfortable,
}

/// Request to fetch a timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineRequest {
    /// Type of timeline to fetch
    pub timeline_type: TimelineType,
    /// Maximum number of posts to return
    pub limit: Option<u32>,
    /// Return posts older than this ID
    pub max_id: Option<String>,
    /// Return posts newer than this ID
    pub since_id: Option<String>,
    /// Return posts immediately newer than this ID
    pub min_id: Option<String>,
}

/// Response containing timeline posts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineResponse {
    /// The posts in the timeline
    pub posts: Vec<super::Post>,
    /// ID of the newest post (for pagination)
    pub max_id: Option<String>,
    /// ID of the oldest post (for pagination)
    pub min_id: Option<String>,
    /// Whether there are more posts available
    pub has_more: bool,
}

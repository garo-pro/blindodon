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

//! Notification model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{Post, User};

/// Type of notification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    /// Someone mentioned you
    Mention,
    /// Someone boosted your post
    Reblog,
    /// Someone favorited your post
    Favourite,
    /// Someone followed you
    Follow,
    /// Someone requested to follow you
    FollowRequest,
    /// A poll you voted in has ended
    Poll,
    /// A post you interacted with was edited
    Update,
    /// Someone signed up (admin only)
    AdminSignUp,
    /// New report (admin only)
    AdminReport,
    /// Severed relationships due to moderation
    SeveredRelationships,
    /// Unknown notification type
    #[serde(other)]
    Unknown,
}

/// A notification from Mastodon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Unique identifier
    pub id: String,

    /// Type of notification
    #[serde(rename = "type")]
    pub notification_type: NotificationType,

    /// When this notification was created
    pub created_at: DateTime<Utc>,

    /// The account that triggered the notification
    pub account: User,

    /// The post associated with the notification (if any)
    pub status: Option<Post>,

    /// Whether this notification has been read
    #[serde(default)]
    pub read: bool,
}

/// Notification filter settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotificationFilter {
    /// Exclude mentions
    pub exclude_mentions: bool,
    /// Exclude boosts
    pub exclude_reblogs: bool,
    /// Exclude favorites
    pub exclude_favourites: bool,
    /// Exclude follows
    pub exclude_follows: bool,
    /// Exclude follow requests
    pub exclude_follow_requests: bool,
    /// Exclude poll notifications
    pub exclude_polls: bool,
    /// Exclude update notifications
    pub exclude_updates: bool,
}

/// Request for fetching notifications
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotificationRequest {
    /// Return results older than this ID
    pub max_id: Option<String>,
    /// Return results newer than this ID
    pub since_id: Option<String>,
    /// Return results immediately newer than this ID
    pub min_id: Option<String>,
    /// Maximum number of results to return (default 20)
    pub limit: Option<u32>,
    /// Only include these notification types
    pub types: Option<Vec<NotificationType>>,
    /// Exclude these notification types
    pub exclude_types: Option<Vec<NotificationType>>,
}

/// Response from fetching notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResponse {
    /// List of notifications
    pub notifications: Vec<Notification>,
    /// ID of the oldest notification (for pagination)
    pub max_id: Option<String>,
    /// ID of the newest notification (for pagination)
    pub min_id: Option<String>,
    /// Whether there are more notifications to fetch
    pub has_more: bool,
}

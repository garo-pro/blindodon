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

//! User model representing a Mastodon account

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::CustomEmoji;

/// A Mastodon user/account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique identifier
    pub id: String,

    /// Username (without domain)
    pub username: String,

    /// Full account name (user@domain or user for local)
    pub acct: String,

    /// Display name
    pub display_name: String,

    /// Profile bio (HTML)
    pub note: String,

    /// URL to the user's profile page
    pub url: String,

    /// URL to the user's avatar image
    pub avatar: String,

    /// URL to the static avatar (for GIF avatars)
    pub avatar_static: String,

    /// URL to the user's header image
    pub header: String,

    /// URL to the static header
    pub header_static: String,

    /// Whether the account is locked (requires follow approval)
    pub locked: bool,

    /// Custom fields on the profile
    pub fields: Vec<ProfileField>,

    /// Custom emoji used in display name or bio
    pub emojis: Vec<CustomEmoji>,

    /// Whether this is a bot account
    pub bot: bool,

    /// Whether this is a group account
    pub group: bool,

    /// Whether profile is discoverable
    pub discoverable: Option<bool>,

    /// When the account was created
    pub created_at: DateTime<Utc>,

    /// When the account last posted
    pub last_status_at: Option<String>,

    /// Number of posts
    pub statuses_count: u64,

    /// Number of followers
    pub followers_count: u64,

    /// Number of accounts following
    pub following_count: u64,

    /// Whether the current user is following this account
    pub following: Option<bool>,

    /// Whether this account is following the current user
    pub followed_by: Option<bool>,

    /// Whether the current user is blocking this account
    pub blocking: Option<bool>,

    /// Whether the current user has muted this account
    pub muting: Option<bool>,

    /// Whether the current user has muted notifications
    pub muting_notifications: Option<bool>,

    /// Whether the current user has requested to follow
    pub requested: Option<bool>,

    /// Whether the current user is blocking this account's domain
    pub domain_blocking: Option<bool>,

    /// Whether the current user has endorsed this account
    pub endorsed: Option<bool>,

    /// User's personal note about this account
    pub note_personal: Option<String>,

    /// Whether this user supports Blindodon PM
    #[serde(default)]
    pub blindodon_pm_capable: bool,

    /// Blindodon PM public key (if available)
    pub blindodon_pm_public_key: Option<String>,
}

/// A custom field on a user's profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileField {
    pub name: String,
    pub value: String,
    pub verified_at: Option<DateTime<Utc>>,
}

/// Relationship between the current user and another account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: String,
    pub following: bool,
    pub showing_reblogs: bool,
    pub notifying: bool,
    pub languages: Option<Vec<String>>,
    pub followed_by: bool,
    pub blocking: bool,
    pub blocked_by: bool,
    pub muting: bool,
    pub muting_notifications: bool,
    pub requested: bool,
    pub requested_by: bool,
    pub domain_blocking: bool,
    pub endorsed: bool,
    pub note: String,
}

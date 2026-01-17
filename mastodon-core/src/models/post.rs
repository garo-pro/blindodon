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

//! Post model representing a Mastodon status/toot

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{MediaAttachment, User};

/// Visibility level for a post
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Public,
    Unlisted,
    Private,
    Direct,
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::Public
    }
}

/// A poll attached to a post
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Poll {
    pub id: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub expired: bool,
    pub multiple: bool,
    pub votes_count: u64,
    pub voters_count: Option<u64>,
    pub options: Vec<PollOption>,
    pub voted: Option<bool>,
    pub own_votes: Option<Vec<u32>>,
}

/// A single option in a poll
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollOption {
    pub title: String,
    pub votes_count: Option<u64>,
}

/// Application that posted the status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    pub name: String,
    pub website: Option<String>,
}

/// A Mastodon post/status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    /// Unique identifier for this post
    pub id: String,

    /// URI of the post (ActivityPub)
    pub uri: String,

    /// URL to the post's HTML representation
    pub url: Option<String>,

    /// The account that authored this post
    pub account: User,

    /// HTML content of the post
    pub content: String,

    /// Plain text content (for accessibility)
    pub plain_content: Option<String>,

    /// Content warning text (if any)
    pub spoiler_text: String,

    /// Visibility of this post
    pub visibility: Visibility,

    /// Whether this is a sensitive post
    pub sensitive: bool,

    /// When this post was created
    pub created_at: DateTime<Utc>,

    /// When this post was last edited (if ever)
    pub edited_at: Option<DateTime<Utc>>,

    /// Language of the post (ISO 639-1)
    pub language: Option<String>,

    /// ID of the post this is replying to
    pub in_reply_to_id: Option<String>,

    /// ID of the account this is replying to
    pub in_reply_to_account_id: Option<String>,

    /// Media attachments
    pub media_attachments: Vec<MediaAttachment>,

    /// Hashtags mentioned in the post
    pub tags: Vec<Tag>,

    /// Accounts mentioned in the post
    pub mentions: Vec<Mention>,

    /// Custom emoji used in the post
    pub emojis: Vec<CustomEmoji>,

    /// Number of boosts
    pub reblogs_count: u64,

    /// Number of favorites
    pub favourites_count: u64,

    /// Number of replies
    pub replies_count: u64,

    /// The boosted post (if this is a boost)
    pub reblog: Option<Box<Post>>,

    /// Poll attached to this post
    pub poll: Option<Poll>,

    /// Application used to post this
    pub application: Option<Application>,

    /// Whether the current user has boosted this
    pub reblogged: Option<bool>,

    /// Whether the current user has favorited this
    pub favourited: Option<bool>,

    /// Whether the current user has bookmarked this
    pub bookmarked: Option<bool>,

    /// Whether the current user has muted this conversation
    pub muted: Option<bool>,

    /// Whether this is pinned on the author's profile
    pub pinned: Option<bool>,

    /// Whether this post contains Blindodon PM encrypted content
    #[serde(default)]
    pub blindodon_encrypted: bool,
}

/// A hashtag mentioned in a post
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub url: String,
}

/// An account mentioned in a post
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mention {
    pub id: String,
    pub username: String,
    pub acct: String,
    pub url: String,
}

/// A custom emoji
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomEmoji {
    pub shortcode: String,
    pub url: String,
    pub static_url: String,
    pub visible_in_picker: bool,
    pub category: Option<String>,
}

/// Request to create a new post
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPost {
    pub content: String,
    pub spoiler_text: Option<String>,
    pub visibility: Visibility,
    pub sensitive: bool,
    pub language: Option<String>,
    pub in_reply_to_id: Option<String>,
    pub media_ids: Vec<String>,
    pub poll: Option<NewPoll>,
    pub scheduled_at: Option<DateTime<Utc>>,
    /// Enable Blindodon PM encryption for this post
    #[serde(default)]
    pub blindodon_pm: bool,
}

/// Request to create a poll
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPoll {
    pub options: Vec<String>,
    pub expires_in: u64,
    pub multiple: bool,
    pub hide_totals: bool,
}

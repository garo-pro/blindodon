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

//! Account model for managing Mastodon accounts

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A stored Mastodon account (for multi-account support)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredAccount {
    /// Unique identifier (local)
    pub id: String,

    /// Instance URL (e.g., "https://mastodon.social")
    pub instance_url: String,

    /// Username
    pub username: String,

    /// Full account name (user@domain)
    pub acct: String,

    /// Display name
    pub display_name: String,

    /// OAuth access token
    #[serde(skip_serializing, default)]
    pub access_token: String,

    /// OAuth refresh token (if available)
    #[serde(skip_serializing, default)]
    pub refresh_token: Option<String>,

    /// When the token expires
    pub token_expires_at: Option<DateTime<Utc>>,

    /// When this account was added
    pub added_at: DateTime<Utc>,

    /// When this account was last used
    pub last_used_at: DateTime<Utc>,

    /// Whether this is the default/active account
    pub is_default: bool,

    /// Avatar URL
    pub avatar_url: Option<String>,

    /// Blindodon PM private key (encrypted)
    #[serde(skip_serializing, default)]
    pub blindodon_pm_private_key: Option<String>,

    /// Blindodon PM public key
    pub blindodon_pm_public_key: Option<String>,
}

/// OAuth application registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthApp {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub instance_url: String,
}

/// OAuth authorization request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    /// Instance URL
    pub instance_url: String,
}

/// OAuth authorization response with auth URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    /// URL to open in browser for authorization
    pub auth_url: String,
    /// State parameter for verification
    pub state: String,
}

/// OAuth callback with authorization code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCallback {
    /// Authorization code from callback
    pub code: String,
    /// State parameter for verification
    pub state: String,
}

/// Result of successful authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    /// The stored account
    pub account: StoredAccount,
    /// Success message
    pub message: String,
}

/// Instance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    /// Instance URL
    pub url: String,
    /// Instance title
    pub title: String,
    /// Short description
    pub short_description: Option<String>,
    /// Full description
    pub description: String,
    /// Instance version
    pub version: String,
    /// User count
    pub user_count: Option<u64>,
    /// Status count
    pub status_count: Option<u64>,
    /// Domain count
    pub domain_count: Option<u64>,
    /// Thumbnail URL
    pub thumbnail: Option<String>,
    /// Maximum post length
    pub max_toot_chars: Option<u32>,
    /// Maximum media attachments
    pub max_media_attachments: Option<u32>,
    /// Supported languages
    pub languages: Vec<String>,
    /// Whether registration is open
    pub registrations: bool,
    /// Whether approval is required
    pub approval_required: bool,
}

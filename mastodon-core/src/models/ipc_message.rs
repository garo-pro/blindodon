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

//! IPC message models for communication between Rust and C#

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Type of IPC message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Request,
    Response,
    Event,
}

/// An IPC message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage {
    /// Unique message ID (UUID)
    pub id: String,

    /// Message type
    #[serde(rename = "type")]
    pub message_type: MessageType,

    /// Method name for requests
    pub method: Option<String>,

    /// Parameters for requests
    pub params: Option<Value>,

    /// Result for responses
    pub result: Option<Value>,

    /// Error for failed responses
    pub error: Option<IpcError>,
}

impl IpcMessage {
    /// Create a new request message
    pub fn request(method: &str, params: Option<Value>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            message_type: MessageType::Request,
            method: Some(method.to_string()),
            params,
            result: None,
            error: None,
        }
    }

    /// Create a success response
    pub fn response_ok(id: &str, result: Value) -> Self {
        Self {
            id: id.to_string(),
            message_type: MessageType::Response,
            method: None,
            params: None,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response
    pub fn response_err(id: &str, error: IpcError) -> Self {
        Self {
            id: id.to_string(),
            message_type: MessageType::Response,
            method: None,
            params: None,
            result: None,
            error: Some(error),
        }
    }

    /// Create an event message
    pub fn event(method: &str, params: Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            message_type: MessageType::Event,
            method: Some(method.to_string()),
            params: Some(params),
            result: None,
            error: None,
        }
    }
}

/// Error in an IPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Additional error data
    pub data: Option<Value>,
}

impl IpcError {
    /// Create a new error
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Add data to the error
    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }
}

/// Standard error codes
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;

    // Application-specific errors
    pub const NOT_AUTHENTICATED: i32 = -1001;
    pub const RATE_LIMITED: i32 = -1002;
    pub const NETWORK_ERROR: i32 = -1003;
    pub const API_ERROR: i32 = -1004;
    pub const ENCRYPTION_ERROR: i32 = -1005;
}

/// IPC method names
pub mod methods {
    // Authentication
    pub const AUTH_START: &str = "auth.start";
    pub const AUTH_CALLBACK: &str = "auth.callback";
    pub const AUTH_LOGOUT: &str = "auth.logout";
    pub const AUTH_GET_ACCOUNTS: &str = "auth.get_accounts";
    pub const AUTH_SET_DEFAULT: &str = "auth.set_default";

    // Timeline
    pub const TIMELINE_GET: &str = "timeline.get";
    pub const TIMELINE_STREAM_START: &str = "timeline.stream.start";
    pub const TIMELINE_STREAM_STOP: &str = "timeline.stream.stop";

    // Posts
    pub const POST_CREATE: &str = "post.create";
    pub const POST_DELETE: &str = "post.delete";
    pub const POST_EDIT: &str = "post.edit";
    pub const POST_BOOST: &str = "post.boost";
    pub const POST_UNBOOST: &str = "post.unboost";
    pub const POST_FAVOURITE: &str = "post.favourite";
    pub const POST_UNFAVOURITE: &str = "post.unfavourite";
    pub const POST_BOOKMARK: &str = "post.bookmark";
    pub const POST_UNBOOKMARK: &str = "post.unbookmark";
    pub const POST_GET_CONTEXT: &str = "post.get_context";

    // Users
    pub const USER_GET: &str = "user.get";
    pub const USER_FOLLOW: &str = "user.follow";
    pub const USER_UNFOLLOW: &str = "user.unfollow";
    pub const USER_BLOCK: &str = "user.block";
    pub const USER_UNBLOCK: &str = "user.unblock";
    pub const USER_MUTE: &str = "user.mute";
    pub const USER_UNMUTE: &str = "user.unmute";

    // Notifications
    pub const NOTIFICATIONS_GET: &str = "notifications.get";
    pub const NOTIFICATIONS_CLEAR: &str = "notifications.clear";
    pub const NOTIFICATIONS_DISMISS: &str = "notifications.dismiss";

    // Search
    pub const SEARCH: &str = "search";

    // Media
    pub const MEDIA_UPLOAD: &str = "media.upload";

    // Instance
    pub const INSTANCE_GET: &str = "instance.get";

    // System
    pub const PING: &str = "ping";
    pub const SHUTDOWN: &str = "shutdown";
}

/// Event names for streaming
pub mod events {
    pub const NEW_POST: &str = "event.new_post";
    pub const POST_UPDATED: &str = "event.post_updated";
    pub const POST_DELETED: &str = "event.post_deleted";
    pub const NEW_NOTIFICATION: &str = "event.new_notification";
    pub const STREAM_CONNECTED: &str = "event.stream_connected";
    pub const STREAM_DISCONNECTED: &str = "event.stream_disconnected";
    pub const RATE_LIMIT_WARNING: &str = "event.rate_limit_warning";
    pub const ERROR: &str = "event.error";
}

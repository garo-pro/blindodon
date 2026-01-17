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

//! IPC message handler

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::api::MastodonClient;
use crate::models::{
    error_codes, methods,
    IpcError, IpcMessage, MediaUploadRequest, NotificationRequest, TimelineRequest, TimelineType,
};
use crate::log_ipc;

/// Handles incoming IPC messages and routes them to appropriate handlers
pub struct MessageHandler {
    /// Active Mastodon client (if authenticated)
    client: RwLock<Option<Arc<MastodonClient>>>,
}

impl MessageHandler {
    /// Create a new message handler
    pub fn new() -> Self {
        Self {
            client: RwLock::new(None),
        }
    }

    /// Handle an incoming IPC message
    pub async fn handle_message(&self, msg: IpcMessage) -> IpcMessage {
        let method = msg.method.as_deref().unwrap_or("unknown");
        log_ipc!(request, method, &msg.id);

        let result = match method {
            // System methods
            methods::PING => self.handle_ping(&msg).await,
            methods::SHUTDOWN => self.handle_shutdown(&msg).await,

            // Authentication methods
            methods::AUTH_START => self.handle_auth_start(&msg).await,
            methods::AUTH_CALLBACK => self.handle_auth_callback(&msg).await,
            methods::AUTH_LOGOUT => self.handle_auth_logout(&msg).await,
            methods::AUTH_GET_ACCOUNTS => self.handle_auth_get_accounts(&msg).await,

            // Timeline methods
            methods::TIMELINE_GET => self.handle_timeline_get(&msg).await,

            // Post methods
            methods::POST_CREATE => self.handle_post_create(&msg).await,
            methods::POST_BOOST => self.handle_post_boost(&msg).await,
            methods::POST_UNBOOST => self.handle_post_unboost(&msg).await,
            methods::POST_FAVOURITE => self.handle_post_favourite(&msg).await,
            methods::POST_UNFAVOURITE => self.handle_post_unfavourite(&msg).await,

            // Notification methods
            methods::NOTIFICATIONS_GET => self.handle_notifications_get(&msg).await,
            methods::NOTIFICATIONS_CLEAR => self.handle_notifications_clear(&msg).await,
            methods::NOTIFICATIONS_DISMISS => self.handle_notifications_dismiss(&msg).await,

            // Media methods
            methods::MEDIA_UPLOAD => self.handle_media_upload(&msg).await,

            // Instance methods
            methods::INSTANCE_GET => self.handle_instance_get(&msg).await,

            // Unknown method
            _ => {
                warn!("Unknown method: {}", method);
                IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::METHOD_NOT_FOUND, format!("Unknown method: {}", method)),
                )
            }
        };

        let success = result.error.is_none();
        log_ipc!(response, method, &msg.id, success);

        result
    }

    /// Handle ping request
    async fn handle_ping(&self, msg: &IpcMessage) -> IpcMessage {
        IpcMessage::response_ok(&msg.id, serde_json::json!({
            "pong": true,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    /// Handle shutdown request
    async fn handle_shutdown(&self, msg: &IpcMessage) -> IpcMessage {
        info!("Shutdown requested via IPC");
        // In a real implementation, we'd signal the main loop to shutdown
        IpcMessage::response_ok(&msg.id, serde_json::json!({
            "status": "shutting_down"
        }))
    }

    /// Handle auth start request
    async fn handle_auth_start(&self, msg: &IpcMessage) -> IpcMessage {
        let params = match &msg.params {
            Some(p) => p,
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::INVALID_PARAMS, "Missing params"),
                );
            }
        };

        let instance_url = match params.get("instance_url").and_then(|v| v.as_str()) {
            Some(url) => url,
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::INVALID_PARAMS, "Missing instance_url"),
                );
            }
        };

        info!("Starting auth flow for instance: {}", instance_url);

        match MastodonClient::start_auth(instance_url).await {
            Ok(auth_response) => {
                IpcMessage::response_ok(&msg.id, serde_json::to_value(auth_response).unwrap())
            }
            Err(e) => {
                error!("Auth start failed: {}", e);
                IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::API_ERROR, format!("Auth failed: {}", e)),
                )
            }
        }
    }

    /// Handle auth callback
    async fn handle_auth_callback(&self, msg: &IpcMessage) -> IpcMessage {
        let params = match &msg.params {
            Some(p) => p,
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::INVALID_PARAMS, "Missing params"),
                );
            }
        };

        let code = match params.get("code").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::INVALID_PARAMS, "Missing code"),
                );
            }
        };

        let instance_url = match params.get("instance_url").and_then(|v| v.as_str()) {
            Some(url) => url,
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::INVALID_PARAMS, "Missing instance_url"),
                );
            }
        };

        info!("Processing auth callback for instance: {}", instance_url);

        match MastodonClient::complete_auth(instance_url, code).await {
            Ok(client) => {
                let account_info = client.get_current_user().await;
                let client = Arc::new(client);
                *self.client.write().await = Some(client);

                match account_info {
                    Ok(user) => {
                        IpcMessage::response_ok(&msg.id, serde_json::json!({
                            "success": true,
                            "account": user
                        }))
                    }
                    Err(e) => {
                        IpcMessage::response_ok(&msg.id, serde_json::json!({
                            "success": true,
                            "error_fetching_user": e.to_string()
                        }))
                    }
                }
            }
            Err(e) => {
                error!("Auth callback failed: {}", e);
                IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::API_ERROR, format!("Auth failed: {}", e)),
                )
            }
        }
    }

    /// Handle auth logout
    async fn handle_auth_logout(&self, msg: &IpcMessage) -> IpcMessage {
        *self.client.write().await = None;
        info!("User logged out");
        IpcMessage::response_ok(&msg.id, serde_json::json!({ "success": true }))
    }

    /// Handle get accounts
    async fn handle_auth_get_accounts(&self, msg: &IpcMessage) -> IpcMessage {
        // In a full implementation, this would return saved accounts from storage
        let has_client = self.client.read().await.is_some();
        IpcMessage::response_ok(&msg.id, serde_json::json!({
            "authenticated": has_client,
            "accounts": []
        }))
    }

    /// Handle timeline get request
    async fn handle_timeline_get(&self, msg: &IpcMessage) -> IpcMessage {
        let client = match self.client.read().await.as_ref() {
            Some(c) => c.clone(),
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::NOT_AUTHENTICATED, "Not authenticated"),
                );
            }
        };

        let params = match &msg.params {
            Some(p) => p,
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::INVALID_PARAMS, "Missing params"),
                );
            }
        };

        let request: TimelineRequest = match serde_json::from_value(params.clone()) {
            Ok(r) => r,
            Err(e) => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::INVALID_PARAMS, format!("Invalid params: {}", e)),
                );
            }
        };

        debug!("Fetching timeline: {:?}", request.timeline_type);

        match client.get_timeline(&request).await {
            Ok(response) => {
                IpcMessage::response_ok(&msg.id, serde_json::to_value(response).unwrap())
            }
            Err(e) => {
                error!("Failed to fetch timeline: {}", e);
                IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::API_ERROR, format!("Failed to fetch timeline: {}", e)),
                )
            }
        }
    }

    /// Handle post create
    async fn handle_post_create(&self, msg: &IpcMessage) -> IpcMessage {
        let client = match self.client.read().await.as_ref() {
            Some(c) => c.clone(),
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::NOT_AUTHENTICATED, "Not authenticated"),
                );
            }
        };

        let params = match &msg.params {
            Some(p) => p,
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::INVALID_PARAMS, "Missing params"),
                );
            }
        };

        let new_post: crate::models::NewPost = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::INVALID_PARAMS, format!("Invalid params: {}", e)),
                );
            }
        };

        match client.create_post(&new_post).await {
            Ok(post) => {
                IpcMessage::response_ok(&msg.id, serde_json::to_value(post).unwrap())
            }
            Err(e) => {
                error!("Failed to create post: {}", e);
                IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::API_ERROR, format!("Failed to create post: {}", e)),
                )
            }
        }
    }

    /// Handle post boost
    async fn handle_post_boost(&self, msg: &IpcMessage) -> IpcMessage {
        self.handle_post_action(msg, "boost").await
    }

    /// Handle post unboost
    async fn handle_post_unboost(&self, msg: &IpcMessage) -> IpcMessage {
        self.handle_post_action(msg, "unboost").await
    }

    /// Handle post favourite
    async fn handle_post_favourite(&self, msg: &IpcMessage) -> IpcMessage {
        self.handle_post_action(msg, "favourite").await
    }

    /// Handle post unfavourite
    async fn handle_post_unfavourite(&self, msg: &IpcMessage) -> IpcMessage {
        self.handle_post_action(msg, "unfavourite").await
    }

    /// Generic post action handler
    async fn handle_post_action(&self, msg: &IpcMessage, action: &str) -> IpcMessage {
        let client = match self.client.read().await.as_ref() {
            Some(c) => c.clone(),
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::NOT_AUTHENTICATED, "Not authenticated"),
                );
            }
        };

        let params = match &msg.params {
            Some(p) => p,
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::INVALID_PARAMS, "Missing params"),
                );
            }
        };

        let post_id = match params.get("post_id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::INVALID_PARAMS, "Missing post_id"),
                );
            }
        };

        let result = match action {
            "boost" => client.boost_post(post_id).await,
            "unboost" => client.unboost_post(post_id).await,
            "favourite" => client.favourite_post(post_id).await,
            "unfavourite" => client.unfavourite_post(post_id).await,
            _ => return IpcMessage::response_err(
                &msg.id,
                IpcError::new(error_codes::INTERNAL_ERROR, "Unknown action"),
            ),
        };

        match result {
            Ok(post) => {
                IpcMessage::response_ok(&msg.id, serde_json::to_value(post).unwrap())
            }
            Err(e) => {
                error!("Failed to {} post: {}", action, e);
                IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::API_ERROR, format!("Failed to {} post: {}", action, e)),
                )
            }
        }
    }

    /// Handle instance get
    async fn handle_instance_get(&self, msg: &IpcMessage) -> IpcMessage {
        let client = match self.client.read().await.as_ref() {
            Some(c) => c.clone(),
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::NOT_AUTHENTICATED, "Not authenticated"),
                );
            }
        };

        match client.get_instance_info().await {
            Ok(info) => {
                IpcMessage::response_ok(&msg.id, serde_json::to_value(info).unwrap())
            }
            Err(e) => {
                error!("Failed to get instance info: {}", e);
                IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::API_ERROR, format!("Failed to get instance info: {}", e)),
                )
            }
        }
    }

    /// Handle notifications get
    async fn handle_notifications_get(&self, msg: &IpcMessage) -> IpcMessage {
        let client = match self.client.read().await.as_ref() {
            Some(c) => c.clone(),
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::NOT_AUTHENTICATED, "Not authenticated"),
                );
            }
        };

        let request: NotificationRequest = match &msg.params {
            Some(p) => match serde_json::from_value(p.clone()) {
                Ok(r) => r,
                Err(e) => {
                    return IpcMessage::response_err(
                        &msg.id,
                        IpcError::new(error_codes::INVALID_PARAMS, format!("Invalid params: {}", e)),
                    );
                }
            },
            None => NotificationRequest::default(),
        };

        debug!("Fetching notifications");

        match client.get_notifications(&request).await {
            Ok(response) => {
                IpcMessage::response_ok(&msg.id, serde_json::to_value(response).unwrap())
            }
            Err(e) => {
                error!("Failed to fetch notifications: {}", e);
                IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::API_ERROR, format!("Failed to fetch notifications: {}", e)),
                )
            }
        }
    }

    /// Handle notifications clear
    async fn handle_notifications_clear(&self, msg: &IpcMessage) -> IpcMessage {
        let client = match self.client.read().await.as_ref() {
            Some(c) => c.clone(),
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::NOT_AUTHENTICATED, "Not authenticated"),
                );
            }
        };

        match client.clear_notifications().await {
            Ok(()) => {
                info!("All notifications cleared");
                IpcMessage::response_ok(&msg.id, serde_json::json!({ "success": true }))
            }
            Err(e) => {
                error!("Failed to clear notifications: {}", e);
                IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::API_ERROR, format!("Failed to clear notifications: {}", e)),
                )
            }
        }
    }

    /// Handle notification dismiss
    async fn handle_notifications_dismiss(&self, msg: &IpcMessage) -> IpcMessage {
        let client = match self.client.read().await.as_ref() {
            Some(c) => c.clone(),
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::NOT_AUTHENTICATED, "Not authenticated"),
                );
            }
        };

        let params = match &msg.params {
            Some(p) => p,
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::INVALID_PARAMS, "Missing params"),
                );
            }
        };

        let notification_id = match params.get("notification_id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::INVALID_PARAMS, "Missing notification_id"),
                );
            }
        };

        match client.dismiss_notification(notification_id).await {
            Ok(()) => {
                debug!("Notification {} dismissed", notification_id);
                IpcMessage::response_ok(&msg.id, serde_json::json!({ "success": true }))
            }
            Err(e) => {
                error!("Failed to dismiss notification: {}", e);
                IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::API_ERROR, format!("Failed to dismiss notification: {}", e)),
                )
            }
        }
    }

    /// Handle media upload
    async fn handle_media_upload(&self, msg: &IpcMessage) -> IpcMessage {
        let client = match self.client.read().await.as_ref() {
            Some(c) => c.clone(),
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::NOT_AUTHENTICATED, "Not authenticated"),
                );
            }
        };

        let request: MediaUploadRequest = match &msg.params {
            Some(p) => match serde_json::from_value(p.clone()) {
                Ok(r) => r,
                Err(e) => {
                    return IpcMessage::response_err(
                        &msg.id,
                        IpcError::new(error_codes::INVALID_PARAMS, format!("Invalid params: {}", e)),
                    );
                }
            },
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::INVALID_PARAMS, "Missing params"),
                );
            }
        };

        debug!("Uploading media from: {}", request.file_path);

        match client.upload_media(&request).await {
            Ok(attachment) => {
                info!("Media uploaded: {}", attachment.id);
                IpcMessage::response_ok(&msg.id, serde_json::to_value(attachment).unwrap())
            }
            Err(e) => {
                error!("Failed to upload media: {}", e);
                IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::API_ERROR, format!("Failed to upload media: {}", e)),
                )
            }
        }
    }
}

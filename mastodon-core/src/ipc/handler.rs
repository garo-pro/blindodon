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
use chrono::Utc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::api::MastodonClient;
use crate::cache::CacheManager;
use crate::models::{
    error_codes, methods,
    IpcError, IpcMessage, MediaUploadRequest, NotificationRequest, StoredAccount,
    TimelineRequest, TimelineType,
};
use crate::log_ipc;

/// Handles incoming IPC messages and routes them to appropriate handlers
pub struct MessageHandler {
    /// Active Mastodon client (if authenticated)
    client: RwLock<Option<Arc<MastodonClient>>>,
    /// Current account ID (if authenticated)
    current_account_id: RwLock<Option<String>>,
    /// Cache manager for persistence
    cache: Arc<CacheManager>,
}

impl MessageHandler {
    /// Create a new message handler with cache
    pub fn new(cache: Arc<CacheManager>) -> Self {
        Self {
            client: RwLock::new(None),
            current_account_id: RwLock::new(None),
            cache,
        }
    }

    /// Initialize handler and restore saved session
    pub async fn initialize(&self) -> anyhow::Result<()> {
        // Try to restore the default account
        if let Some(account) = self.cache.get_default_account().await? {
            info!("Restoring session for {}", account.acct);

            match MastodonClient::from_token(&account.instance_url, &account.access_token) {
                Ok(client) => {
                    // Verify the token is still valid
                    match client.get_current_user().await {
                        Ok(user) => {
                            info!("Session restored for {}", user.acct);
                            *self.client.write().await = Some(Arc::new(client));
                            *self.current_account_id.write().await = Some(account.id.clone());

                            // Update last used time
                            if let Err(e) = self.cache.set_default_account(&account.id).await {
                                warn!("Failed to update last used time: {}", e);
                            }
                        }
                        Err(e) => {
                            warn!(
                                "Saved token invalid, will require re-authentication: {}",
                                e
                            );
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to create client from saved token: {}", e);
                }
            }
        }

        Ok(())
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
            methods::AUTH_SWITCH_ACCOUNT => self.handle_auth_switch_account(&msg).await,
            methods::AUTH_DELETE_ACCOUNT => self.handle_auth_delete_account(&msg).await,

            // Settings methods
            methods::SETTINGS_GET => self.handle_settings_get(&msg).await,
            methods::SETTINGS_SET => self.handle_settings_set(&msg).await,
            methods::SETTINGS_GET_ALL => self.handle_settings_get_all(&msg).await,

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
                match client.get_current_user().await {
                    Ok(user) => {
                        // Create account ID from user@instance
                        let instance_domain = instance_url
                            .replace("https://", "")
                            .replace("http://", "");
                        let account_id = format!("{}@{}", user.username, instance_domain);

                        // Create StoredAccount for persistence
                        let stored_account = StoredAccount {
                            id: account_id.clone(),
                            instance_url: client.instance_url().to_string(),
                            username: user.username.clone(),
                            acct: user.acct.clone(),
                            display_name: user.display_name.clone(),
                            access_token: client.access_token().to_string(),
                            refresh_token: None,
                            token_expires_at: None,
                            added_at: Utc::now(),
                            last_used_at: Utc::now(),
                            is_default: true,
                            avatar_url: Some(user.avatar.clone()),
                            blindodon_pm_private_key: None,
                            blindodon_pm_public_key: None,
                        };

                        // Save to database
                        if let Err(e) = self.cache.save_account(&stored_account).await {
                            error!("Failed to save account: {}", e);
                            // Continue anyway - auth succeeded
                        }

                        // Set as default
                        if let Err(e) = self.cache.set_default_account(&account_id).await {
                            error!("Failed to set default account: {}", e);
                        }

                        // Store client in memory
                        let client = Arc::new(client);
                        *self.client.write().await = Some(client);
                        *self.current_account_id.write().await = Some(account_id.clone());

                        // Return account in the format expected by the UI
                        IpcMessage::response_ok(&msg.id, serde_json::json!({
                            "success": true,
                            "account": {
                                "id": account_id,
                                "instance_url": stored_account.instance_url,
                                "username": stored_account.username,
                                "display_name": stored_account.display_name,
                                "avatar_url": stored_account.avatar_url,
                                "is_default": stored_account.is_default,
                                "last_used_at": stored_account.last_used_at
                            }
                        }))
                    }
                    Err(e) => {
                        // Auth succeeded but couldn't fetch user info - still save what we can
                        let client = Arc::new(client);
                        *self.client.write().await = Some(client);

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
        let account_id = self.current_account_id.read().await.clone();

        *self.client.write().await = None;
        *self.current_account_id.write().await = None;

        // Optionally delete the account from storage if requested
        let delete_account = msg
            .params
            .as_ref()
            .and_then(|p| p.get("delete_account"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if delete_account {
            if let Some(id) = &account_id {
                if let Err(e) = self.cache.delete_account(id).await {
                    error!("Failed to delete account: {}", e);
                }
            }
        }

        info!("User logged out");
        IpcMessage::response_ok(&msg.id, serde_json::json!({ "success": true }))
    }

    /// Handle get accounts - returns saved accounts from storage
    async fn handle_auth_get_accounts(&self, msg: &IpcMessage) -> IpcMessage {
        let has_client = self.client.read().await.is_some();
        let current_account_id = self.current_account_id.read().await.clone();

        let accounts = match self.cache.get_accounts().await {
            Ok(accounts) => accounts,
            Err(e) => {
                error!("Failed to get accounts: {}", e);
                vec![]
            }
        };

        IpcMessage::response_ok(
            &msg.id,
            serde_json::json!({
                "authenticated": has_client,
                "current_account_id": current_account_id,
                "accounts": accounts
            }),
        )
    }

    /// Handle switch account
    async fn handle_auth_switch_account(&self, msg: &IpcMessage) -> IpcMessage {
        let params = match &msg.params {
            Some(p) => p,
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::INVALID_PARAMS, "Missing params"),
                );
            }
        };

        let account_id = match params.get("account_id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::INVALID_PARAMS, "Missing account_id"),
                );
            }
        };

        let account = match self.cache.get_account(account_id).await {
            Ok(Some(acc)) => acc,
            Ok(None) => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::NOT_AUTHENTICATED, "Account not found"),
                );
            }
            Err(e) => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(
                        error_codes::INTERNAL_ERROR,
                        format!("Database error: {}", e),
                    ),
                );
            }
        };

        // Create client from saved token
        match MastodonClient::from_token(&account.instance_url, &account.access_token) {
            Ok(client) => {
                // Verify token is still valid
                match client.get_current_user().await {
                    Ok(user) => {
                        *self.client.write().await = Some(Arc::new(client));
                        *self.current_account_id.write().await = Some(account_id.to_string());

                        // Update default and last_used
                        let _ = self.cache.set_default_account(account_id).await;

                        info!("Switched to account {}", account_id);
                        IpcMessage::response_ok(
                            &msg.id,
                            serde_json::json!({
                                "success": true,
                                "account": account,
                                "user": user
                            }),
                        )
                    }
                    Err(e) => IpcMessage::response_err(
                        &msg.id,
                        IpcError::new(
                            error_codes::API_ERROR,
                            format!("Token expired, please re-authenticate: {}", e),
                        ),
                    ),
                }
            }
            Err(e) => IpcMessage::response_err(
                &msg.id,
                IpcError::new(
                    error_codes::INTERNAL_ERROR,
                    format!("Failed to create client: {}", e),
                ),
            ),
        }
    }

    /// Handle delete account
    async fn handle_auth_delete_account(&self, msg: &IpcMessage) -> IpcMessage {
        let params = match &msg.params {
            Some(p) => p,
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::INVALID_PARAMS, "Missing params"),
                );
            }
        };

        let account_id = match params.get("account_id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::INVALID_PARAMS, "Missing account_id"),
                );
            }
        };

        // If this is the current account, log out first
        let current_id = self.current_account_id.read().await.clone();
        if current_id.as_deref() == Some(account_id) {
            *self.client.write().await = None;
            *self.current_account_id.write().await = None;
        }

        match self.cache.delete_account(account_id).await {
            Ok(()) => {
                info!("Deleted account {}", account_id);
                IpcMessage::response_ok(&msg.id, serde_json::json!({ "success": true }))
            }
            Err(e) => IpcMessage::response_err(
                &msg.id,
                IpcError::new(
                    error_codes::INTERNAL_ERROR,
                    format!("Failed to delete account: {}", e),
                ),
            ),
        }
    }

    // ===== SETTINGS HANDLERS =====

    /// Handle settings get
    async fn handle_settings_get(&self, msg: &IpcMessage) -> IpcMessage {
        let params = match &msg.params {
            Some(p) => p,
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::INVALID_PARAMS, "Missing params"),
                );
            }
        };

        let key = match params.get("key").and_then(|v| v.as_str()) {
            Some(k) => k,
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::INVALID_PARAMS, "Missing key"),
                );
            }
        };

        match self.cache.get_setting(key).await {
            Ok(value) => IpcMessage::response_ok(
                &msg.id,
                serde_json::json!({
                    "key": key,
                    "value": value
                }),
            ),
            Err(e) => IpcMessage::response_err(
                &msg.id,
                IpcError::new(
                    error_codes::INTERNAL_ERROR,
                    format!("Database error: {}", e),
                ),
            ),
        }
    }

    /// Handle settings set
    async fn handle_settings_set(&self, msg: &IpcMessage) -> IpcMessage {
        let params = match &msg.params {
            Some(p) => p,
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::INVALID_PARAMS, "Missing params"),
                );
            }
        };

        let key = match params.get("key").and_then(|v| v.as_str()) {
            Some(k) => k,
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::INVALID_PARAMS, "Missing key"),
                );
            }
        };

        let value = match params.get("value").and_then(|v| v.as_str()) {
            Some(v) => v,
            None => {
                return IpcMessage::response_err(
                    &msg.id,
                    IpcError::new(error_codes::INVALID_PARAMS, "Missing value"),
                );
            }
        };

        match self.cache.set_setting(key, value).await {
            Ok(()) => IpcMessage::response_ok(&msg.id, serde_json::json!({ "success": true })),
            Err(e) => IpcMessage::response_err(
                &msg.id,
                IpcError::new(
                    error_codes::INTERNAL_ERROR,
                    format!("Database error: {}", e),
                ),
            ),
        }
    }

    /// Handle settings get all
    async fn handle_settings_get_all(&self, msg: &IpcMessage) -> IpcMessage {
        match self.cache.get_all_settings().await {
            Ok(settings) => IpcMessage::response_ok(
                &msg.id,
                serde_json::json!({
                    "settings": settings
                }),
            ),
            Err(e) => IpcMessage::response_err(
                &msg.id,
                IpcError::new(
                    error_codes::INTERNAL_ERROR,
                    format!("Database error: {}", e),
                ),
            ),
        }
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

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

//! Mastodon API client implementation

use anyhow::{Context, Result};
use megalodon::{
    self,
    generator,
    megalodon::GetHomeTimelineInputOptions,
    megalodon::GetLocalTimelineInputOptions,
    megalodon::GetPublicTimelineInputOptions,
    megalodon::PostStatusInputOptions,
    response::Response,
    Megalodon,
    SNS,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::models::{
    AuthResponse, InstanceInfo, MediaAttachment, MediaUploadRequest, NewPost, Notification,
    NotificationRequest, NotificationResponse, Post, TimelineRequest, TimelineResponse,
    TimelineType, User, Visibility,
};

use super::converter;

/// Application name for OAuth
const APP_NAME: &str = "Blindodon";
/// Scopes required for the application
const SCOPES: &[&str] = &["read", "write", "follow", "push"];
/// Redirect URI for OAuth callback
const REDIRECT_URI: &str = "urn:ietf:wg:oauth:2.0:oob";

/// Stored OAuth application details
struct OAuthAppData {
    client_id: String,
    client_secret: String,
    instance_url: String,
}

/// Thread-safe storage for pending OAuth apps
static PENDING_OAUTH: RwLock<Option<OAuthAppData>> = RwLock::const_new(None);

/// Mastodon API client
pub struct MastodonClient {
    client: Arc<Box<dyn Megalodon + Send + Sync>>,
    instance_url: String,
    access_token: String,
}

impl MastodonClient {
    /// Start the OAuth authentication flow
    pub async fn start_auth(instance_url: &str) -> Result<AuthResponse> {
        info!("Starting OAuth flow for {}", instance_url);

        // Normalize the instance URL
        let instance_url = normalize_url(instance_url);

        // Create a client to register the app
        let client = generator(
            SNS::Mastodon,
            instance_url.clone(),
            None,
            None,
        )?;

        // Register the application
        let app_data = client
            .register_app(
                APP_NAME.to_string(),
                &megalodon::megalodon::AppInputOptions {
                    redirect_uris: Some(REDIRECT_URI.to_string()),
                    scopes: Some(SCOPES.iter().map(|s| s.to_string()).collect()),
                    website: Some("https://github.com/blindodon/blindodon".to_string()),
                },
            )
            .await
            .context("Failed to register application")?;

        let client_id = app_data.client_id.clone();
        let client_secret = app_data.client_secret.clone();

        // Store the app data for the callback
        *PENDING_OAUTH.write().await = Some(OAuthAppData {
            client_id: client_id.clone(),
            client_secret: client_secret.clone(),
            instance_url: instance_url.clone(),
        });

        // Generate the authorization URL
        let auth_url = format!(
            "{}/oauth/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}",
            instance_url,
            client_id,
            urlencoding::encode(REDIRECT_URI),
            SCOPES.join("+")
        );

        info!("Authorization URL generated");

        Ok(AuthResponse {
            auth_url,
            state: uuid::Uuid::new_v4().to_string(),
        })
    }

    /// Complete the OAuth authentication flow
    pub async fn complete_auth(instance_url: &str, code: &str) -> Result<Self> {
        info!("Completing OAuth flow");

        let instance_url = normalize_url(instance_url);

        // Get the stored app data
        let app_data = PENDING_OAUTH.read().await;
        let app_data = app_data
            .as_ref()
            .context("No pending OAuth flow")?;

        if app_data.instance_url != instance_url {
            anyhow::bail!("Instance URL mismatch");
        }

        // Create a client to exchange the code
        let client = generator(
            SNS::Mastodon,
            instance_url.clone(),
            None,
            None,
        )?;

        // Exchange the code for a token
        let token_data = client
            .fetch_access_token(
                app_data.client_id.clone(),
                app_data.client_secret.clone(),
                code.to_string(),
                REDIRECT_URI.to_string(),
            )
            .await
            .context("Failed to fetch access token")?;

        let access_token = token_data.access_token.clone();

        info!("Access token obtained successfully");

        // Create the authenticated client
        let auth_client = generator(
            SNS::Mastodon,
            instance_url.clone(),
            Some(access_token.clone()),
            None,
        )?;

        Ok(Self {
            client: Arc::new(auth_client),
            instance_url,
            access_token,
        })
    }

    /// Create a client from an existing access token
    pub fn from_token(instance_url: &str, access_token: &str) -> Result<Self> {
        let instance_url = normalize_url(instance_url);

        let client = generator(
            SNS::Mastodon,
            instance_url.clone(),
            Some(access_token.to_string()),
            None,
        )?;

        Ok(Self {
            client: Arc::new(client),
            instance_url,
            access_token: access_token.to_string(),
        })
    }

    /// Get the access token (for persistence)
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    /// Get the instance URL
    pub fn instance_url(&self) -> &str {
        &self.instance_url
    }

    /// Get the current authenticated user
    pub async fn get_current_user(&self) -> Result<User> {
        let response = self.client
            .verify_account_credentials()
            .await
            .context("Failed to verify credentials")?;

        Ok(converter::convert_account(&response.json))
    }

    /// Get a timeline
    pub async fn get_timeline(&self, request: &TimelineRequest) -> Result<TimelineResponse> {
        let limit = request.limit.unwrap_or(20);

        let posts = match &request.timeline_type {
            TimelineType::Home => {
                let options = GetHomeTimelineInputOptions {
                    max_id: request.max_id.clone(),
                    since_id: request.since_id.clone(),
                    min_id: request.min_id.clone(),
                    limit: Some(limit),
                    ..Default::default()
                };
                let response = self.client.get_home_timeline(Some(&options)).await?;
                response.json.into_iter().map(|s| converter::convert_status(&s)).collect()
            }
            TimelineType::Local => {
                let options = GetLocalTimelineInputOptions {
                    max_id: request.max_id.clone(),
                    since_id: request.since_id.clone(),
                    min_id: request.min_id.clone(),
                    limit: Some(limit),
                    ..Default::default()
                };
                let response = self.client.get_local_timeline(Some(&options)).await?;
                response.json.into_iter().map(|s| converter::convert_status(&s)).collect()
            }
            TimelineType::Federated => {
                let options = GetPublicTimelineInputOptions {
                    max_id: request.max_id.clone(),
                    since_id: request.since_id.clone(),
                    min_id: request.min_id.clone(),
                    limit: Some(limit),
                    ..Default::default()
                };
                let response = self.client.get_public_timeline(Some(&options)).await?;
                response.json.into_iter().map(|s| converter::convert_status(&s)).collect()
            }
            TimelineType::Notifications => {
                // For notifications, we return an empty list for now
                // This should be handled separately
                vec![]
            }
            TimelineType::Hashtag { tag } => {
                let options = megalodon::megalodon::GetTagTimelineInputOptions {
                    max_id: request.max_id.clone(),
                    since_id: request.since_id.clone(),
                    min_id: request.min_id.clone(),
                    limit: Some(limit),
                    ..Default::default()
                };
                let response = self.client.get_tag_timeline(tag.clone(), Some(&options)).await?;
                response.json.into_iter().map(|s| converter::convert_status(&s)).collect()
            }
            TimelineType::User { user_id } => {
                let options = megalodon::megalodon::GetAccountStatusesInputOptions {
                    limit: Some(limit),
                    ..Default::default()
                };
                let response = self.client.get_account_statuses(user_id.clone(), Some(&options)).await?;
                response.json.into_iter().map(|s| converter::convert_status(&s)).collect()
            }
            TimelineType::Bookmarks => {
                let response = self.client.get_bookmarks(None).await?;
                response.json.into_iter().map(|s| converter::convert_status(&s)).collect()
            }
            TimelineType::Favourites => {
                let response = self.client.get_favourites(None).await?;
                response.json.into_iter().map(|s| converter::convert_status(&s)).collect()
            }
            TimelineType::List { list_id } => {
                let options = megalodon::megalodon::GetListTimelineInputOptions {
                    max_id: request.max_id.clone(),
                    since_id: request.since_id.clone(),
                    min_id: request.min_id.clone(),
                    limit: Some(limit),
                };
                let response = self.client.get_list_timeline(list_id.clone(), Some(&options)).await?;
                response.json.into_iter().map(|s| converter::convert_status(&s)).collect()
            }
            TimelineType::Direct => {
                // Direct messages timeline - skip for now as API changed significantly
                vec![]
            }
            _ => {
                warn!("Unsupported timeline type: {:?}", request.timeline_type);
                vec![]
            }
        };

        let max_id = posts.first().map(|p| p.id.clone());
        let min_id = posts.last().map(|p| p.id.clone());
        let has_more = posts.len() == limit as usize;

        Ok(TimelineResponse {
            posts,
            max_id,
            min_id,
            has_more,
        })
    }

    /// Create a new post
    pub async fn create_post(&self, new_post: &NewPost) -> Result<Post> {
        let visibility = match new_post.visibility {
            Visibility::Public => megalodon::entities::StatusVisibility::Public,
            Visibility::Unlisted => megalodon::entities::StatusVisibility::Unlisted,
            Visibility::Private => megalodon::entities::StatusVisibility::Private,
            Visibility::Direct => megalodon::entities::StatusVisibility::Direct,
        };

        let options = PostStatusInputOptions {
            in_reply_to_id: new_post.in_reply_to_id.clone(),
            sensitive: Some(new_post.sensitive),
            spoiler_text: new_post.spoiler_text.clone(),
            visibility: Some(visibility),
            language: new_post.language.clone(),
            media_ids: if new_post.media_ids.is_empty() {
                None
            } else {
                Some(new_post.media_ids.clone())
            },
            scheduled_at: new_post.scheduled_at,
            ..Default::default()
        };

        let response = self.client
            .post_status(new_post.content.clone(), Some(&options))
            .await
            .context("Failed to create post")?;

        // PostStatusOutput contains a Status field
        match &response.json {
            megalodon::megalodon::PostStatusOutput::Status(status) => {
                Ok(converter::convert_status(status))
            }
            megalodon::megalodon::PostStatusOutput::ScheduledStatus(_) => {
                anyhow::bail!("Scheduled status not supported")
            }
        }
    }

    /// Boost a post
    pub async fn boost_post(&self, post_id: &str) -> Result<Post> {
        let response = self.client
            .reblog_status(post_id.to_string())
            .await
            .context("Failed to boost post")?;

        Ok(converter::convert_status(&response.json))
    }

    /// Unboost a post
    pub async fn unboost_post(&self, post_id: &str) -> Result<Post> {
        let response = self.client
            .unreblog_status(post_id.to_string())
            .await
            .context("Failed to unboost post")?;

        Ok(converter::convert_status(&response.json))
    }

    /// Favourite a post
    pub async fn favourite_post(&self, post_id: &str) -> Result<Post> {
        let response = self.client
            .favourite_status(post_id.to_string())
            .await
            .context("Failed to favourite post")?;

        Ok(converter::convert_status(&response.json))
    }

    /// Unfavourite a post
    pub async fn unfavourite_post(&self, post_id: &str) -> Result<Post> {
        let response = self.client
            .unfavourite_status(post_id.to_string())
            .await
            .context("Failed to unfavourite post")?;

        Ok(converter::convert_status(&response.json))
    }

    /// Get notifications
    pub async fn get_notifications(&self, request: &NotificationRequest) -> Result<NotificationResponse> {
        let limit = request.limit.unwrap_or(20);

        let options = megalodon::megalodon::GetNotificationsInputOptions {
            max_id: request.max_id.clone(),
            since_id: request.since_id.clone(),
            min_id: request.min_id.clone(),
            limit: Some(limit),
            ..Default::default()
        };

        let response = self.client
            .get_notifications(Some(&options))
            .await
            .context("Failed to fetch notifications")?;

        let notifications: Vec<Notification> = response
            .json
            .iter()
            .filter_map(|n| converter::convert_notification(n))
            .collect();

        let max_id = notifications.first().map(|n| n.id.clone());
        let min_id = notifications.last().map(|n| n.id.clone());
        let has_more = notifications.len() == limit as usize;

        Ok(NotificationResponse {
            notifications,
            max_id,
            min_id,
            has_more,
        })
    }

    /// Clear all notifications
    pub async fn clear_notifications(&self) -> Result<()> {
        self.client
            .dismiss_notifications()
            .await
            .context("Failed to clear notifications")?;

        Ok(())
    }

    /// Dismiss a specific notification
    pub async fn dismiss_notification(&self, notification_id: &str) -> Result<()> {
        self.client
            .dismiss_notification(notification_id.to_string())
            .await
            .context("Failed to dismiss notification")?;

        Ok(())
    }

    /// Upload a media file
    pub async fn upload_media(&self, request: &MediaUploadRequest) -> Result<MediaAttachment> {
        use std::path::Path;

        let path = Path::new(&request.file_path);

        // Validate file exists
        if !path.exists() {
            anyhow::bail!("File not found: {}", request.file_path);
        }

        // Build the upload options
        let options = megalodon::megalodon::UploadMediaInputOptions {
            description: request.description.clone(),
            focus: request.focus.as_ref().map(|f| format!("{},{}", f.x, f.y)),
            ..Default::default()
        };

        // Upload the media using the file path directly
        let response = self.client
            .upload_media(request.file_path.clone(), Some(&options))
            .await
            .context("Failed to upload media")?;

        // Convert the UploadMedia response to our MediaAttachment
        // UploadMedia is an enum - handle both variants
        let attachment = match &response.json {
            megalodon::entities::UploadMedia::Attachment(att) => {
                converter::convert_media(att)
            }
            megalodon::entities::UploadMedia::AsyncAttachment(async_att) => {
                // Async attachment - media is still processing
                // Return minimal info with the ID
                MediaAttachment {
                    id: async_att.id.clone(),
                    media_type: crate::models::MediaType::Unknown,
                    url: async_att.url.clone().unwrap_or_default(),
                    preview_url: async_att.preview_url.clone(),
                    remote_url: None,
                    meta: None,
                    description: async_att.description.clone(),
                    blurhash: async_att.blurhash.clone(),
                }
            }
        };

        info!("Media uploaded: {}", attachment.id);
        Ok(attachment)
    }

    /// Get instance information
    pub async fn get_instance_info(&self) -> Result<InstanceInfo> {
        let response = self.client
            .get_instance()
            .await
            .context("Failed to get instance info")?;

        let instance = &response.json;

        Ok(InstanceInfo {
            url: self.instance_url.clone(),
            title: instance.title.clone(),
            short_description: Some(instance.description.clone()),
            description: instance.description.clone(),
            version: instance.version.clone(),
            user_count: Some(instance.stats.user_count as u64),
            status_count: Some(instance.stats.status_count as u64),
            domain_count: Some(instance.stats.domain_count as u64),
            thumbnail: instance.thumbnail.clone(),
            max_toot_chars: Some(instance.configuration.statuses.max_characters as u32),
            max_media_attachments: instance.configuration.statuses.max_media_attachments.map(|v| v as u32),
            languages: instance.languages.clone(),
            registrations: instance.registrations,
            approval_required: instance.approval_required,
        })
    }
}

/// Normalize an instance URL
fn normalize_url(url: &str) -> String {
    let url = url.trim();
    let url = if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("https://{}", url)
    };

    // Remove trailing slash
    url.trim_end_matches('/').to_string()
}

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

//! Streaming module for real-time updates via WebSocket
//!
//! Handles WebSocket connections to Mastodon streaming API for
//! real-time timeline updates.

use anyhow::Result;
use megalodon::{streaming::Message, SNS};
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, info, warn};

use crate::log_stream;
use crate::models::{events, IpcMessage, Post, TimelineType};
use crate::api::convert_status;

/// Event from the streaming connection
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// New post received
    NewPost(Post),
    /// Post was updated
    PostUpdated(Post),
    /// Post was deleted
    PostDeleted(String),
    /// Stream connected
    Connected,
    /// Stream disconnected
    Disconnected(String),
}

/// Streaming connection manager
pub struct StreamManager {
    /// Access token
    access_token: String,
    /// Instance URL
    instance_url: String,
    /// Shutdown signal sender
    shutdown_tx: broadcast::Sender<()>,
}

impl StreamManager {
    /// Create a new stream manager
    pub fn new(instance_url: &str, access_token: &str) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);

        Self {
            access_token: access_token.to_string(),
            instance_url: instance_url.to_string(),
            shutdown_tx,
        }
    }

    /// Start streaming for a timeline
    pub async fn start_stream(
        &self,
        timeline_type: TimelineType,
        event_tx: mpsc::Sender<IpcMessage>,
    ) -> Result<()> {
        let timeline_name = timeline_type.display_name();
        info!("Starting stream for timeline: {}", timeline_name);

        let client = megalodon::generator(
            SNS::Mastodon,
            self.instance_url.clone(),
            Some(self.access_token.clone()),
            None,
        )?;

        let mut shutdown_rx = self.shutdown_tx.subscribe();

        // Get the appropriate streaming endpoint
        let stream = match &timeline_type {
            TimelineType::Home => client.user_streaming().await,
            TimelineType::Local => client.local_streaming().await,
            TimelineType::Federated => client.public_streaming().await,
            TimelineType::Hashtag { tag } => client.tag_streaming(tag.clone()).await,
            TimelineType::List { list_id } => client.list_streaming(list_id.clone()).await,
            TimelineType::Direct => client.direct_streaming().await,
            _ => {
                warn!("Streaming not supported for timeline type: {:?}", timeline_type);
                return Ok(());
            }
        };

        log_stream!(connected, &timeline_name);
        let _ = event_tx
            .send(IpcMessage::event(
                events::STREAM_CONNECTED,
                serde_json::json!({ "timeline": timeline_name }),
            ))
            .await;

        // Use the listen method from megalodon's Streaming trait
        let event_tx_clone = event_tx.clone();
        let timeline_name_clone = timeline_name.clone();

        // Spawn listening task
        let listen_handle = tokio::spawn(async move {
            stream.listen(Box::new(move |message| {
                let event_tx = event_tx_clone.clone();
                let timeline_name = timeline_name_clone.clone();

                Box::pin(async move {
                    match message {
                        Message::Update(status) => {
                            let post = convert_status(&status);
                            let _ = event_tx
                                .send(IpcMessage::event(
                                    events::NEW_POST,
                                    serde_json::json!({
                                        "timeline": timeline_name,
                                        "post": post
                                    }),
                                ))
                                .await;
                        }
                        Message::Delete(id) => {
                            let _ = event_tx
                                .send(IpcMessage::event(
                                    events::POST_DELETED,
                                    serde_json::json!({
                                        "timeline": timeline_name,
                                        "post_id": id
                                    }),
                                ))
                                .await;
                        }
                        Message::StatusUpdate(status) => {
                            let post = convert_status(&status);
                            let _ = event_tx
                                .send(IpcMessage::event(
                                    events::POST_UPDATED,
                                    serde_json::json!({
                                        "timeline": timeline_name,
                                        "post": post
                                    }),
                                ))
                                .await;
                        }
                        _ => {
                            debug!("Unhandled stream message type");
                        }
                    }
                })
            })).await;
        });

        // Wait for shutdown signal
        let _ = shutdown_rx.recv().await;
        info!("Shutdown signal received, stopping stream");
        listen_handle.abort();

        Ok(())
    }

    /// Handle a streaming message
    async fn handle_message(
        &self,
        message: Message,
        timeline_type: &TimelineType,
        event_tx: &mpsc::Sender<IpcMessage>,
    ) -> Result<()> {
        let timeline_name = timeline_type.display_name();

        match message {
            Message::Update(status) => {
                log_stream!(message, &timeline_name, "update");
                let post = convert_status(&status);
                event_tx
                    .send(IpcMessage::event(
                        events::NEW_POST,
                        serde_json::json!({
                            "timeline": timeline_name,
                            "post": post
                        }),
                    ))
                    .await?;
            }
            Message::Notification(notification) => {
                log_stream!(message, &timeline_name, "notification");
                event_tx
                    .send(IpcMessage::event(
                        events::NEW_NOTIFICATION,
                        serde_json::json!({
                            "notification": notification
                        }),
                    ))
                    .await?;
            }
            Message::Delete(id) => {
                log_stream!(message, &timeline_name, "delete");
                event_tx
                    .send(IpcMessage::event(
                        events::POST_DELETED,
                        serde_json::json!({
                            "timeline": timeline_name,
                            "post_id": id
                        }),
                    ))
                    .await?;
            }
            Message::StatusUpdate(status) => {
                log_stream!(message, &timeline_name, "status_update");
                let post = convert_status(&status);
                event_tx
                    .send(IpcMessage::event(
                        events::POST_UPDATED,
                        serde_json::json!({
                            "timeline": timeline_name,
                            "post": post
                        }),
                    ))
                    .await?;
            }
            _ => {
                debug!("Unhandled stream message type");
            }
        }

        Ok(())
    }

    /// Stop all streaming connections
    pub fn stop_all(&self) {
        let _ = self.shutdown_tx.send(());
    }
}

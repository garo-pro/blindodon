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

//! IPC Server implementation using named pipes

use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{broadcast, Mutex};
use tracing::{debug, error, info, warn};

use crate::api::MastodonClient;
use crate::models::{IpcMessage, MessageType};

use super::handler::MessageHandler;

/// Named pipe name for Windows
#[cfg(windows)]
const PIPE_NAME: &str = r"\\.\pipe\blindodon_ipc";

/// Unix socket path
#[cfg(not(windows))]
const PIPE_NAME: &str = "/tmp/blindodon_ipc.sock";

/// IPC Server that listens for connections from the C# UI
pub struct IpcServer {
    handler: Arc<MessageHandler>,
    shutdown_tx: broadcast::Sender<()>,
}

impl IpcServer {
    /// Create a new IPC server
    pub fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            handler: Arc::new(MessageHandler::new()),
            shutdown_tx,
        }
    }

    /// Get a shutdown signal receiver
    pub fn shutdown_signal(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }

    /// Signal shutdown
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }
}

/// Run the IPC server
pub async fn run_server() -> Result<()> {
    info!("Starting IPC server on {}", PIPE_NAME);

    let server = Arc::new(IpcServer::new());
    let handler = Arc::new(MessageHandler::new());

    #[cfg(windows)]
    {
        run_windows_pipe_server(handler, server.shutdown_signal()).await
    }

    #[cfg(not(windows))]
    {
        run_unix_socket_server(handler, server.shutdown_signal()).await
    }
}

#[cfg(windows)]
async fn run_windows_pipe_server(
    handler: Arc<MessageHandler>,
    mut shutdown: broadcast::Receiver<()>,
) -> Result<()> {
    use tokio::net::windows::named_pipe::{ServerOptions, PipeMode};

    loop {
        // Create a new pipe instance
        let pipe = ServerOptions::new()
            .first_pipe_instance(false)
            .pipe_mode(PipeMode::Message)
            .create(PIPE_NAME)
            .context("Failed to create named pipe")?;

        info!("Waiting for client connection...");

        tokio::select! {
            result = pipe.connect() => {
                match result {
                    Ok(()) => {
                        info!("Client connected");
                        let handler_clone = handler.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_client_windows(pipe, handler_clone).await {
                                error!("Client handler error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept connection: {}", e);
                    }
                }
            }
            _ = shutdown.recv() => {
                info!("Shutdown signal received");
                break;
            }
        }
    }

    Ok(())
}

#[cfg(windows)]
async fn handle_client_windows(
    pipe: tokio::net::windows::named_pipe::NamedPipeServer,
    handler: Arc<MessageHandler>,
) -> Result<()> {
    use tokio::io::{split, AsyncBufReadExt, AsyncWriteExt, BufReader};

    let (reader, writer) = tokio::io::split(pipe);
    let mut reader = BufReader::new(reader);
    let writer = Arc::new(Mutex::new(writer));

    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                info!("Client disconnected");
                break;
            }
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                debug!("Received message: {}", trimmed);

                match serde_json::from_str::<IpcMessage>(trimmed) {
                    Ok(msg) => {
                        let response = handler.handle_message(msg).await;
                        let response_json = serde_json::to_string(&response)?;

                        let mut w = writer.lock().await;
                        w.write_all(response_json.as_bytes()).await?;
                        w.write_all(b"\n").await?;
                        w.flush().await?;

                        debug!("Sent response: {}", response_json);
                    }
                    Err(e) => {
                        warn!("Failed to parse message: {}", e);
                        let error_response = IpcMessage::response_err(
                            "unknown",
                            crate::models::IpcError::new(
                                crate::models::error_codes::PARSE_ERROR,
                                format!("Failed to parse message: {}", e),
                            ),
                        );
                        let response_json = serde_json::to_string(&error_response)?;
                        let mut w = writer.lock().await;
                        w.write_all(response_json.as_bytes()).await?;
                        w.write_all(b"\n").await?;
                        w.flush().await?;
                    }
                }
            }
            Err(e) => {
                error!("Read error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

#[cfg(not(windows))]
async fn run_unix_socket_server(
    handler: Arc<MessageHandler>,
    mut shutdown: broadcast::Receiver<()>,
) -> Result<()> {
    use tokio::net::UnixListener;

    // Remove existing socket file
    let _ = std::fs::remove_file(PIPE_NAME);

    let listener = UnixListener::bind(PIPE_NAME)
        .context("Failed to bind Unix socket")?;

    info!("Listening on {}", PIPE_NAME);

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream, _)) => {
                        info!("Client connected");
                        let handler_clone = handler.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_client_unix(stream, handler_clone).await {
                                error!("Client handler error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept connection: {}", e);
                    }
                }
            }
            _ = shutdown.recv() => {
                info!("Shutdown signal received");
                break;
            }
        }
    }

    // Cleanup
    let _ = std::fs::remove_file(PIPE_NAME);

    Ok(())
}

#[cfg(not(windows))]
async fn handle_client_unix(
    stream: tokio::net::UnixStream,
    handler: Arc<MessageHandler>,
) -> Result<()> {
    let (reader, writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let writer = Arc::new(Mutex::new(writer));

    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                info!("Client disconnected");
                break;
            }
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                debug!("Received message: {}", trimmed);

                match serde_json::from_str::<IpcMessage>(trimmed) {
                    Ok(msg) => {
                        let response = handler.handle_message(msg).await;
                        let response_json = serde_json::to_string(&response)?;

                        let mut w = writer.lock().await;
                        w.write_all(response_json.as_bytes()).await?;
                        w.write_all(b"\n").await?;
                        w.flush().await?;

                        debug!("Sent response: {}", response_json);
                    }
                    Err(e) => {
                        warn!("Failed to parse message: {}", e);
                        let error_response = IpcMessage::response_err(
                            "unknown",
                            crate::models::IpcError::new(
                                crate::models::error_codes::PARSE_ERROR,
                                format!("Failed to parse message: {}", e),
                            ),
                        );
                        let response_json = serde_json::to_string(&error_response)?;
                        let mut w = writer.lock().await;
                        w.write_all(response_json.as_bytes()).await?;
                        w.write_all(b"\n").await?;
                        w.flush().await?;
                    }
                }
            }
            Err(e) => {
                error!("Read error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

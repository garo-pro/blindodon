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

//! Logging system for Blindodon
//!
//! Provides structured logging with multiple output targets,
//! log rotation, and configurable verbosity levels.

use anyhow::Result;
use std::path::PathBuf;
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// Logger configuration
pub struct LoggerConfig {
    /// Log directory path
    pub log_dir: PathBuf,
    /// Log file prefix
    pub file_prefix: String,
    /// Maximum log level
    pub level: Level,
    /// Whether to log to console
    pub console_output: bool,
    /// Whether to log to file
    pub file_output: bool,
    /// Log rotation strategy
    pub rotation: Rotation,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        let log_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Blindodon")
            .join("logs");

        Self {
            log_dir,
            file_prefix: "blindodon".to_string(),
            level: Level::INFO,
            console_output: true,
            file_output: true,
            rotation: Rotation::DAILY,
        }
    }
}

/// Main logger struct
pub struct Logger;

impl Logger {
    /// Initialize the logging system with default configuration
    pub fn init() -> Result<()> {
        Self::init_with_config(LoggerConfig::default())
    }

    /// Initialize the logging system with custom configuration
    pub fn init_with_config(config: LoggerConfig) -> Result<()> {
        // Ensure log directory exists
        if config.file_output {
            std::fs::create_dir_all(&config.log_dir)?;
        }

        // Create environment filter
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| {
                EnvFilter::new(format!("mastodon_core={}", config.level))
                    .add_directive(format!("blindodon={}", config.level).parse().unwrap())
            });

        // Build the subscriber
        let subscriber = tracing_subscriber::registry().with(env_filter);

        if config.console_output && config.file_output {
            // Both console and file output
            let file_appender = RollingFileAppender::new(
                config.rotation,
                &config.log_dir,
                &config.file_prefix,
            );

            let file_layer = fmt::layer()
                .with_writer(file_appender)
                .with_ansi(false)
                .with_span_events(FmtSpan::CLOSE)
                .json();

            let console_layer = fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(true)
                .with_span_events(FmtSpan::CLOSE);

            subscriber
                .with(file_layer)
                .with(console_layer)
                .init();
        } else if config.console_output {
            // Console only
            let console_layer = fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(true)
                .with_span_events(FmtSpan::CLOSE);

            subscriber.with(console_layer).init();
        } else if config.file_output {
            // File only
            let file_appender = RollingFileAppender::new(
                config.rotation,
                &config.log_dir,
                &config.file_prefix,
            );

            let file_layer = fmt::layer()
                .with_writer(file_appender)
                .with_ansi(false)
                .with_span_events(FmtSpan::CLOSE)
                .json();

            subscriber.with(file_layer).init();
        }

        Ok(())
    }
}

/// Macro for logging API calls with timing
#[macro_export]
macro_rules! log_api_call {
    ($method:expr, $url:expr) => {
        tracing::info!(
            target: "api",
            method = $method,
            url = $url,
            "API call started"
        )
    };
    ($method:expr, $url:expr, $duration:expr) => {
        tracing::info!(
            target: "api",
            method = $method,
            url = $url,
            duration_ms = $duration,
            "API call completed"
        )
    };
}

/// Macro for logging IPC messages
#[macro_export]
macro_rules! log_ipc {
    (request, $method:expr, $id:expr) => {
        tracing::debug!(
            target: "ipc",
            direction = "request",
            method = $method,
            id = $id,
            "IPC request received"
        )
    };
    (response, $method:expr, $id:expr, $success:expr) => {
        tracing::debug!(
            target: "ipc",
            direction = "response",
            method = $method,
            id = $id,
            success = $success,
            "IPC response sent"
        )
    };
    (event, $event:expr) => {
        tracing::debug!(
            target: "ipc",
            direction = "event",
            event = $event,
            "IPC event sent"
        )
    };
}

/// Macro for logging streaming events
#[macro_export]
macro_rules! log_stream {
    (connected, $timeline:expr) => {
        tracing::info!(
            target: "streaming",
            event = "connected",
            timeline = $timeline,
            "Stream connected"
        )
    };
    (disconnected, $timeline:expr, $reason:expr) => {
        tracing::warn!(
            target: "streaming",
            event = "disconnected",
            timeline = $timeline,
            reason = $reason,
            "Stream disconnected"
        )
    };
    (message, $timeline:expr, $msg_type:expr) => {
        tracing::trace!(
            target: "streaming",
            event = "message",
            timeline = $timeline,
            message_type = $msg_type,
            "Stream message received"
        )
    };
}

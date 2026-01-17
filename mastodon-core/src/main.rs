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

//! Mastodon Core - Rust backend for Blindodon
//!
//! This binary runs as a background process and communicates with the C# UI
//! via named pipes using JSON-based IPC protocol.

mod api;
mod cache;
mod crypto;
mod ipc;
mod logger;
mod models;
mod streaming;

use anyhow::Result;
use logger::Logger;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging system
    Logger::init()?;

    info!("Mastodon Core starting up...");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));

    // Initialize the IPC server
    match ipc::server::run_server().await {
        Ok(_) => {
            info!("Mastodon Core shutting down gracefully");
        }
        Err(e) => {
            error!("Fatal error in IPC server: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

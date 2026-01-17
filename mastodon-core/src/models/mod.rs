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

//! Data models for Blindodon
//!
//! These models represent the core data structures used throughout the application,
//! including posts, users, notifications, and IPC messages.

mod post;
mod user;
mod notification;
mod timeline;
mod ipc_message;
mod account;
mod media;

pub use post::*;
pub use user::*;
pub use notification::*;
pub use timeline::*;
pub use ipc_message::*;
pub use account::*;
pub use media::*;

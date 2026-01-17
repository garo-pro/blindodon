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

//! Media attachment model

use serde::{Deserialize, Serialize};

/// Type of media attachment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    Image,
    Video,
    Gifv,
    Audio,
    Unknown,
}

/// A media attachment on a post
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaAttachment {
    /// Unique identifier
    pub id: String,

    /// Type of media
    #[serde(rename = "type")]
    pub media_type: MediaType,

    /// URL to the media file
    pub url: String,

    /// URL to the preview image
    pub preview_url: Option<String>,

    /// URL for remote media
    pub remote_url: Option<String>,

    /// Metadata about the media
    pub meta: Option<MediaMeta>,

    /// Alt text description
    pub description: Option<String>,

    /// Blurhash for placeholder
    pub blurhash: Option<String>,
}

/// Metadata about a media attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaMeta {
    pub original: Option<MediaDimensions>,
    pub small: Option<MediaDimensions>,
    pub focus: Option<MediaFocus>,
    pub length: Option<String>,
    pub duration: Option<f64>,
    pub fps: Option<u32>,
    pub audio_encode: Option<String>,
    pub audio_bitrate: Option<String>,
    pub audio_channels: Option<String>,
}

/// Dimensions of a media file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaDimensions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub size: Option<String>,
    pub aspect: Option<f64>,
    pub frame_rate: Option<String>,
    pub duration: Option<f64>,
    pub bitrate: Option<u64>,
}

/// Focus point for cropping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaFocus {
    pub x: f64,
    pub y: f64,
}

/// Request to upload media
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaUploadRequest {
    /// Path to the file to upload
    pub file_path: String,
    /// Alt text description
    pub description: Option<String>,
    /// Focus point
    pub focus: Option<MediaFocus>,
}

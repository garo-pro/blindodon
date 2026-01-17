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

//! Type converters from megalodon types to Blindodon types

use megalodon::entities;

use crate::models::{
    Application, CustomEmoji, MediaAttachment, MediaDimensions, MediaFocus, MediaMeta,
    MediaType, Mention, Notification, NotificationType, Poll, PollOption, Post, ProfileField,
    Tag, User, Visibility,
};

/// Convert a megalodon Status to a Blindodon Post
pub fn convert_status(status: &entities::Status) -> Post {
    Post {
        id: status.id.clone(),
        uri: status.uri.clone(),
        url: status.url.clone(),
        account: convert_account(&status.account),
        content: status.content.clone(),
        plain_content: Some(strip_html(&status.content)),
        spoiler_text: status.spoiler_text.clone(),
        visibility: convert_visibility(&status.visibility),
        sensitive: status.sensitive,
        created_at: status.created_at,
        edited_at: status.edited_at,
        language: status.language.clone(),
        in_reply_to_id: status.in_reply_to_id.clone(),
        in_reply_to_account_id: status.in_reply_to_account_id.clone(),
        media_attachments: status.media_attachments.iter().map(convert_media).collect(),
        tags: status.tags.iter().map(|t| Tag { name: t.name.clone(), url: t.url.clone() }).collect(),
        mentions: status.mentions.iter().map(convert_mention).collect(),
        emojis: status.emojis.iter().map(convert_emoji).collect(),
        reblogs_count: status.reblogs_count as u64,
        favourites_count: status.favourites_count as u64,
        replies_count: status.replies_count as u64,
        reblog: status.reblog.as_ref().map(|r| Box::new(convert_status(r))),
        poll: status.poll.as_ref().map(convert_poll),
        application: status.application.as_ref().map(convert_application),
        reblogged: status.reblogged,
        favourited: status.favourited,
        bookmarked: status.bookmarked,
        muted: status.muted,
        pinned: status.pinned,
        blindodon_encrypted: false,
    }
}

/// Convert a megalodon Account to a Blindodon User
pub fn convert_account(account: &entities::Account) -> User {
    User {
        id: account.id.clone(),
        username: account.username.clone(),
        acct: account.acct.clone(),
        display_name: account.display_name.clone(),
        note: account.note.clone(),
        url: account.url.clone(),
        avatar: account.avatar.clone(),
        avatar_static: account.avatar_static.clone(),
        header: account.header.clone(),
        header_static: account.header_static.clone(),
        locked: account.locked,
        fields: account.fields.iter().map(convert_field).collect(),
        emojis: account.emojis.iter().map(convert_emoji).collect(),
        bot: account.bot,
        group: account.group.unwrap_or(false),
        discoverable: account.discoverable,
        created_at: account.created_at,
        last_status_at: None,
        statuses_count: account.statuses_count as u64,
        followers_count: account.followers_count as u64,
        following_count: account.following_count as u64,
        following: None,
        followed_by: None,
        blocking: None,
        muting: None,
        muting_notifications: None,
        requested: None,
        domain_blocking: None,
        endorsed: None,
        note_personal: None,
        blindodon_pm_capable: false,
        blindodon_pm_public_key: None,
    }
}

/// Convert visibility
fn convert_visibility(visibility: &entities::StatusVisibility) -> Visibility {
    match visibility {
        entities::StatusVisibility::Public => Visibility::Public,
        entities::StatusVisibility::Unlisted => Visibility::Unlisted,
        entities::StatusVisibility::Private => Visibility::Private,
        entities::StatusVisibility::Direct => Visibility::Direct,
        entities::StatusVisibility::Local => Visibility::Unlisted, // Map Local to Unlisted
    }
}

/// Convert a media attachment
pub fn convert_media(media: &entities::Attachment) -> MediaAttachment {
    MediaAttachment {
        id: media.id.clone(),
        media_type: convert_media_type(&media.r#type),
        url: media.url.clone(),
        preview_url: media.preview_url.clone(),
        remote_url: media.remote_url.clone(),
        meta: media.meta.as_ref().map(convert_media_meta),
        description: media.description.clone(),
        blurhash: media.blurhash.clone(),
    }
}

/// Convert media type
fn convert_media_type(media_type: &entities::attachment::AttachmentType) -> MediaType {
    match media_type {
        entities::attachment::AttachmentType::Image => MediaType::Image,
        entities::attachment::AttachmentType::Video => MediaType::Video,
        entities::attachment::AttachmentType::Gifv => MediaType::Gifv,
        entities::attachment::AttachmentType::Audio => MediaType::Audio,
        entities::attachment::AttachmentType::Unknown => MediaType::Unknown,
    }
}

/// Convert media meta
fn convert_media_meta(meta: &entities::attachment::AttachmentMeta) -> MediaMeta {
    MediaMeta {
        original: meta.original.as_ref().map(convert_media_dimensions),
        small: meta.small.as_ref().map(convert_media_dimensions),
        focus: meta.focus.as_ref().map(|f| MediaFocus { x: f.x, y: f.y }),
        length: meta.length.clone(),
        duration: meta.duration,
        fps: meta.fps.map(|f| f as u32),
        audio_encode: meta.audio_encode.clone(),
        audio_bitrate: meta.audio_bitrate.clone(),
        audio_channels: meta.audio_channel.clone(),
    }
}

/// Convert media dimensions
fn convert_media_dimensions(dims: &entities::attachment::MetaSub) -> MediaDimensions {
    MediaDimensions {
        width: dims.width.map(|w| w as u32),
        height: dims.height.map(|h| h as u32),
        size: dims.size.clone(),
        aspect: dims.aspect,
        frame_rate: dims.frame_rate.clone(),
        duration: dims.duration,
        bitrate: dims.bitrate.map(|b| b as u64),
    }
}

/// Convert a tag
fn convert_tag(tag: &entities::Tag) -> Tag {
    Tag {
        name: tag.name.clone(),
        url: tag.url.clone(),
    }
}

/// Convert a mention
fn convert_mention(mention: &entities::Mention) -> Mention {
    Mention {
        id: mention.id.clone(),
        username: mention.username.clone(),
        acct: mention.acct.clone(),
        url: mention.url.clone(),
    }
}

/// Convert a custom emoji
fn convert_emoji(emoji: &entities::Emoji) -> CustomEmoji {
    CustomEmoji {
        shortcode: emoji.shortcode.clone(),
        url: emoji.url.clone(),
        static_url: emoji.static_url.clone(),
        visible_in_picker: emoji.visible_in_picker,
        category: emoji.category.clone(),
    }
}

/// Convert a poll
fn convert_poll(poll: &entities::Poll) -> Poll {
    Poll {
        id: poll.id.clone(),
        expires_at: poll.expires_at,
        expired: poll.expired,
        multiple: poll.multiple,
        votes_count: poll.votes_count as u64,
        voters_count: poll.voters_count.map(|c| c as u64),
        options: poll.options.iter().map(convert_poll_option).collect(),
        voted: poll.voted,
        own_votes: None,
    }
}

/// Convert a poll option
fn convert_poll_option(option: &entities::PollOption) -> PollOption {
    PollOption {
        title: option.title.clone(),
        votes_count: option.votes_count.map(|c| c as u64),
    }
}

/// Convert a profile field
fn convert_field(field: &entities::Field) -> ProfileField {
    ProfileField {
        name: field.name.clone(),
        value: field.value.clone(),
        verified_at: field.verified_at,
    }
}

/// Convert an application
fn convert_application(app: &entities::Application) -> Application {
    Application {
        name: app.name.clone(),
        website: app.website.clone(),
    }
}

/// Convert a megalodon Notification to a Blindodon Notification
pub fn convert_notification(notification: &entities::Notification) -> Option<Notification> {
    // Account is required for our notification model
    let account = notification.account.as_ref()?;

    Some(Notification {
        id: notification.id.clone(),
        notification_type: convert_notification_type(&notification.r#type),
        created_at: notification.created_at,
        account: convert_account(account),
        status: notification.status.as_ref().map(|s| convert_status(s)),
        read: false,
    })
}

/// Convert notification type
fn convert_notification_type(notification_type: &entities::notification::NotificationType) -> NotificationType {
    match notification_type {
        entities::notification::NotificationType::Mention => NotificationType::Mention,
        entities::notification::NotificationType::Reblog => NotificationType::Reblog,
        entities::notification::NotificationType::Favourite => NotificationType::Favourite,
        entities::notification::NotificationType::Follow => NotificationType::Follow,
        entities::notification::NotificationType::FollowRequest => NotificationType::FollowRequest,
        entities::notification::NotificationType::PollExpired => NotificationType::Poll,
        entities::notification::NotificationType::Update => NotificationType::Update,
        entities::notification::NotificationType::AdminSignup => NotificationType::AdminSignUp,
        entities::notification::NotificationType::AdminReport => NotificationType::AdminReport,
        _ => NotificationType::Unknown,
    }
}

/// Strip HTML tags from content for plain text
fn strip_html(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    // Decode common HTML entities
    result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
}

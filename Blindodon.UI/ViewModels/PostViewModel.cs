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

using CommunityToolkit.Mvvm.ComponentModel;
using Newtonsoft.Json.Linq;

namespace Blindodon.ViewModels;

/// <summary>
/// View model for a Mastodon post
/// </summary>
public partial class PostViewModel : ObservableObject
{
    [ObservableProperty]
    private string _id = "";

    [ObservableProperty]
    private string _content = "";

    [ObservableProperty]
    private string _plainContent = "";

    [ObservableProperty]
    private string _spoilerText = "";

    [ObservableProperty]
    private string _visibility = "public";

    [ObservableProperty]
    private bool _sensitive;

    [ObservableProperty]
    private DateTime _createdAt;

    [ObservableProperty]
    private DateTime? _editedAt;

    [ObservableProperty]
    private string? _language;

    [ObservableProperty]
    private string? _inReplyToId;

    [ObservableProperty]
    private int _reblogsCount;

    [ObservableProperty]
    private int _favouritesCount;

    [ObservableProperty]
    private int _repliesCount;

    [ObservableProperty]
    private bool _reblogged;

    [ObservableProperty]
    private bool _favourited;

    [ObservableProperty]
    private bool _bookmarked;

    [ObservableProperty]
    private bool _muted;

    [ObservableProperty]
    private bool _pinned;

    [ObservableProperty]
    private UserViewModel _account = new();

    [ObservableProperty]
    private PostViewModel? _reblog;

    [ObservableProperty]
    private List<MediaAttachmentViewModel> _mediaAttachments = new();

    /// <summary>
    /// Gets whether this post has a content warning
    /// </summary>
    public bool HasContentWarning => !string.IsNullOrEmpty(SpoilerText);

    /// <summary>
    /// Gets whether this post has media attachments
    /// </summary>
    public bool HasMedia => MediaAttachments.Count > 0;

    /// <summary>
    /// Gets whether this is a boost of another post
    /// </summary>
    public bool IsReblog => Reblog != null;

    /// <summary>
    /// Gets the effective post (the reblog if this is a boost, otherwise this)
    /// </summary>
    public PostViewModel EffectivePost => Reblog ?? this;

    /// <summary>
    /// Create a PostViewModel from JSON
    /// </summary>
    public static PostViewModel FromJson(JObject json)
    {
        var post = new PostViewModel
        {
            Id = json["id"]?.Value<string>() ?? "",
            Content = json["content"]?.Value<string>() ?? "",
            PlainContent = json["plain_content"]?.Value<string>() ?? StripHtml(json["content"]?.Value<string>() ?? ""),
            SpoilerText = json["spoiler_text"]?.Value<string>() ?? "",
            Visibility = json["visibility"]?.Value<string>() ?? "public",
            Sensitive = json["sensitive"]?.Value<bool>() ?? false,
            CreatedAt = json["created_at"]?.Value<DateTime>() ?? DateTime.Now,
            Language = json["language"]?.Value<string>(),
            InReplyToId = json["in_reply_to_id"]?.Value<string>(),
            ReblogsCount = json["reblogs_count"]?.Value<int>() ?? 0,
            FavouritesCount = json["favourites_count"]?.Value<int>() ?? 0,
            RepliesCount = json["replies_count"]?.Value<int>() ?? 0,
            Reblogged = json["reblogged"]?.Value<bool>() ?? false,
            Favourited = json["favourited"]?.Value<bool>() ?? false,
            Bookmarked = json["bookmarked"]?.Value<bool>() ?? false,
            Muted = json["muted"]?.Value<bool>() ?? false,
            Pinned = json["pinned"]?.Value<bool>() ?? false
        };

        // Parse account
        var accountJson = json["account"] as JObject;
        if (accountJson != null)
        {
            post.Account = UserViewModel.FromJson(accountJson);
        }

        // Parse reblog
        var reblogJson = json["reblog"] as JObject;
        if (reblogJson != null)
        {
            post.Reblog = FromJson(reblogJson);
        }

        // Parse media attachments
        var mediaJson = json["media_attachments"] as JArray;
        if (mediaJson != null)
        {
            foreach (var media in mediaJson)
            {
                if (media is JObject mediaObj)
                {
                    post.MediaAttachments.Add(MediaAttachmentViewModel.FromJson(mediaObj));
                }
            }
        }

        return post;
    }

    private static string StripHtml(string html)
    {
        // Simple HTML stripping
        var result = System.Text.RegularExpressions.Regex.Replace(html, "<[^>]+>", "");
        result = System.Net.WebUtility.HtmlDecode(result);
        return result.Trim();
    }
}

/// <summary>
/// View model for a media attachment
/// </summary>
public partial class MediaAttachmentViewModel : ObservableObject
{
    [ObservableProperty]
    private string _id = "";

    [ObservableProperty]
    private string _type = "image";

    [ObservableProperty]
    private string _url = "";

    [ObservableProperty]
    private string? _previewUrl;

    [ObservableProperty]
    private string? _description;

    [ObservableProperty]
    private string? _blurhash;

    public static MediaAttachmentViewModel FromJson(JObject json)
    {
        return new MediaAttachmentViewModel
        {
            Id = json["id"]?.Value<string>() ?? "",
            Type = json["type"]?.Value<string>() ?? "image",
            Url = json["url"]?.Value<string>() ?? "",
            PreviewUrl = json["preview_url"]?.Value<string>(),
            Description = json["description"]?.Value<string>(),
            Blurhash = json["blurhash"]?.Value<string>()
        };
    }
}

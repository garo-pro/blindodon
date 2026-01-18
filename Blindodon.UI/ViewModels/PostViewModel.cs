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
            Id = GetString(json, "id") ?? "",
            Content = GetString(json, "content") ?? "",
            PlainContent = GetString(json, "plain_content") ?? StripHtml(GetString(json, "content") ?? ""),
            SpoilerText = GetString(json, "spoiler_text") ?? "",
            Visibility = GetString(json, "visibility") ?? "public",
            Sensitive = GetBool(json, "sensitive", false),
            CreatedAt = GetDateTime(json, "created_at", DateTime.Now),
            Language = GetString(json, "language", null),
            InReplyToId = GetString(json, "in_reply_to_id", null),
            ReblogsCount = GetInt(json, "reblogs_count", 0),
            FavouritesCount = GetInt(json, "favourites_count", 0),
            RepliesCount = GetInt(json, "replies_count", 0),
            Reblogged = GetBool(json, "reblogged", false),
            Favourited = GetBool(json, "favourited", false),
            Bookmarked = GetBool(json, "bookmarked", false),
            Muted = GetBool(json, "muted", false),
            Pinned = GetBool(json, "pinned", false)
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

    private static string? GetString(JObject json, string key, string? defaultValue = null)
    {
        var token = json[key];
        if (token == null || token.Type == JTokenType.Null)
            return defaultValue;
        return token.Value<string>() ?? defaultValue;
    }

    private static int GetInt(JObject json, string key, int defaultValue)
    {
        var token = json[key];
        if (token == null || token.Type == JTokenType.Null)
            return defaultValue;
        return token.Value<int>();
    }

    private static bool GetBool(JObject json, string key, bool defaultValue)
    {
        var token = json[key];
        if (token == null || token.Type == JTokenType.Null)
            return defaultValue;
        return token.Value<bool>();
    }

    private static DateTime GetDateTime(JObject json, string key, DateTime defaultValue)
    {
        var token = json[key];
        if (token == null || token.Type == JTokenType.Null)
            return defaultValue;
        return token.Value<DateTime>();
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

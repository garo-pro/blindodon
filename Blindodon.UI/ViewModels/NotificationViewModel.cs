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
/// View model for a Mastodon notification
/// </summary>
public partial class NotificationViewModel : ObservableObject
{
    [ObservableProperty]
    private string _id = "";

    [ObservableProperty]
    private string _type = "";

    [ObservableProperty]
    private DateTime _createdAt;

    [ObservableProperty]
    private UserViewModel _account = new();

    [ObservableProperty]
    private PostViewModel? _status;

    [ObservableProperty]
    private bool _read;

    /// <summary>
    /// Gets a human-readable description of the notification for accessibility
    /// </summary>
    public string DisplayText
    {
        get
        {
            var from = Account.EffectiveDisplayName;
            return Type switch
            {
                "mention" => $"{from} mentioned you",
                "reblog" => $"{from} boosted your post",
                "favourite" => $"{from} favorited your post",
                "follow" => $"{from} followed you",
                "follow_request" => $"{from} requested to follow you",
                "poll" => "A poll you voted in has ended",
                "update" => $"{from} edited a post you interacted with",
                "admin_sign_up" => $"{from} signed up",
                "admin_report" => "New report submitted",
                _ => $"Notification from {from}"
            };
        }
    }

    /// <summary>
    /// Gets the icon text for this notification type
    /// </summary>
    public string TypeIcon => Type switch
    {
        "mention" => "@",
        "reblog" => "♻",
        "favourite" => "★",
        "follow" => "+",
        "follow_request" => "?",
        "poll" => "📊",
        "update" => "✎",
        _ => "•"
    };

    /// <summary>
    /// Gets whether this notification has an associated post
    /// </summary>
    public bool HasStatus => Status != null;

    /// <summary>
    /// Gets the relative time string for display
    /// </summary>
    public string RelativeTime
    {
        get
        {
            var span = DateTime.UtcNow - CreatedAt;
            if (span.TotalMinutes < 1) return "just now";
            if (span.TotalMinutes < 60) return $"{(int)span.TotalMinutes}m";
            if (span.TotalHours < 24) return $"{(int)span.TotalHours}h";
            if (span.TotalDays < 7) return $"{(int)span.TotalDays}d";
            return CreatedAt.ToString("MMM d");
        }
    }

    /// <summary>
    /// Create a NotificationViewModel from JSON
    /// </summary>
    public static NotificationViewModel FromJson(JObject json)
    {
        var notification = new NotificationViewModel
        {
            Id = json["id"]?.Value<string>() ?? "",
            Type = json["notification_type"]?.Value<string>() ?? json["type"]?.Value<string>() ?? "",
            CreatedAt = json["created_at"]?.Value<DateTime>() ?? DateTime.Now,
            Read = json["read"]?.Value<bool>() ?? false
        };

        // Parse account
        var accountJson = json["account"] as JObject;
        if (accountJson != null)
        {
            notification.Account = UserViewModel.FromJson(accountJson);
        }

        // Parse associated status/post
        var statusJson = json["status"] as JObject;
        if (statusJson != null)
        {
            notification.Status = PostViewModel.FromJson(statusJson);
        }

        return notification;
    }
}

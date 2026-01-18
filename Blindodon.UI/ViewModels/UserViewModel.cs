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
/// View model for a Mastodon user/account
/// </summary>
public partial class UserViewModel : ObservableObject
{
    [ObservableProperty]
    private string _id = "";

    [ObservableProperty]
    private string _username = "";

    [ObservableProperty]
    private string _acct = "";

    [ObservableProperty]
    private string _displayName = "";

    [ObservableProperty]
    private string _note = "";

    [ObservableProperty]
    private string _url = "";

    [ObservableProperty]
    private string _avatar = "";

    [ObservableProperty]
    private string _avatarStatic = "";

    [ObservableProperty]
    private string _header = "";

    [ObservableProperty]
    private string _headerStatic = "";

    [ObservableProperty]
    private bool _locked;

    [ObservableProperty]
    private bool _bot;

    [ObservableProperty]
    private DateTime _createdAt;

    [ObservableProperty]
    private int _statusesCount;

    [ObservableProperty]
    private int _followersCount;

    [ObservableProperty]
    private int _followingCount;

    [ObservableProperty]
    private bool _following;

    [ObservableProperty]
    private bool _followedBy;

    [ObservableProperty]
    private bool _blocking;

    [ObservableProperty]
    private bool _muting;

    /// <summary>
    /// Gets the display name or username if display name is empty
    /// </summary>
    public string EffectiveDisplayName => string.IsNullOrWhiteSpace(DisplayName) ? Username : DisplayName;

    /// <summary>
    /// Gets the initials for avatar placeholder
    /// </summary>
    public string Initials
    {
        get
        {
            var name = EffectiveDisplayName;
            if (string.IsNullOrEmpty(name))
                return "?";

            var parts = name.Split(' ', StringSplitOptions.RemoveEmptyEntries);
            if (parts.Length >= 2)
                return $"{parts[0][0]}{parts[1][0]}".ToUpper();

            return name.Length >= 2 ? name[..2].ToUpper() : name.ToUpper();
        }
    }

    /// <summary>
    /// Create a UserViewModel from JSON
    /// </summary>
    public static UserViewModel FromJson(JObject json)
    {
        return new UserViewModel
        {
            Id = GetString(json, "id", ""),
            Username = GetString(json, "username", ""),
            Acct = GetString(json, "acct", ""),
            DisplayName = GetString(json, "display_name", ""),
            Note = GetString(json, "note", ""),
            Url = GetString(json, "url", ""),
            Avatar = GetString(json, "avatar", ""),
            AvatarStatic = GetString(json, "avatar_static", ""),
            Header = GetString(json, "header", ""),
            HeaderStatic = GetString(json, "header_static", ""),
            Locked = GetBool(json, "locked", false),
            Bot = GetBool(json, "bot", false),
            CreatedAt = GetDateTime(json, "created_at", DateTime.Now),
            StatusesCount = GetInt(json, "statuses_count", 0),
            FollowersCount = GetInt(json, "followers_count", 0),
            FollowingCount = GetInt(json, "following_count", 0),
            Following = GetBool(json, "following", false),
            FollowedBy = GetBool(json, "followed_by", false),
            Blocking = GetBool(json, "blocking", false),
            Muting = GetBool(json, "muting", false)
        };
    }

    private static string GetString(JObject json, string key, string defaultValue)
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

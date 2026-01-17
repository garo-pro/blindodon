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
            Id = json["id"]?.Value<string>() ?? "",
            Username = json["username"]?.Value<string>() ?? "",
            Acct = json["acct"]?.Value<string>() ?? "",
            DisplayName = json["display_name"]?.Value<string>() ?? "",
            Note = json["note"]?.Value<string>() ?? "",
            Url = json["url"]?.Value<string>() ?? "",
            Avatar = json["avatar"]?.Value<string>() ?? "",
            AvatarStatic = json["avatar_static"]?.Value<string>() ?? "",
            Header = json["header"]?.Value<string>() ?? "",
            HeaderStatic = json["header_static"]?.Value<string>() ?? "",
            Locked = json["locked"]?.Value<bool>() ?? false,
            Bot = json["bot"]?.Value<bool>() ?? false,
            CreatedAt = json["created_at"]?.Value<DateTime>() ?? DateTime.Now,
            StatusesCount = json["statuses_count"]?.Value<int>() ?? 0,
            FollowersCount = json["followers_count"]?.Value<int>() ?? 0,
            FollowingCount = json["following_count"]?.Value<int>() ?? 0,
            Following = json["following"]?.Value<bool>() ?? false,
            FollowedBy = json["followed_by"]?.Value<bool>() ?? false,
            Blocking = json["blocking"]?.Value<bool>() ?? false,
            Muting = json["muting"]?.Value<bool>() ?? false
        };
    }
}

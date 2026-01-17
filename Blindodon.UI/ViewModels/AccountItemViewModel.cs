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
/// View model representing a saved Mastodon account for display in the account selection list.
/// </summary>
public partial class AccountItemViewModel : ObservableObject
{
    [ObservableProperty]
    private string _id = "";

    [ObservableProperty]
    private string _instanceUrl = "";

    [ObservableProperty]
    private string _username = "";

    [ObservableProperty]
    private string _displayName = "";

    [ObservableProperty]
    private string _avatarUrl = "";

    [ObservableProperty]
    private bool _isDefault;

    [ObservableProperty]
    private DateTime _lastUsedAt;

    /// <summary>
    /// Gets the display name if set, otherwise falls back to username.
    /// </summary>
    public string EffectiveDisplayName =>
        string.IsNullOrWhiteSpace(DisplayName) ? Username : DisplayName;

    /// <summary>
    /// Gets the initials for avatar placeholder display.
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
    /// Gets the domain portion of the instance URL for display.
    /// </summary>
    public string InstanceDomain
    {
        get
        {
            if (string.IsNullOrEmpty(InstanceUrl))
                return "";

            try
            {
                var uri = new Uri(InstanceUrl);
                return uri.Host;
            }
            catch
            {
                return InstanceUrl;
            }
        }
    }

    /// <summary>
    /// Gets the full accessibility label for screen readers.
    /// </summary>
    public string AccessibilityLabel =>
        $"{EffectiveDisplayName} at {InstanceDomain}" + (IsDefault ? ", default account" : "");

    /// <summary>
    /// Creates an AccountItemViewModel from a JSON object received from the backend.
    /// </summary>
    public static AccountItemViewModel FromJson(JObject json)
    {
        return new AccountItemViewModel
        {
            Id = json["id"]?.Value<string>() ?? "",
            InstanceUrl = json["instance_url"]?.Value<string>() ?? "",
            Username = json["username"]?.Value<string>() ?? "",
            DisplayName = json["display_name"]?.Value<string>() ?? "",
            AvatarUrl = json["avatar_url"]?.Value<string>() ?? "",
            IsDefault = json["is_default"]?.Value<bool>() ?? false,
            LastUsedAt = json["last_used_at"]?.Value<DateTime>() ?? DateTime.MinValue
        };
    }
}

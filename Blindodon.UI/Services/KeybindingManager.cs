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

using System.Windows.Input;
using Serilog;

namespace Blindodon.Services;

/// <summary>
/// Manages keyboard bindings and shortcuts
/// </summary>
public class KeybindingManager
{
    private readonly Dictionary<string, KeyBinding> _bindings = new();
    private readonly Dictionary<string, Action> _actions = new();

    /// <summary>
    /// Represents a key binding
    /// </summary>
    public class KeyBinding
    {
        public Key Key { get; set; }
        public ModifierKeys Modifiers { get; set; }
        public string Description { get; set; } = "";

        public override string ToString()
        {
            var parts = new List<string>();

            if (Modifiers.HasFlag(ModifierKeys.Control))
                parts.Add("Ctrl");
            if (Modifiers.HasFlag(ModifierKeys.Alt))
                parts.Add("Alt");
            if (Modifiers.HasFlag(ModifierKeys.Shift))
                parts.Add("Shift");

            parts.Add(Key.ToString());

            return string.Join("+", parts);
        }
    }

    public KeybindingManager()
    {
        LoadDefaultBindings();
    }

    private void LoadDefaultBindings()
    {
        // Navigation
        RegisterBinding("NextPost", Key.J, ModifierKeys.None, "Move to next post");
        RegisterBinding("PreviousPost", Key.K, ModifierKeys.None, "Move to previous post");
        RegisterBinding("FirstPost", Key.Home, ModifierKeys.Control, "Go to first post");
        RegisterBinding("LastPost", Key.End, ModifierKeys.Control, "Go to last post");

        // Timeline switching
        RegisterBinding("HomeTimeline", Key.D1, ModifierKeys.Control, "Switch to Home timeline");
        RegisterBinding("LocalTimeline", Key.D2, ModifierKeys.Control, "Switch to Local timeline");
        RegisterBinding("FederatedTimeline", Key.D3, ModifierKeys.Control, "Switch to Federated timeline");
        RegisterBinding("Notifications", Key.D4, ModifierKeys.Control, "Switch to Notifications");
        RegisterBinding("DirectMessages", Key.D5, ModifierKeys.Control, "Switch to Direct Messages");

        // Buffer navigation (like TweeseCake)
        RegisterBinding("NextBuffer", Key.Right, ModifierKeys.Control, "Switch to next buffer");
        RegisterBinding("PreviousBuffer", Key.Left, ModifierKeys.Control, "Switch to previous buffer");

        // Post actions
        RegisterBinding("Reply", Key.R, ModifierKeys.None, "Reply to post");
        RegisterBinding("Boost", Key.B, ModifierKeys.None, "Boost/reblog post");
        RegisterBinding("Favorite", Key.F, ModifierKeys.None, "Favorite post");
        RegisterBinding("Bookmark", Key.D, ModifierKeys.None, "Bookmark post");
        RegisterBinding("OpenInBrowser", Key.O, ModifierKeys.None, "Open post in browser");
        RegisterBinding("CopyLink", Key.C, ModifierKeys.Control, "Copy post link");
        RegisterBinding("ViewThread", Key.T, ModifierKeys.None, "View full thread");
        RegisterBinding("ViewProfile", Key.P, ModifierKeys.None, "View author profile");

        // Compose
        RegisterBinding("NewPost", Key.N, ModifierKeys.None, "New post");
        RegisterBinding("NewPostCtrl", Key.N, ModifierKeys.Control, "New post (alternate)");

        // Refresh
        RegisterBinding("Refresh", Key.R, ModifierKeys.Control, "Refresh current timeline");
        RegisterBinding("RefreshAll", Key.R, ModifierKeys.Control | ModifierKeys.Shift, "Refresh all timelines");

        // Media
        RegisterBinding("PlayMedia", Key.Enter, ModifierKeys.None, "Play/view media");
        RegisterBinding("NextMedia", Key.Right, ModifierKeys.None, "Next media item");
        RegisterBinding("PreviousMedia", Key.Left, ModifierKeys.None, "Previous media item");

        // Accessibility
        RegisterBinding("AnnouncePost", Key.Space, ModifierKeys.None, "Read current post");
        RegisterBinding("StopSpeech", Key.Escape, ModifierKeys.None, "Stop speech");

        // Application
        RegisterBinding("Search", Key.S, ModifierKeys.Control, "Search");
        RegisterBinding("Settings", Key.OemComma, ModifierKeys.Control, "Open settings");
        RegisterBinding("Help", Key.F1, ModifierKeys.None, "Show help");
        RegisterBinding("Quit", Key.Q, ModifierKeys.Control, "Quit application");

        Log.Information("Loaded {Count} default key bindings", _bindings.Count);
    }

    /// <summary>
    /// Register a key binding
    /// </summary>
    public void RegisterBinding(string action, Key key, ModifierKeys modifiers, string description)
    {
        _bindings[action] = new KeyBinding
        {
            Key = key,
            Modifiers = modifiers,
            Description = description
        };
    }

    /// <summary>
    /// Register an action handler
    /// </summary>
    public void RegisterAction(string action, Action handler)
    {
        _actions[action] = handler;
    }

    /// <summary>
    /// Handle a key press
    /// </summary>
    public bool HandleKeyPress(Key key, ModifierKeys modifiers)
    {
        foreach (var (action, binding) in _bindings)
        {
            if (binding.Key == key && binding.Modifiers == modifiers)
            {
                if (_actions.TryGetValue(action, out var handler))
                {
                    Log.Debug("Executing action: {Action}", action);
                    handler();
                    return true;
                }
            }
        }

        return false;
    }

    /// <summary>
    /// Get all bindings
    /// </summary>
    public IReadOnlyDictionary<string, KeyBinding> GetAllBindings() => _bindings;

    /// <summary>
    /// Get binding for an action
    /// </summary>
    public KeyBinding? GetBinding(string action)
    {
        return _bindings.TryGetValue(action, out var binding) ? binding : null;
    }

    /// <summary>
    /// Update a binding
    /// </summary>
    public void UpdateBinding(string action, Key key, ModifierKeys modifiers)
    {
        if (_bindings.TryGetValue(action, out var binding))
        {
            binding.Key = key;
            binding.Modifiers = modifiers;
            Log.Information("Updated binding for {Action} to {Binding}", action, binding);
        }
    }

    /// <summary>
    /// Reset all bindings to defaults
    /// </summary>
    public void ResetToDefaults()
    {
        _bindings.Clear();
        LoadDefaultBindings();
    }

    /// <summary>
    /// Export bindings to JSON
    /// </summary>
    public string ExportBindings()
    {
        var exportData = _bindings.ToDictionary(
            kvp => kvp.Key,
            kvp => new { Key = kvp.Value.Key.ToString(), Modifiers = kvp.Value.Modifiers.ToString() }
        );

        return System.Text.Json.JsonSerializer.Serialize(exportData, new System.Text.Json.JsonSerializerOptions { WriteIndented = true });
    }

    /// <summary>
    /// Import bindings from JSON
    /// </summary>
    public void ImportBindings(string json)
    {
        try
        {
            var importData = System.Text.Json.JsonSerializer.Deserialize<Dictionary<string, BindingData>>(json);

            if (importData == null)
                return;

            foreach (var (action, data) in importData)
            {
                if (_bindings.TryGetValue(action, out var binding))
                {
                    if (Enum.TryParse<Key>(data.Key, out var key) &&
                        Enum.TryParse<ModifierKeys>(data.Modifiers, out var modifiers))
                    {
                        binding.Key = key;
                        binding.Modifiers = modifiers;
                    }
                }
            }

            Log.Information("Imported key bindings");
        }
        catch (Exception ex)
        {
            Log.Warning(ex, "Failed to import key bindings");
        }
    }

    private class BindingData
    {
        public string Key { get; set; } = "";
        public string Modifiers { get; set; } = "";
    }
}

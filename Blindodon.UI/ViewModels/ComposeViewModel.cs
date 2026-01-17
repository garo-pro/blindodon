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

using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Serilog;

namespace Blindodon.ViewModels;

/// <summary>
/// View model for the compose window
/// </summary>
public partial class ComposeViewModel : ObservableObject
{
    [ObservableProperty]
    private string _content = "";

    [ObservableProperty]
    private string _spoilerText = "";

    [ObservableProperty]
    private string _visibility = "public";

    [ObservableProperty]
    private bool _sensitive;

    [ObservableProperty]
    private string? _inReplyToId;

    [ObservableProperty]
    private PostViewModel? _replyingTo;

    [ObservableProperty]
    private int _characterCount;

    [ObservableProperty]
    private int _maxCharacters = 500;

    [ObservableProperty]
    private bool _isPosting;

    [ObservableProperty]
    private string _statusMessage = "";

    public ObservableCollection<MediaAttachmentViewModel> Attachments { get; } = new();

    /// <summary>
    /// Gets whether posting is allowed
    /// </summary>
    public bool CanPost => Content.Length > 0 && Content.Length <= MaxCharacters && !IsPosting;

    /// <summary>
    /// Gets the remaining character count
    /// </summary>
    public int RemainingCharacters => MaxCharacters - CharacterCount;

    /// <summary>
    /// Gets whether we are replying to a post
    /// </summary>
    public bool IsReply => ReplyingTo != null;

    /// <summary>
    /// Event raised when posting is complete
    /// </summary>
    public event EventHandler<bool>? PostComplete;

    /// <summary>
    /// Event raised to request window close
    /// </summary>
    public event EventHandler? RequestClose;

    /// <summary>
    /// Visibility options for the dropdown
    /// </summary>
    public static List<VisibilityOption> VisibilityOptions { get; } = new()
    {
        new VisibilityOption("public", "Public", "Visible to everyone"),
        new VisibilityOption("unlisted", "Unlisted", "Visible but not in public timelines"),
        new VisibilityOption("private", "Followers only", "Only visible to followers"),
        new VisibilityOption("direct", "Direct", "Only visible to mentioned users")
    };

    partial void OnContentChanged(string value)
    {
        CharacterCount = value.Length;
        OnPropertyChanged(nameof(CanPost));
        OnPropertyChanged(nameof(RemainingCharacters));

        // Announce character count at thresholds for accessibility
        var remaining = RemainingCharacters;
        if (remaining == 50 || remaining == 20 || remaining == 10 || remaining == 0)
        {
            App.Accessibility.AnnounceCharacterCount(remaining, MaxCharacters);
        }
        else if (remaining < 0)
        {
            App.Accessibility.Announce($"Over limit by {-remaining} characters");
        }
    }

    [RelayCommand]
    private async Task Post()
    {
        if (!CanPost) return;

        IsPosting = true;
        StatusMessage = "Posting...";

        try
        {
            var result = await App.Bridge.SendRequestAsync("post.create", new
            {
                content = Content,
                spoiler_text = string.IsNullOrEmpty(SpoilerText) ? null : SpoilerText,
                visibility = Visibility,
                sensitive = Sensitive,
                in_reply_to_id = InReplyToId,
                media_ids = Attachments.Select(a => a.Id).ToList()
            });

            if (result != null)
            {
                Log.Information("Post created successfully");
                App.Audio.Play(Services.AudioManager.SoundEvent.PostSent);
                App.Accessibility.Announce("Post sent successfully");
                PostComplete?.Invoke(this, true);
                RequestClose?.Invoke(this, EventArgs.Empty);
            }
            else
            {
                StatusMessage = "Failed to create post";
                App.Audio.Play(Services.AudioManager.SoundEvent.Error);
                App.Accessibility.Announce("Failed to send post");
                PostComplete?.Invoke(this, false);
            }
        }
        catch (Exception ex)
        {
            Log.Error(ex, "Failed to create post");
            StatusMessage = $"Error: {ex.Message}";
            App.Audio.Play(Services.AudioManager.SoundEvent.Error);
            PostComplete?.Invoke(this, false);
        }
        finally
        {
            IsPosting = false;
        }
    }

    [RelayCommand]
    private void Cancel()
    {
        RequestClose?.Invoke(this, EventArgs.Empty);
    }

    [RelayCommand]
    private void CycleVisibility()
    {
        var currentIndex = VisibilityOptions.FindIndex(v => v.Value == Visibility);
        var nextIndex = (currentIndex + 1) % VisibilityOptions.Count;
        Visibility = VisibilityOptions[nextIndex].Value;

        var option = VisibilityOptions[nextIndex];
        App.Accessibility.Announce($"Visibility: {option.Label}");
    }

    [RelayCommand]
    private void RemoveAttachment(MediaAttachmentViewModel? attachment)
    {
        if (attachment == null) return;

        Attachments.Remove(attachment);
        App.Accessibility.Announce($"Removed attachment. {Attachments.Count} attachments remaining.");
    }

    /// <summary>
    /// Set up the compose window for a reply
    /// </summary>
    public void SetupReply(PostViewModel post)
    {
        ReplyingTo = post;
        InReplyToId = post.Id;

        // Pre-fill with mention
        Content = $"@{post.Account.Acct} ";
        OnPropertyChanged(nameof(IsReply));
    }
}

/// <summary>
/// Visibility option for the dropdown
/// </summary>
public class VisibilityOption
{
    public string Value { get; }
    public string Label { get; }
    public string Description { get; }

    public VisibilityOption(string value, string label, string description)
    {
        Value = value;
        Label = label;
        Description = description;
    }

    public override string ToString() => Label;
}

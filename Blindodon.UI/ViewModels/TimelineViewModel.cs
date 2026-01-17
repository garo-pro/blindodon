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
using Newtonsoft.Json.Linq;
using Serilog;

namespace Blindodon.ViewModels;

/// <summary>
/// View model for a single timeline with its own state
/// </summary>
public partial class TimelineViewModel : ObservableObject
{
    [ObservableProperty]
    private string _name = "";

    [ObservableProperty]
    private string _type = ""; // home, local, federated, notifications

    [ObservableProperty]
    private bool _isLoading;

    [ObservableProperty]
    private int _unreadCount;

    [ObservableProperty]
    private PostViewModel? _selectedPost;

    [ObservableProperty]
    private string? _oldestPostId;

    [ObservableProperty]
    private string? _newestPostId;

    [ObservableProperty]
    private bool _hasMore = true;

    public ObservableCollection<PostViewModel> Posts { get; } = new();

    /// <summary>
    /// Gets whether this timeline has been loaded at least once
    /// </summary>
    public bool IsLoaded => Posts.Count > 0 || !HasMore;

    /// <summary>
    /// Load the timeline
    /// </summary>
    public async Task LoadAsync(int limit = 20)
    {
        if (IsLoading) return;

        IsLoading = true;

        try
        {
            var result = await App.Bridge.SendRequestAsync("timeline.get", new
            {
                timeline_type = Type,
                limit = limit
            });

            if (result != null)
            {
                var posts = result["posts"]?.ToObject<List<JObject>>();
                if (posts != null)
                {
                    Posts.Clear();
                    foreach (var post in posts)
                    {
                        Posts.Add(PostViewModel.FromJson(post));
                    }

                    UpdatePaginationInfo();

                    if (Posts.Count > 0)
                    {
                        SelectedPost = Posts[0];
                    }
                }

                HasMore = result["has_more"]?.Value<bool>() ?? false;
            }

            // Reset unread count when loading fresh
            UnreadCount = 0;
        }
        catch (Exception ex)
        {
            Log.Error(ex, "Failed to load timeline {Type}", Type);
        }
        finally
        {
            IsLoading = false;
        }
    }

    /// <summary>
    /// Load older posts (pagination)
    /// </summary>
    public async Task LoadOlderAsync(int limit = 20)
    {
        if (IsLoading || !HasMore || string.IsNullOrEmpty(OldestPostId)) return;

        IsLoading = true;

        try
        {
            var result = await App.Bridge.SendRequestAsync("timeline.get", new
            {
                timeline_type = Type,
                max_id = OldestPostId,
                limit = limit
            });

            if (result != null)
            {
                var posts = result["posts"]?.ToObject<List<JObject>>();
                if (posts != null && posts.Count > 0)
                {
                    foreach (var post in posts)
                    {
                        Posts.Add(PostViewModel.FromJson(post));
                    }

                    UpdatePaginationInfo();
                }

                HasMore = result["has_more"]?.Value<bool>() ?? false;
            }
        }
        catch (Exception ex)
        {
            Log.Error(ex, "Failed to load older posts for timeline {Type}", Type);
        }
        finally
        {
            IsLoading = false;
        }
    }

    /// <summary>
    /// Load newer posts (refresh)
    /// </summary>
    public async Task LoadNewerAsync(int limit = 20)
    {
        if (IsLoading || string.IsNullOrEmpty(NewestPostId)) return;

        IsLoading = true;

        try
        {
            var result = await App.Bridge.SendRequestAsync("timeline.get", new
            {
                timeline_type = Type,
                since_id = NewestPostId,
                limit = limit
            });

            if (result != null)
            {
                var posts = result["posts"]?.ToObject<List<JObject>>();
                if (posts != null && posts.Count > 0)
                {
                    // Insert at the beginning
                    for (int i = posts.Count - 1; i >= 0; i--)
                    {
                        Posts.Insert(0, PostViewModel.FromJson(posts[i]));
                    }

                    UpdatePaginationInfo();
                }
            }
        }
        catch (Exception ex)
        {
            Log.Error(ex, "Failed to load newer posts for timeline {Type}", Type);
        }
        finally
        {
            IsLoading = false;
        }
    }

    /// <summary>
    /// Insert a new post at the beginning (from streaming)
    /// </summary>
    public void InsertNewPost(PostViewModel post)
    {
        Posts.Insert(0, post);
        NewestPostId = post.Id;
        UnreadCount++;
    }

    /// <summary>
    /// Remove a post by ID
    /// </summary>
    public void RemovePost(string postId)
    {
        var post = Posts.FirstOrDefault(p => p.Id == postId);
        if (post != null)
        {
            Posts.Remove(post);
            UpdatePaginationInfo();
        }
    }

    private void UpdatePaginationInfo()
    {
        if (Posts.Count > 0)
        {
            NewestPostId = Posts[0].Id;
            OldestPostId = Posts[^1].Id;
        }
        else
        {
            NewestPostId = null;
            OldestPostId = null;
        }
    }
}

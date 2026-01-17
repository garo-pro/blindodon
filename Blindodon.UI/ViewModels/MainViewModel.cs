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
using System.Windows;
using System.Windows.Input;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Newtonsoft.Json.Linq;
using Serilog;

namespace Blindodon.ViewModels;

/// <summary>
/// Main view model for the application
/// </summary>
public partial class MainViewModel : ObservableObject
{
    [ObservableProperty]
    private bool _isLoggedIn;

    [ObservableProperty]
    private bool _isLoading;

    [ObservableProperty]
    private bool _isConnected;

    [ObservableProperty]
    private string _instanceUrl = "";

    [ObservableProperty]
    private string _loginStatus = "";

    [ObservableProperty]
    private string _statusMessage = "Ready";

    [ObservableProperty]
    private string _currentTimeline = "Home";

    [ObservableProperty]
    private PostViewModel? _selectedPost;

    [ObservableProperty]
    private UserViewModel? _currentUser;

    public ObservableCollection<PostViewModel> Posts { get; } = new();

    public ObservableCollection<NotificationViewModel> Notifications { get; } = new();

    /// <summary>
    /// Dictionary of timeline view models for caching
    /// </summary>
    public Dictionary<string, TimelineViewModel> Timelines { get; } = new();

    /// <summary>
    /// Gets the current timeline view model
    /// </summary>
    public TimelineViewModel? CurrentTimelineViewModel =>
        Timelines.TryGetValue(CurrentTimeline, out var vm) ? vm : null;

    public MainViewModel()
    {
        // Initialize standard timelines
        Timelines["Home"] = new TimelineViewModel { Name = "Home", Type = "home" };
        Timelines["Local"] = new TimelineViewModel { Name = "Local", Type = "local" };
        Timelines["Federated"] = new TimelineViewModel { Name = "Federated", Type = "federated" };

        // Subscribe to bridge events
        App.Bridge.EventReceived += Bridge_EventReceived;
        App.Bridge.ConnectionStateChanged += Bridge_ConnectionStateChanged;
    }

    public async Task InitializeAsync()
    {
        IsLoading = true;
        StatusMessage = "Connecting to backend...";

        try
        {
            // Try to connect to the Rust core
            var connected = await App.Bridge.ConnectAsync();
            IsConnected = connected;

            if (connected)
            {
                StatusMessage = "Connected";
                Log.Information("Connected to Rust core");

                // Check if we have saved credentials and restored session
                var authResult = await App.Bridge.SendRequestAsync("auth.get_accounts", null);
                if (authResult != null)
                {
                    var isAuthenticated = authResult["authenticated"]?.Value<bool>() == true;
                    var currentAccountId = authResult["current_account_id"]?.Value<string>();
                    var accounts = authResult["accounts"]?.ToObject<List<Newtonsoft.Json.Linq.JObject>>() ?? new();

                    if (isAuthenticated && !string.IsNullOrEmpty(currentAccountId))
                    {
                        // Session was restored automatically by Rust backend
                        IsLoggedIn = true;
                        Log.Information("Session restored for account: {AccountId}", currentAccountId);

                        // Find current account info to get display name and instance
                        var currentAccount = accounts.FirstOrDefault(a =>
                            a["id"]?.Value<string>() == currentAccountId);

                        if (currentAccount != null)
                        {
                            InstanceUrl = currentAccount["instance_url"]?.Value<string>() ?? "";
                            var displayName = currentAccount["display_name"]?.Value<string>();
                            if (string.IsNullOrEmpty(displayName))
                            {
                                displayName = currentAccount["username"]?.Value<string>() ?? "";
                            }
                            App.Audio.Play(Services.AudioManager.SoundEvent.Connected);
                            App.Accessibility.Announce($"Welcome back, {displayName}");
                        }

                        await LoadTimelineAsync();
                    }
                    else if (accounts.Count > 0)
                    {
                        // Have saved accounts but no active session (token may have expired)
                        Log.Information("Found {Count} saved accounts but no active session", accounts.Count);
                        StatusMessage = "Please log in to continue";
                    }
                }
            }
            else
            {
                StatusMessage = "Running in offline mode";
                Log.Warning("Could not connect to Rust core, running in UI-only mode");
            }
        }
        catch (Exception ex)
        {
            Log.Error(ex, "Initialization failed");
            StatusMessage = "Initialization failed";
        }
        finally
        {
            IsLoading = false;
        }
    }

    [RelayCommand]
    private async Task Login()
    {
        if (string.IsNullOrWhiteSpace(InstanceUrl))
        {
            LoginStatus = "Please enter your instance URL";
            return;
        }

        IsLoading = true;
        LoginStatus = "Starting authentication...";

        try
        {
            var result = await App.Bridge.SendRequestAsync("auth.start", new { instance_url = InstanceUrl });

            if (result != null)
            {
                var authUrl = result["auth_url"]?.Value<string>();
                if (!string.IsNullOrEmpty(authUrl))
                {
                    LoginStatus = "Opening browser for authorization...";

                    // Open the auth URL in the default browser
                    System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo
                    {
                        FileName = authUrl,
                        UseShellExecute = true
                    });

                    // Show dialog to enter the authorization code
                    var code = await ShowAuthCodeDialogAsync();

                    if (!string.IsNullOrEmpty(code))
                    {
                        LoginStatus = "Completing authentication...";

                        var callbackResult = await App.Bridge.SendRequestAsync("auth.callback", new
                        {
                            instance_url = InstanceUrl,
                            code = code
                        });

                        if (callbackResult != null && callbackResult["success"]?.Value<bool>() == true)
                        {
                            IsLoggedIn = true;
                            LoginStatus = "";
                            StatusMessage = "Logged in successfully";

                            App.Audio.Play(Services.AudioManager.SoundEvent.Connected);
                            App.Accessibility.Announce("Login successful. Loading your timeline.");

                            await LoadTimelineAsync();
                        }
                        else
                        {
                            LoginStatus = "Authentication failed. Please try again.";
                        }
                    }
                    else
                    {
                        LoginStatus = "Authentication cancelled";
                    }
                }
            }
            else
            {
                LoginStatus = "Failed to start authentication";
            }
        }
        catch (Exception ex)
        {
            Log.Error(ex, "Login failed");
            LoginStatus = $"Error: {ex.Message}";
        }
        finally
        {
            IsLoading = false;
        }
    }

    private Task<string?> ShowAuthCodeDialogAsync()
    {
        // Simple input dialog for the authorization code
        var tcs = new TaskCompletionSource<string?>();

        Application.Current.Dispatcher.Invoke(() =>
        {
            var dialog = new Window
            {
                Title = "Enter Authorization Code",
                Width = 400,
                Height = 200,
                WindowStartupLocation = WindowStartupLocation.CenterOwner,
                Owner = Application.Current.MainWindow,
                Background = Application.Current.Resources["BackgroundBrush"] as System.Windows.Media.Brush,
                Foreground = Application.Current.Resources["TextPrimaryBrush"] as System.Windows.Media.Brush
            };

            var panel = new System.Windows.Controls.StackPanel
            {
                Margin = new Thickness(20)
            };

            var label = new System.Windows.Controls.TextBlock
            {
                Text = "After authorizing in your browser, paste the code here:",
                Margin = new Thickness(0, 0, 0, 10),
                TextWrapping = TextWrapping.Wrap
            };

            var textBox = new System.Windows.Controls.TextBox
            {
                Margin = new Thickness(0, 0, 0, 10)
            };
            System.Windows.Automation.AutomationProperties.SetName(textBox, "Authorization code");

            var buttonPanel = new System.Windows.Controls.StackPanel
            {
                Orientation = System.Windows.Controls.Orientation.Horizontal,
                HorizontalAlignment = HorizontalAlignment.Right
            };

            var okButton = new System.Windows.Controls.Button
            {
                Content = "Submit",
                Padding = new Thickness(16, 8, 16, 8),
                Margin = new Thickness(0, 0, 8, 0)
            };
            okButton.Click += (s, e) =>
            {
                tcs.SetResult(textBox.Text);
                dialog.Close();
            };

            var cancelButton = new System.Windows.Controls.Button
            {
                Content = "Cancel",
                Padding = new Thickness(16, 8, 16, 8)
            };
            cancelButton.Click += (s, e) =>
            {
                tcs.SetResult(null);
                dialog.Close();
            };

            buttonPanel.Children.Add(okButton);
            buttonPanel.Children.Add(cancelButton);

            panel.Children.Add(label);
            panel.Children.Add(textBox);
            panel.Children.Add(buttonPanel);

            dialog.Content = panel;
            dialog.ShowDialog();
        });

        return tcs.Task;
    }

    private async Task LoadTimelineAsync()
    {
        // Use specialized notification loading for Notifications timeline
        if (CurrentTimeline.Equals("Notifications", StringComparison.OrdinalIgnoreCase))
        {
            await LoadNotificationsAsync();
            return;
        }

        var vm = CurrentTimelineViewModel;
        if (vm == null) return;

        IsLoading = true;
        StatusMessage = $"Loading {CurrentTimeline} timeline...";

        try
        {
            // Use cached timeline if already loaded
            if (!vm.IsLoaded)
            {
                await vm.LoadAsync();
            }

            // Sync the Posts collection for backward compatibility with MainWindow.xaml bindings
            Posts.Clear();
            foreach (var post in vm.Posts)
            {
                Posts.Add(post);
            }

            var unreadText = vm.UnreadCount > 0 ? $", {vm.UnreadCount} new" : "";
            StatusMessage = $"Loaded {vm.Posts.Count} posts{unreadText}";

            if (vm.Posts.Count > 0 && !vm.IsLoaded)
            {
                App.Audio.Play(Services.AudioManager.SoundEvent.NewPost);
            }

            App.Accessibility.Announce($"{CurrentTimeline} timeline. {vm.Posts.Count} posts{unreadText}");

            if (vm.Posts.Count > 0)
            {
                SelectedPost = vm.SelectedPost ?? vm.Posts[0];
            }

            // Clear unread count when viewing
            vm.UnreadCount = 0;
        }
        catch (Exception ex)
        {
            Log.Error(ex, "Failed to load timeline");
            StatusMessage = "Failed to load timeline";
        }
        finally
        {
            IsLoading = false;
        }
    }

    private async Task LoadNotificationsAsync()
    {
        IsLoading = true;
        StatusMessage = "Loading notifications...";

        try
        {
            var result = await App.Bridge.SendRequestAsync("notifications.get", new
            {
                limit = 30
            });

            if (result != null)
            {
                var notifications = result["notifications"]?.ToObject<List<JObject>>();
                if (notifications != null)
                {
                    Notifications.Clear();
                    foreach (var notification in notifications)
                    {
                        Notifications.Add(NotificationViewModel.FromJson(notification));
                    }

                    StatusMessage = $"Loaded {Notifications.Count} notifications";
                    App.Audio.Play(Services.AudioManager.SoundEvent.NewNotification);
                    App.Accessibility.Announce($"Loaded {Notifications.Count} notifications");
                }
            }
        }
        catch (Exception ex)
        {
            Log.Error(ex, "Failed to load notifications");
            StatusMessage = "Failed to load notifications";
            App.Accessibility.Announce("Failed to load notifications");
        }
        finally
        {
            IsLoading = false;
        }
    }

    [RelayCommand]
    private async Task ClearNotifications()
    {
        try
        {
            var result = await App.Bridge.SendRequestAsync("notifications.clear", null);
            if (result != null && result["success"]?.Value<bool>() == true)
            {
                Notifications.Clear();
                App.Accessibility.Announce("All notifications cleared");
            }
        }
        catch (Exception ex)
        {
            Log.Error(ex, "Failed to clear notifications");
            App.Audio.Play(Services.AudioManager.SoundEvent.Error);
        }
    }

    [RelayCommand]
    private async Task DismissNotification(NotificationViewModel? notification)
    {
        if (notification == null) return;

        try
        {
            var result = await App.Bridge.SendRequestAsync("notifications.dismiss", new
            {
                notification_id = notification.Id
            });

            if (result != null && result["success"]?.Value<bool>() == true)
            {
                Notifications.Remove(notification);
                App.Accessibility.Announce("Notification dismissed");
            }
        }
        catch (Exception ex)
        {
            Log.Error(ex, "Failed to dismiss notification");
            App.Audio.Play(Services.AudioManager.SoundEvent.Error);
        }
    }

    [RelayCommand]
    private async Task SwitchTimeline(string timeline)
    {
        CurrentTimeline = timeline;
        await LoadTimelineAsync();
    }

    [RelayCommand]
    private async Task Refresh()
    {
        // Force reload current timeline
        var vm = CurrentTimelineViewModel;
        if (vm != null)
        {
            vm.Posts.Clear();
        }
        await LoadTimelineAsync();
    }

    [RelayCommand]
    private void NewPost()
    {
        var window = new Views.ComposeWindow();
        window.Owner = System.Windows.Application.Current.MainWindow;
        window.ShowDialog();
        Log.Information("Compose window closed");
    }

    [RelayCommand]
    private void Reply(PostViewModel? post)
    {
        if (post == null) return;

        var window = new Views.ComposeWindow(replyTo: post);
        window.Owner = System.Windows.Application.Current.MainWindow;
        window.ShowDialog();
        Log.Information("Reply window closed for post {PostId}", post.Id);
    }

    [RelayCommand]
    private async Task Boost(PostViewModel? post)
    {
        if (post == null) return;

        try
        {
            var result = await App.Bridge.SendRequestAsync("post.boost", new { post_id = post.Id });
            if (result != null)
            {
                post.Reblogged = true;
                post.ReblogsCount++;
                App.Audio.Play(Services.AudioManager.SoundEvent.BoostSent);
                App.Accessibility.Announce("Post boosted");
            }
        }
        catch (Exception ex)
        {
            Log.Error(ex, "Failed to boost post");
            App.Audio.Play(Services.AudioManager.SoundEvent.Error);
        }
    }

    [RelayCommand]
    private async Task Favorite(PostViewModel? post)
    {
        if (post == null) return;

        try
        {
            var result = await App.Bridge.SendRequestAsync("post.favourite", new { post_id = post.Id });
            if (result != null)
            {
                post.Favourited = true;
                post.FavouritesCount++;
                App.Audio.Play(Services.AudioManager.SoundEvent.FavoriteAdded);
                App.Accessibility.Announce("Post favorited");
            }
        }
        catch (Exception ex)
        {
            Log.Error(ex, "Failed to favorite post");
            App.Audio.Play(Services.AudioManager.SoundEvent.Error);
        }
    }

    [RelayCommand]
    private void ViewThread(PostViewModel? post)
    {
        if (post == null) return;

        // TODO: Open thread view
        Log.Information("View thread for post {PostId}", post.Id);
    }

    [RelayCommand]
    private void OpenSettings()
    {
        // TODO: Open settings window
        App.Accessibility.Announce("Settings. Feature coming soon.");
    }

    [RelayCommand]
    private void Quit()
    {
        Application.Current.Shutdown();
    }

    private void Bridge_EventReceived(object? sender, Services.IpcEvent e)
    {
        Application.Current.Dispatcher.Invoke(() =>
        {
            switch (e.EventType)
            {
                case "event.new_post":
                    HandleNewPost(e.Data);
                    break;

                case "event.new_notification":
                    HandleNewNotification(e.Data);
                    break;

                case "event.post_deleted":
                    HandlePostDeleted(e.Data);
                    break;
            }
        });
    }

    private void HandleNewPost(JObject? data)
    {
        if (data == null) return;

        var postData = data["post"]?.ToObject<JObject>();
        var timelineType = data["timeline"]?.Value<string>() ?? "home";

        if (postData != null)
        {
            var post = PostViewModel.FromJson(postData);

            // Map timeline type to our names
            var timelineName = timelineType switch
            {
                "home" => "Home",
                "local" => "Local",
                "public" or "federated" => "Federated",
                _ => "Home"
            };

            // Route to the correct timeline
            if (Timelines.TryGetValue(timelineName, out var vm))
            {
                vm.InsertNewPost(post);

                // If this is the active timeline, also update the Posts collection
                if (CurrentTimeline == timelineName)
                {
                    Posts.Insert(0, post);
                    App.Audio.Play(Services.AudioManager.SoundEvent.NewPost);
                    App.Accessibility.Announce($"New post from {post.Account.DisplayName}");
                }
            }
        }
    }

    private void HandleNewNotification(JObject? data)
    {
        if (data == null) return;

        App.Audio.Play(Services.AudioManager.SoundEvent.NewNotification);

        var notificationType = data["notification"]?["type"]?.Value<string>() ?? "unknown";
        var from = data["notification"]?["account"]?["display_name"]?.Value<string>() ?? "Someone";

        App.Accessibility.AnnounceNotification(notificationType, from);
    }

    private void HandlePostDeleted(JObject? data)
    {
        if (data == null) return;

        var postId = data["post_id"]?.Value<string>();
        if (!string.IsNullOrEmpty(postId))
        {
            // Remove from all timelines
            foreach (var vm in Timelines.Values)
            {
                vm.RemovePost(postId);
            }

            // Also remove from the main Posts collection
            var post = Posts.FirstOrDefault(p => p.Id == postId);
            if (post != null)
            {
                Posts.Remove(post);
            }
        }
    }

    private void Bridge_ConnectionStateChanged(object? sender, bool connected)
    {
        Application.Current.Dispatcher.Invoke(() =>
        {
            IsConnected = connected;
            StatusMessage = connected ? "Connected" : "Disconnected";

            if (connected)
            {
                App.Audio.Play(Services.AudioManager.SoundEvent.Connected);
            }
            else
            {
                App.Audio.Play(Services.AudioManager.SoundEvent.Disconnected);
            }
        });
    }
}

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

using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using Blindodon.ViewModels;
using Serilog;

namespace Blindodon.Views;

/// <summary>
/// Main window for Blindodon
/// </summary>
public partial class MainWindow : Window
{
    private readonly MainViewModel _viewModel;

    public MainWindow()
    {
        InitializeComponent();

        _viewModel = new MainViewModel();
        DataContext = _viewModel;

        // Set up keyboard navigation
        SetupKeyboardNavigation();
    }

    private async void Window_Loaded(object sender, RoutedEventArgs e)
    {
        Log.Information("Main window loaded");

        // Announce to screen reader
        App.Accessibility.Announce("Blindodon is ready. Press Tab to navigate, or N to compose a new post.");

        // Try to connect to the Rust core
        await _viewModel.InitializeAsync();

        // Focus the appropriate element
        if (_viewModel.IsLoggedIn)
        {
            TimelineListBox.Focus();
        }
        else
        {
            InstanceUrlTextBox.Focus();
        }
    }

    private void SetupKeyboardNavigation()
    {
        // Register keybinding actions
        App.Keybindings.RegisterAction("NextPost", () => NavigatePost(1));
        App.Keybindings.RegisterAction("PreviousPost", () => NavigatePost(-1));
        App.Keybindings.RegisterAction("AnnouncePost", AnnounceCurrentPost);
        App.Keybindings.RegisterAction("StopSpeech", () => App.Accessibility.StopAnnouncing());
        App.Keybindings.RegisterAction("HomeTimeline", () => SwitchTimeline("Home"));
        App.Keybindings.RegisterAction("LocalTimeline", () => SwitchTimeline("Local"));
        App.Keybindings.RegisterAction("FederatedTimeline", () => SwitchTimeline("Federated"));
        App.Keybindings.RegisterAction("Notifications", () => SwitchTimeline("Notifications"));
    }

    private void Window_KeyDown(object sender, KeyEventArgs e)
    {
        // Don't handle keys when typing in a text box
        if (e.OriginalSource is TextBox)
            return;

        var modifiers = Keyboard.Modifiers;

        if (App.Keybindings.HandleKeyPress(e.Key, modifiers))
        {
            e.Handled = true;
        }
    }

    private void TimelineListBox_KeyDown(object sender, KeyEventArgs e)
    {
        switch (e.Key)
        {
            case Key.J:
                NavigatePost(1);
                e.Handled = true;
                break;

            case Key.K:
                NavigatePost(-1);
                e.Handled = true;
                break;

            case Key.Space:
                AnnounceCurrentPost();
                e.Handled = true;
                break;

            case Key.R when Keyboard.Modifiers == ModifierKeys.None:
                _viewModel.ReplyCommand.Execute(_viewModel.SelectedPost);
                e.Handled = true;
                break;

            case Key.B:
                _viewModel.BoostCommand.Execute(_viewModel.SelectedPost);
                e.Handled = true;
                break;

            case Key.F:
                _viewModel.FavoriteCommand.Execute(_viewModel.SelectedPost);
                e.Handled = true;
                break;

            case Key.Enter:
                // View full post/thread
                _viewModel.ViewThreadCommand.Execute(_viewModel.SelectedPost);
                e.Handled = true;
                break;
        }
    }

    private void TimelineListBox_SelectionChanged(object sender, SelectionChangedEventArgs e)
    {
        if (TimelineListBox.SelectedItem is PostViewModel post)
        {
            // Announce the selected post for screen readers
            var announcement = $"Post by {post.Account.DisplayName}";

            if (!string.IsNullOrEmpty(post.SpoilerText))
            {
                announcement += $", Content warning: {post.SpoilerText}";
            }
            else
            {
                announcement += $": {post.PlainContent}";
            }

            announcement += $". {post.ReblogsCount} boosts, {post.FavouritesCount} favorites.";

            App.Accessibility.Announce(announcement);
        }
    }

    private void NotificationsListBox_KeyDown(object sender, KeyEventArgs e)
    {
        switch (e.Key)
        {
            case Key.J:
                NavigateNotification(1);
                e.Handled = true;
                break;

            case Key.K:
                NavigateNotification(-1);
                e.Handled = true;
                break;

            case Key.Space:
                AnnounceCurrentNotification();
                e.Handled = true;
                break;

            case Key.D:
            case Key.Delete:
                // Dismiss notification
                if (NotificationsListBox.SelectedItem is NotificationViewModel notification)
                {
                    _viewModel.DismissNotificationCommand.Execute(notification);
                }
                e.Handled = true;
                break;

            case Key.Enter:
                // If the notification has an associated post, view it
                if (NotificationsListBox.SelectedItem is NotificationViewModel notif && notif.Status != null)
                {
                    _viewModel.ViewThreadCommand.Execute(notif.Status);
                }
                e.Handled = true;
                break;
        }
    }

    private void NavigateNotification(int direction)
    {
        var currentIndex = NotificationsListBox.SelectedIndex;
        var newIndex = currentIndex + direction;

        if (newIndex < 0)
        {
            App.Audio.Play(Services.AudioManager.SoundEvent.BoundaryReached);
            App.Accessibility.AnnounceBoundary(isTop: true);
            return;
        }

        if (newIndex >= NotificationsListBox.Items.Count)
        {
            App.Audio.Play(Services.AudioManager.SoundEvent.BoundaryReached);
            App.Accessibility.AnnounceBoundary(isTop: false);
            return;
        }

        NotificationsListBox.SelectedIndex = newIndex;
        NotificationsListBox.ScrollIntoView(NotificationsListBox.SelectedItem);

        // Announce the notification
        if (NotificationsListBox.SelectedItem is NotificationViewModel notification)
        {
            var announcement = notification.DisplayText;
            if (notification.HasStatus && notification.Status != null)
            {
                announcement += $": {notification.Status.PlainContent}";
            }
            App.Accessibility.Announce(announcement);
        }
    }

    private void AnnounceCurrentNotification()
    {
        if (NotificationsListBox.SelectedItem is NotificationViewModel notification)
        {
            var announcement = notification.DisplayText;
            if (notification.HasStatus && notification.Status != null)
            {
                announcement += $": {notification.Status.PlainContent}";
            }
            announcement += $". {notification.RelativeTime}";
            App.Accessibility.Announce(announcement);
        }
    }

    private void NavigatePost(int direction)
    {
        var currentIndex = TimelineListBox.SelectedIndex;
        var newIndex = currentIndex + direction;

        if (newIndex < 0)
        {
            // At the beginning
            App.Audio.Play(Services.AudioManager.SoundEvent.BoundaryReached);
            App.Accessibility.AnnounceBoundary(isTop: true);
            return;
        }

        if (newIndex >= TimelineListBox.Items.Count)
        {
            // At the end
            App.Audio.Play(Services.AudioManager.SoundEvent.BoundaryReached);
            App.Accessibility.AnnounceBoundary(isTop: false);
            return;
        }

        TimelineListBox.SelectedIndex = newIndex;
        TimelineListBox.ScrollIntoView(TimelineListBox.SelectedItem);
    }

    private void AnnounceCurrentPost()
    {
        if (_viewModel.SelectedPost is PostViewModel post)
        {
            App.Accessibility.AnnouncePost(
                post.Account.DisplayName,
                post.PlainContent,
                post.HasMedia,
                post.SpoilerText
            );
        }
    }

    private void SwitchTimeline(string timeline)
    {
        _viewModel.SwitchTimelineCommand.Execute(timeline);

        // Update the tab selection
        switch (timeline)
        {
            case "Home":
                HomeTab.IsChecked = true;
                break;
            case "Local":
                LocalTab.IsChecked = true;
                break;
            case "Federated":
                FederatedTab.IsChecked = true;
                break;
            case "Notifications":
                NotificationsTab.IsChecked = true;
                break;
        }

        App.Accessibility.Announce($"Switched to {timeline} timeline");
    }

    protected override void OnClosed(EventArgs e)
    {
        base.OnClosed(e);
        Application.Current.Shutdown();
    }
}

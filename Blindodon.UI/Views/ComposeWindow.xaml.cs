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
using System.Windows.Input;
using Blindodon.ViewModels;

namespace Blindodon.Views;

/// <summary>
/// Compose window for creating new posts
/// </summary>
public partial class ComposeWindow : Window
{
    private readonly ComposeViewModel _viewModel;

    public ComposeWindow(PostViewModel? replyTo = null)
    {
        InitializeComponent();

        _viewModel = new ComposeViewModel();
        DataContext = _viewModel;

        // Set up reply if provided
        if (replyTo != null)
        {
            _viewModel.SetupReply(replyTo);
        }

        // Subscribe to close request
        _viewModel.RequestClose += (_, _) => Close();
    }

    private void Window_Loaded(object sender, RoutedEventArgs e)
    {
        // Announce for accessibility
        if (_viewModel.IsReply)
        {
            App.Accessibility.Announce($"Compose reply to {_viewModel.ReplyingTo?.Account.DisplayName}. Press Tab to navigate, Ctrl+Enter to send, Escape to cancel.");
        }
        else
        {
            App.Accessibility.Announce("Compose new post. Press Tab to navigate, Ctrl+Enter to send, Escape to cancel.");
        }

        // Focus the content text box
        ContentTextBox.Focus();
        ContentTextBox.CaretIndex = ContentTextBox.Text.Length;
    }

    private void Window_KeyDown(object sender, KeyEventArgs e)
    {
        // V key cycles visibility when not in a text box
        if (e.Key == Key.V && Keyboard.Modifiers == ModifierKeys.None)
        {
            if (!(e.OriginalSource is System.Windows.Controls.TextBox))
            {
                _viewModel.CycleVisibilityCommand.Execute(null);
                e.Handled = true;
            }
        }
    }
}

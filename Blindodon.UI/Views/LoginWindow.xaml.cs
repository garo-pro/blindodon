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
/// Login window for OAuth authentication flow.
/// </summary>
public partial class LoginWindow : Window
{
    private readonly LoginViewModel _viewModel;

    /// <summary>
    /// Gets the account that was successfully logged in, or null if cancelled.
    /// </summary>
    public AccountItemViewModel? LoggedInAccount { get; private set; }

    public LoginWindow()
    {
        InitializeComponent();

        _viewModel = new LoginViewModel();
        DataContext = _viewModel;

        // Subscribe to view model events
        _viewModel.LoginComplete += ViewModel_LoginComplete;
        _viewModel.RequestClose += ViewModel_RequestClose;
        _viewModel.PropertyChanged += ViewModel_PropertyChanged;
    }

    private void Window_Loaded(object sender, RoutedEventArgs e)
    {
        Log.Information("Login window loaded");
        App.Accessibility.Announce("Add account. Enter your Mastodon instance URL.");
        InstanceUrlTextBox.Focus();
    }

    private void Window_KeyDown(object sender, KeyEventArgs e)
    {
        // Don't handle keys when typing in a text box (except Enter/Escape)
        if (e.OriginalSource is TextBox && e.Key != Key.Enter && e.Key != Key.Escape)
            return;

        switch (e.Key)
        {
            case Key.Escape:
                _viewModel.CancelCommand.Execute(null);
                e.Handled = true;
                break;

            case Key.Enter:
                // Handle Enter based on current state
                if (_viewModel.CurrentState == LoginState.EnterInstance && _viewModel.CanStartAuth)
                {
                    _viewModel.StartAuthCommand.Execute(null);
                    e.Handled = true;
                }
                else if (_viewModel.CurrentState == LoginState.WaitingForCode && _viewModel.CanSubmitCode)
                {
                    _viewModel.SubmitCodeCommand.Execute(null);
                    e.Handled = true;
                }
                break;
        }
    }

    private void ViewModel_PropertyChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
    {
        if (e.PropertyName == nameof(LoginViewModel.CurrentState))
        {
            // Focus appropriate control when state changes
            Dispatcher.BeginInvoke(() =>
            {
                switch (_viewModel.CurrentState)
                {
                    case LoginState.EnterInstance:
                        InstanceUrlTextBox.Focus();
                        break;
                    case LoginState.WaitingForCode:
                        AuthCodeTextBox.Focus();
                        break;
                }
            });
        }
    }

    private void ViewModel_LoginComplete(object? sender, AccountItemViewModel account)
    {
        LoggedInAccount = account;
        DialogResult = true;
        Close();
    }

    private void ViewModel_RequestClose(object? sender, bool success)
    {
        DialogResult = success;
        Close();
    }

    protected override void OnClosed(EventArgs e)
    {
        _viewModel.LoginComplete -= ViewModel_LoginComplete;
        _viewModel.RequestClose -= ViewModel_RequestClose;
        _viewModel.PropertyChanged -= ViewModel_PropertyChanged;
        base.OnClosed(e);
    }
}

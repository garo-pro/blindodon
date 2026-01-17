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

using System.Linq;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using Blindodon.ViewModels;
using Serilog;

namespace Blindodon.Views;

/// <summary>
/// Account selection window shown at startup.
/// </summary>
public partial class AccountSelectWindow : Window
{
    private readonly AccountSelectViewModel _viewModel;

    /// <summary>
    /// Gets the account that was selected, or null if cancelled.
    /// </summary>
    public AccountItemViewModel? SelectedAccount { get; private set; }

    public AccountSelectWindow()
    {
        InitializeComponent();

        _viewModel = new AccountSelectViewModel();
        DataContext = _viewModel;

        // Subscribe to view model events
        _viewModel.AccountSelected += ViewModel_AccountSelected;
        _viewModel.AddAccountRequested += ViewModel_AddAccountRequested;
    }

    private async void Window_Loaded(object sender, RoutedEventArgs e)
    {
        Log.Information("Account selection window loaded");

        // Load saved accounts
        await _viewModel.LoadAccountsAsync();

        // Focus appropriate element
        if (_viewModel.HasAccounts)
        {
            App.Accessibility.Announce($"Select account. {_viewModel.Accounts.Count} accounts available. Use J and K to navigate, Enter to login.");
            AccountListBox.Focus();
        }
        else
        {
            App.Accessibility.Announce("No saved accounts. Press Enter to add an account.");
            AddAccountButton.Focus();
        }
    }

    private void Window_KeyDown(object sender, KeyEventArgs e)
    {
        switch (e.Key)
        {
            case Key.Escape:
                DialogResult = false;
                Close();
                e.Handled = true;
                break;
        }
    }

    private void AccountListBox_KeyDown(object sender, KeyEventArgs e)
    {
        switch (e.Key)
        {
            case Key.J:
                NavigateAccount(1);
                e.Handled = true;
                break;

            case Key.K:
                NavigateAccount(-1);
                e.Handled = true;
                break;

            case Key.Space:
                AnnounceCurrentAccount();
                e.Handled = true;
                break;

            case Key.Enter:
                if (_viewModel.CanLogin)
                {
                    _viewModel.LoginCommand.Execute(null);
                }
                e.Handled = true;
                break;

            case Key.Delete:
            case Key.D:
                if (_viewModel.SelectedAccount != null)
                {
                    _viewModel.DeleteAccountCommand.Execute(_viewModel.SelectedAccount);
                }
                e.Handled = true;
                break;
        }
    }

    private void AccountListBox_MouseDoubleClick(object sender, MouseButtonEventArgs e)
    {
        if (_viewModel.CanLogin)
        {
            _viewModel.LoginCommand.Execute(null);
        }
    }

    private void NavigateAccount(int direction)
    {
        var currentIndex = AccountListBox.SelectedIndex;
        var newIndex = currentIndex + direction;

        if (newIndex < 0)
        {
            App.Audio.Play(Services.AudioManager.SoundEvent.BoundaryReached);
            App.Accessibility.AnnounceBoundary(isTop: true);
            return;
        }

        if (newIndex >= AccountListBox.Items.Count)
        {
            App.Audio.Play(Services.AudioManager.SoundEvent.BoundaryReached);
            App.Accessibility.AnnounceBoundary(isTop: false);
            return;
        }

        AccountListBox.SelectedIndex = newIndex;
        AccountListBox.ScrollIntoView(AccountListBox.SelectedItem);

        // Announce the account
        if (AccountListBox.SelectedItem is AccountItemViewModel account)
        {
            App.Accessibility.Announce(account.AccessibilityLabel);
        }
    }

    private void AnnounceCurrentAccount()
    {
        if (_viewModel.SelectedAccount is AccountItemViewModel account)
        {
            App.Accessibility.Announce(account.AccessibilityLabel);
        }
    }

    private void ViewModel_AccountSelected(object? sender, AccountItemViewModel account)
    {
        SelectedAccount = account;
        DialogResult = true;
        Close();
    }

    private async void ViewModel_AddAccountRequested(object? sender, EventArgs e)
    {
        var loginWindow = new LoginWindow
        {
            Owner = this
        };

        var result = loginWindow.ShowDialog();

        if (result == true && loginWindow.LoggedInAccount != null)
        {
            Log.Information("Login completed for {Account}, refreshing account list",
                loginWindow.LoggedInAccount.EffectiveDisplayName);

            // Refresh the account list to include the new account
            await _viewModel.LoadAccountsAsync();

            // Auto-select the newly added account if found
            var newAccount = _viewModel.Accounts.FirstOrDefault(a =>
                a.InstanceUrl == loginWindow.LoggedInAccount.InstanceUrl);
            if (newAccount != null)
            {
                _viewModel.SelectedAccount = newAccount;
                AccountListBox.ScrollIntoView(newAccount);
                App.Accessibility.Announce($"Account {newAccount.EffectiveDisplayName} added and selected. Press Enter to login.");
            }

            // Focus the list if we now have accounts
            if (_viewModel.HasAccounts)
            {
                AccountListBox.Focus();
            }
        }
        else
        {
            Log.Information("Login cancelled or failed");
        }
    }

    private void CancelButton_Click(object sender, RoutedEventArgs e)
    {
        DialogResult = false;
        Close();
    }

    protected override void OnClosed(EventArgs e)
    {
        _viewModel.AccountSelected -= ViewModel_AccountSelected;
        _viewModel.AddAccountRequested -= ViewModel_AddAccountRequested;
        base.OnClosed(e);
    }
}

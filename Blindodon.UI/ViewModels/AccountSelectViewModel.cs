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
using Newtonsoft.Json.Linq;
using Serilog;

namespace Blindodon.ViewModels;

/// <summary>
/// View model for the account selection window.
/// </summary>
public partial class AccountSelectViewModel : ObservableObject
{
    [ObservableProperty]
    private bool _isLoading;

    [ObservableProperty]
    private string _statusMessage = "";

    [ObservableProperty]
    private AccountItemViewModel? _selectedAccount;

    /// <summary>
    /// Collection of saved accounts.
    /// </summary>
    public ObservableCollection<AccountItemViewModel> Accounts { get; } = new();

    /// <summary>
    /// Returns true if there are saved accounts.
    /// </summary>
    public bool HasAccounts => Accounts.Count > 0;

    /// <summary>
    /// Returns true if an account is selected and can be logged into.
    /// </summary>
    public bool CanLogin => SelectedAccount != null;

    /// <summary>
    /// Event raised when an account is successfully selected and authenticated.
    /// </summary>
    public event EventHandler<AccountItemViewModel>? AccountSelected;

    /// <summary>
    /// Event raised when the user wants to add a new account.
    /// </summary>
    public event EventHandler? AddAccountRequested;

    public AccountSelectViewModel()
    {
        Accounts.CollectionChanged += (s, e) => OnPropertyChanged(nameof(HasAccounts));
    }

    partial void OnSelectedAccountChanged(AccountItemViewModel? value)
    {
        OnPropertyChanged(nameof(CanLogin));
    }

    /// <summary>
    /// Loads the list of saved accounts from the backend.
    /// </summary>
    [RelayCommand]
    public async Task LoadAccountsAsync()
    {
        IsLoading = true;
        StatusMessage = "Loading accounts...";

        try
        {
            var result = await App.Bridge.SendRequestAsync("auth.get_accounts", null);

            if (result != null)
            {
                var accountsArray = result["accounts"]?.ToObject<List<JObject>>();
                if (accountsArray != null)
                {
                    Accounts.Clear();
                    foreach (var accountJson in accountsArray)
                    {
                        Accounts.Add(AccountItemViewModel.FromJson(accountJson));
                    }

                    // Select the default account if any
                    var defaultAccount = Accounts.FirstOrDefault(a => a.IsDefault)
                                         ?? Accounts.FirstOrDefault();
                    SelectedAccount = defaultAccount;

                    StatusMessage = Accounts.Count > 0
                        ? $"{Accounts.Count} account{(Accounts.Count == 1 ? "" : "s")} found"
                        : "No accounts saved";

                    Log.Information("Loaded {Count} accounts", Accounts.Count);
                }
            }
            else
            {
                StatusMessage = "No accounts saved";
            }
        }
        catch (Exception ex)
        {
            Log.Error(ex, "Failed to load accounts");
            StatusMessage = "Failed to load accounts";
        }
        finally
        {
            IsLoading = false;
        }
    }

    /// <summary>
    /// Attempts to log in with the selected account.
    /// </summary>
    [RelayCommand]
    public async Task LoginAsync()
    {
        if (SelectedAccount == null)
        {
            StatusMessage = "Please select an account";
            return;
        }

        IsLoading = true;
        StatusMessage = $"Logging in as {SelectedAccount.EffectiveDisplayName}...";

        try
        {
            var result = await App.Bridge.SendRequestAsync("auth.switch_account", new
            {
                account_id = SelectedAccount.Id
            });

            if (result != null && result["success"]?.Value<bool>() == true)
            {
                Log.Information("Switched to account {AccountId}", SelectedAccount.Id);
                App.Audio.Play(Services.AudioManager.SoundEvent.Connected);
                AccountSelected?.Invoke(this, SelectedAccount);
            }
            else
            {
                var error = result?["error"]?.Value<string>() ?? "Login failed";
                StatusMessage = error;
                App.Audio.Play(Services.AudioManager.SoundEvent.Error);
                App.Accessibility.Announce(error);
            }
        }
        catch (Exception ex)
        {
            Log.Error(ex, "Failed to switch account");
            StatusMessage = "Login failed. Please try again.";
            App.Audio.Play(Services.AudioManager.SoundEvent.Error);
        }
        finally
        {
            IsLoading = false;
        }
    }

    /// <summary>
    /// Requests to add a new account (opens OAuth flow).
    /// </summary>
    [RelayCommand]
    public void AddAccount()
    {
        AddAccountRequested?.Invoke(this, EventArgs.Empty);
    }

    /// <summary>
    /// Deletes the specified account.
    /// </summary>
    [RelayCommand]
    public async Task DeleteAccountAsync(AccountItemViewModel? account)
    {
        if (account == null) return;

        IsLoading = true;
        StatusMessage = $"Removing {account.EffectiveDisplayName}...";

        try
        {
            var result = await App.Bridge.SendRequestAsync("auth.delete_account", new
            {
                account_id = account.Id
            });

            if (result != null && result["success"]?.Value<bool>() == true)
            {
                Accounts.Remove(account);

                if (SelectedAccount == account)
                {
                    SelectedAccount = Accounts.FirstOrDefault();
                }

                StatusMessage = "Account removed";
                App.Accessibility.Announce($"Account {account.EffectiveDisplayName} removed");
                Log.Information("Deleted account {AccountId}", account.Id);
            }
            else
            {
                StatusMessage = "Failed to remove account";
                App.Audio.Play(Services.AudioManager.SoundEvent.Error);
            }
        }
        catch (Exception ex)
        {
            Log.Error(ex, "Failed to delete account");
            StatusMessage = "Failed to remove account";
            App.Audio.Play(Services.AudioManager.SoundEvent.Error);
        }
        finally
        {
            IsLoading = false;
        }
    }
}

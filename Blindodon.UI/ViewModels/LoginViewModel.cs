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

using System.Diagnostics;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Newtonsoft.Json.Linq;
using Serilog;

namespace Blindodon.ViewModels;

/// <summary>
/// Represents the current state of the login flow.
/// </summary>
public enum LoginState
{
    /// <summary>
    /// Waiting for the user to enter their instance URL.
    /// </summary>
    EnterInstance,

    /// <summary>
    /// OAuth started, waiting for user to enter the authorization code.
    /// </summary>
    WaitingForCode,

    /// <summary>
    /// Processing the authorization callback.
    /// </summary>
    Processing
}

/// <summary>
/// View model for the login window handling OAuth flow.
/// </summary>
public partial class LoginViewModel : ObservableObject
{
    [ObservableProperty]
    private string _instanceUrl = "";

    [ObservableProperty]
    private string _authorizationCode = "";

    [ObservableProperty]
    private LoginState _currentState = LoginState.EnterInstance;

    [ObservableProperty]
    private string _statusMessage = "";

    [ObservableProperty]
    private bool _isLoading;

    /// <summary>
    /// Event raised when login is successful.
    /// </summary>
    public event EventHandler<AccountItemViewModel>? LoginComplete;

    /// <summary>
    /// Event raised when the window should close (cancelled or error).
    /// </summary>
    public event EventHandler<bool>? RequestClose;

    /// <summary>
    /// Gets whether the start auth button should be enabled.
    /// </summary>
    public bool CanStartAuth => !string.IsNullOrWhiteSpace(InstanceUrl) && !IsLoading;

    /// <summary>
    /// Gets whether the submit code button should be enabled.
    /// </summary>
    public bool CanSubmitCode => !string.IsNullOrWhiteSpace(AuthorizationCode) && !IsLoading;

    partial void OnInstanceUrlChanged(string value)
    {
        OnPropertyChanged(nameof(CanStartAuth));
    }

    partial void OnAuthorizationCodeChanged(string value)
    {
        OnPropertyChanged(nameof(CanSubmitCode));
    }

    partial void OnIsLoadingChanged(bool value)
    {
        OnPropertyChanged(nameof(CanStartAuth));
        OnPropertyChanged(nameof(CanSubmitCode));
    }

    /// <summary>
    /// Starts the OAuth authentication flow.
    /// </summary>
    [RelayCommand]
    public async Task StartAuthAsync()
    {
        if (string.IsNullOrWhiteSpace(InstanceUrl))
        {
            StatusMessage = "Please enter your instance URL";
            return;
        }

        // Normalize the instance URL
        var normalizedUrl = InstanceUrl.Trim();
        if (!normalizedUrl.StartsWith("http://") && !normalizedUrl.StartsWith("https://"))
        {
            normalizedUrl = "https://" + normalizedUrl;
        }
        InstanceUrl = normalizedUrl;

        IsLoading = true;
        StatusMessage = "Starting authentication...";
        App.Accessibility.Announce("Starting authentication. Please wait.");

        try
        {
            var result = await App.Bridge.SendRequestAsync("auth.start", new
            {
                instance_url = InstanceUrl
            });

            if (result != null)
            {
                var authUrl = result["auth_url"]?.Value<string>();
                if (!string.IsNullOrEmpty(authUrl))
                {
                    StatusMessage = "Opening browser for authorization...";
                    App.Accessibility.Announce("Opening your browser. After authorizing, copy the code and paste it here.");

                    // Open the auth URL in the default browser
                    Process.Start(new ProcessStartInfo
                    {
                        FileName = authUrl,
                        UseShellExecute = true
                    });

                    // Move to the code entry state
                    CurrentState = LoginState.WaitingForCode;
                    StatusMessage = "Paste the authorization code from your browser";
                }
                else
                {
                    StatusMessage = "Failed to get authorization URL";
                    App.Audio.Play(Services.AudioManager.SoundEvent.Error);
                }
            }
            else
            {
                StatusMessage = "Failed to start authentication";
                App.Audio.Play(Services.AudioManager.SoundEvent.Error);
            }
        }
        catch (Exception ex)
        {
            Log.Error(ex, "Failed to start authentication");
            StatusMessage = $"Error: {ex.Message}";
            App.Audio.Play(Services.AudioManager.SoundEvent.Error);
        }
        finally
        {
            IsLoading = false;
        }
    }

    /// <summary>
    /// Submits the authorization code to complete the OAuth flow.
    /// </summary>
    [RelayCommand]
    public async Task SubmitCodeAsync()
    {
        if (string.IsNullOrWhiteSpace(AuthorizationCode))
        {
            StatusMessage = "Please enter the authorization code";
            return;
        }

        IsLoading = true;
        CurrentState = LoginState.Processing;
        StatusMessage = "Completing authentication...";
        App.Accessibility.Announce("Completing authentication. Please wait.");

        try
        {
            var result = await App.Bridge.SendRequestAsync("auth.callback", new
            {
                instance_url = InstanceUrl,
                code = AuthorizationCode.Trim()
            });

            if (result != null && result["success"]?.Value<bool>() == true)
            {
                StatusMessage = "Login successful!";
                App.Audio.Play(Services.AudioManager.SoundEvent.Connected);
                App.Accessibility.Announce("Login successful!");

                // Create account view model from the response
                var accountJson = result["account"]?.ToObject<Newtonsoft.Json.Linq.JObject>();
                if (accountJson != null)
                {
                    var account = AccountItemViewModel.FromJson(accountJson);
                    Log.Information("Login successful for {Username} at {Instance}",
                        account.Username, account.InstanceDomain);
                    LoginComplete?.Invoke(this, account);
                }
                else
                {
                    // Create a minimal account if we don't get full details
                    var account = new AccountItemViewModel
                    {
                        InstanceUrl = InstanceUrl,
                        Username = "user"
                    };
                    LoginComplete?.Invoke(this, account);
                }
            }
            else
            {
                var error = result?["error"]?.Value<string>() ?? "Authentication failed";
                StatusMessage = error;
                App.Audio.Play(Services.AudioManager.SoundEvent.Error);
                App.Accessibility.Announce(error);

                // Go back to code entry state to let user try again
                CurrentState = LoginState.WaitingForCode;
            }
        }
        catch (Exception ex)
        {
            Log.Error(ex, "Failed to complete authentication");
            StatusMessage = $"Error: {ex.Message}";
            App.Audio.Play(Services.AudioManager.SoundEvent.Error);
            CurrentState = LoginState.WaitingForCode;
        }
        finally
        {
            IsLoading = false;
        }
    }

    /// <summary>
    /// Cancels the login flow and closes the window.
    /// </summary>
    [RelayCommand]
    public void Cancel()
    {
        Log.Information("Login cancelled by user");
        RequestClose?.Invoke(this, false);
    }

    /// <summary>
    /// Goes back to the instance URL entry state.
    /// </summary>
    [RelayCommand]
    public void GoBack()
    {
        CurrentState = LoginState.EnterInstance;
        AuthorizationCode = "";
        StatusMessage = "";
        App.Accessibility.Announce("Back to instance entry. Enter your Mastodon instance URL.");
    }
}

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
using System.IO;
using System.Windows;
using Blindodon.Services;
using Serilog;

namespace Blindodon;

/// <summary>
/// Main application class for Blindodon
/// </summary>
public partial class App : Application
{
    private Process? _rustCoreProcess;
    private MastodonBridge? _bridge;

    /// <summary>
    /// Gets the Mastodon bridge for IPC communication
    /// </summary>
    public static MastodonBridge Bridge => ((App)Current)._bridge!;

    /// <summary>
    /// Gets the audio manager
    /// </summary>
    public static AudioManager Audio { get; private set; } = null!;

    /// <summary>
    /// Gets the accessibility manager
    /// </summary>
    public static AccessibilityManager Accessibility { get; private set; } = null!;

    /// <summary>
    /// Gets the keybinding manager
    /// </summary>
    public static KeybindingManager Keybindings { get; private set; } = null!;

    protected override async void OnStartup(StartupEventArgs e)
    {
        // Set up global exception handling first
        AppDomain.CurrentDomain.UnhandledException += (s, args) =>
        {
            var ex = args.ExceptionObject as Exception;
            Log.Fatal(ex, "Unhandled exception");
            MessageBox.Show($"Fatal error: {ex?.Message}\n\n{ex?.StackTrace}", "Blindodon Error", MessageBoxButton.OK, MessageBoxImage.Error);
        };

        DispatcherUnhandledException += (s, args) =>
        {
            Log.Error(args.Exception, "Dispatcher unhandled exception");
            MessageBox.Show($"Error: {args.Exception.Message}\n\n{args.Exception.StackTrace}", "Blindodon Error", MessageBoxButton.OK, MessageBoxImage.Error);
            args.Handled = true;
        };

        // Initialize logging BEFORE anything else
        InitializeLogging();

        Log.Information("Blindodon starting up...");

        try
        {
            base.OnStartup(e);

            // Prevent automatic shutdown when dialogs close
            ShutdownMode = ShutdownMode.OnExplicitShutdown;

            // Initialize services
            InitializeServices();

            // Start the Rust core process
            StartRustCore();

            // Wait for IPC connection
            await ConnectToBackendAsync();

            // Show account selection window
            var accountSelectWindow = new Views.AccountSelectWindow();
            var result = accountSelectWindow.ShowDialog();

            if (result == true && accountSelectWindow.SelectedAccount != null)
            {
                // User selected an account, show main window
                Log.Information("Account selected: {Account}", accountSelectWindow.SelectedAccount.EffectiveDisplayName);
                var mainWindow = new Views.MainWindow();
                MainWindow = mainWindow;

                // Switch to normal shutdown mode now that main window exists
                ShutdownMode = ShutdownMode.OnMainWindowClose;

                mainWindow.Show();
            }
            else
            {
                // User cancelled, exit application
                Log.Information("Account selection cancelled, exiting");
                Shutdown(0);
            }
        }
        catch (Exception ex)
        {
            Log.Fatal(ex, "Startup failed");
            MessageBox.Show($"Startup failed: {ex.Message}\n\n{ex.StackTrace}", "Blindodon Error", MessageBoxButton.OK, MessageBoxImage.Error);
            Shutdown(1);
        }
    }

    private async Task ConnectToBackendAsync()
    {
        Log.Information("Connecting to backend...");
        var connected = await _bridge!.ConnectAsync();

        if (connected)
        {
            Log.Information("Connected to Rust core");
        }
        else
        {
            Log.Warning("Could not connect to Rust core, running in UI-only mode");
        }
    }

    private void InitializeLogging()
    {
        var logPath = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
            "Blindodon",
            "logs",
            "blindodon-ui-.log"
        );

        Log.Logger = new LoggerConfiguration()
            .MinimumLevel.Debug()
            .WriteTo.Console()
            .WriteTo.File(logPath, rollingInterval: RollingInterval.Day)
            .CreateLogger();
    }

    private void InitializeServices()
    {
        // Initialize the IPC bridge
        _bridge = new MastodonBridge();

        // Initialize audio manager
        Audio = new AudioManager();

        // Initialize accessibility manager
        Accessibility = new AccessibilityManager();

        // Initialize keybinding manager
        Keybindings = new KeybindingManager();

        Log.Information("Services initialized");
    }

    private void StartRustCore()
    {
        try
        {
            var rustCorePath = Path.Combine(
                AppDomain.CurrentDomain.BaseDirectory,
                "mastodon-core.exe"
            );

            if (!File.Exists(rustCorePath))
            {
                // Try development path
                rustCorePath = Path.Combine(
                    AppDomain.CurrentDomain.BaseDirectory,
                    "..", "..", "..", "..",
                    "mastodon-core", "target", "debug", "mastodon-core.exe"
                );
            }

            if (File.Exists(rustCorePath))
            {
                Log.Information("Starting Rust core from: {Path}", rustCorePath);

                _rustCoreProcess = new Process
                {
                    StartInfo = new ProcessStartInfo
                    {
                        FileName = rustCorePath,
                        UseShellExecute = false,
                        CreateNoWindow = true,
                        RedirectStandardOutput = true,
                        RedirectStandardError = true
                    }
                };

                _rustCoreProcess.OutputDataReceived += (s, e) =>
                {
                    if (!string.IsNullOrEmpty(e.Data))
                        Log.Debug("[Rust] {Message}", e.Data);
                };

                _rustCoreProcess.ErrorDataReceived += (s, e) =>
                {
                    if (!string.IsNullOrEmpty(e.Data))
                        Log.Warning("[Rust] {Message}", e.Data);
                };

                _rustCoreProcess.Start();
                _rustCoreProcess.BeginOutputReadLine();
                _rustCoreProcess.BeginErrorReadLine();

                Log.Information("Rust core started with PID: {PID}", _rustCoreProcess.Id);
            }
            else
            {
                Log.Warning("Rust core not found at expected path. Running in UI-only mode.");
            }
        }
        catch (Exception ex)
        {
            Log.Error(ex, "Failed to start Rust core");
        }
    }

    protected override void OnExit(ExitEventArgs e)
    {
        Log.Information("Blindodon shutting down...");

        // Disconnect from IPC
        _bridge?.Disconnect();

        // Stop the Rust core process
        if (_rustCoreProcess != null && !_rustCoreProcess.HasExited)
        {
            try
            {
                // Send shutdown command first
                _bridge?.SendShutdown().Wait(TimeSpan.FromSeconds(2));

                if (!_rustCoreProcess.HasExited)
                {
                    _rustCoreProcess.Kill();
                }
            }
            catch (Exception ex)
            {
                Log.Warning(ex, "Error stopping Rust core");
            }
        }

        // Dispose services
        Audio?.Dispose();

        Log.Information("Blindodon shutdown complete");
        Log.CloseAndFlush();

        base.OnExit(e);
    }
}

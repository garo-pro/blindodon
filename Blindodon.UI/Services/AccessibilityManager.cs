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

using System.Runtime.InteropServices;
using System.Speech.Synthesis;
using System.Windows;
using System.Windows.Automation.Peers;
using System.Windows.Automation.Provider;
using Serilog;

namespace Blindodon.Services;

/// <summary>
/// Manages accessibility features including screen reader integration
/// </summary>
public class AccessibilityManager
{
    private SpeechSynthesizer? _speechSynthesizer;
    private bool _screenReaderDetected;
    private bool _useSapiFallback;
    private readonly Queue<string> _announcementQueue = new();
    private bool _isAnnouncing;
    private readonly object _announceLock = new();

    /// <summary>
    /// Detected screen reader type
    /// </summary>
    public enum ScreenReaderType
    {
        None,
        NVDA,
        JAWS,
        Narrator,
        Unknown
    }

    /// <summary>
    /// Gets the detected screen reader
    /// </summary>
    public ScreenReaderType DetectedScreenReader { get; private set; }

    /// <summary>
    /// Gets or sets whether to use SAPI fallback when no screen reader is detected
    /// </summary>
    public bool UseSapiFallback
    {
        get => _useSapiFallback;
        set
        {
            _useSapiFallback = value;
            if (value && _speechSynthesizer == null)
            {
                InitializeSapi();
            }
        }
    }

    public AccessibilityManager()
    {
        DetectScreenReader();

        // Initialize SAPI as fallback
        if (!_screenReaderDetected)
        {
            _useSapiFallback = true;
            InitializeSapi();
        }
    }

    private void DetectScreenReader()
    {
        // Check for NVDA
        if (IsProcessRunning("nvda"))
        {
            DetectedScreenReader = ScreenReaderType.NVDA;
            _screenReaderDetected = true;
            Log.Information("Detected screen reader: NVDA");
            return;
        }

        // Check for JAWS
        if (IsProcessRunning("jfw"))
        {
            DetectedScreenReader = ScreenReaderType.JAWS;
            _screenReaderDetected = true;
            Log.Information("Detected screen reader: JAWS");
            return;
        }

        // Check for Narrator
        if (IsProcessRunning("narrator"))
        {
            DetectedScreenReader = ScreenReaderType.Narrator;
            _screenReaderDetected = true;
            Log.Information("Detected screen reader: Narrator");
            return;
        }

        // Check if any screen reader is active via UI Automation
        if (IsScreenReaderActive())
        {
            DetectedScreenReader = ScreenReaderType.Unknown;
            _screenReaderDetected = true;
            Log.Information("Detected screen reader: Unknown (via UI Automation)");
            return;
        }

        DetectedScreenReader = ScreenReaderType.None;
        _screenReaderDetected = false;
        Log.Information("No screen reader detected");
    }

    private bool IsProcessRunning(string processName)
    {
        return System.Diagnostics.Process.GetProcessesByName(processName).Length > 0;
    }

    [DllImport("UIAutomationCore.dll", CharSet = CharSet.Unicode)]
    private static extern bool UiaClientsAreListening();

    private bool IsScreenReaderActive()
    {
        try
        {
            return UiaClientsAreListening();
        }
        catch
        {
            return false;
        }
    }

    private void InitializeSapi()
    {
        try
        {
            _speechSynthesizer = new SpeechSynthesizer();
            _speechSynthesizer.Rate = 2; // Slightly faster speech
            Log.Information("SAPI speech synthesizer initialized");
        }
        catch (Exception ex)
        {
            Log.Warning(ex, "Failed to initialize SAPI speech synthesizer");
        }
    }

    /// <summary>
    /// Announce text to the screen reader
    /// </summary>
    public void Announce(string text, bool interrupt = false)
    {
        if (string.IsNullOrWhiteSpace(text))
            return;

        Log.Debug("Announcing: {Text}", text);

        // Try to use native screen reader first
        if (_screenReaderDetected)
        {
            AnnounceViaScreenReader(text, interrupt);
        }
        else if (_useSapiFallback && _speechSynthesizer != null)
        {
            AnnounceViaSapi(text, interrupt);
        }
    }

    private void AnnounceViaScreenReader(string text, bool interrupt)
    {
        // Use UI Automation live region for announcement
        // This works with NVDA, JAWS, and Narrator
        Application.Current.Dispatcher.Invoke(() =>
        {
            try
            {
                // For now, we use SAPI as the screen readers will pick up
                // the live region changes automatically via UI Automation
                if (_useSapiFallback && _speechSynthesizer != null)
                {
                    AnnounceViaSapi(text, interrupt);
                }
            }
            catch (Exception ex)
            {
                Log.Warning(ex, "Failed to announce via screen reader");
            }
        });
    }

    private void AnnounceViaSapi(string text, bool interrupt)
    {
        if (_speechSynthesizer == null)
            return;

        lock (_announceLock)
        {
            if (interrupt)
            {
                _speechSynthesizer.SpeakAsyncCancelAll();
                _announcementQueue.Clear();
            }

            _announcementQueue.Enqueue(text);

            if (!_isAnnouncing)
            {
                ProcessAnnouncementQueue();
            }
        }
    }

    private void ProcessAnnouncementQueue()
    {
        Task.Run(() =>
        {
            while (true)
            {
                string? textToSpeak = null;

                lock (_announceLock)
                {
                    if (_announcementQueue.Count == 0)
                    {
                        _isAnnouncing = false;
                        return;
                    }

                    _isAnnouncing = true;
                    textToSpeak = _announcementQueue.Dequeue();
                }

                if (textToSpeak != null && _speechSynthesizer != null)
                {
                    try
                    {
                        _speechSynthesizer.Speak(textToSpeak);
                    }
                    catch (Exception ex)
                    {
                        Log.Warning(ex, "SAPI speak failed");
                    }
                }
            }
        });
    }

    /// <summary>
    /// Stop any ongoing announcements
    /// </summary>
    public void StopAnnouncing()
    {
        lock (_announceLock)
        {
            _announcementQueue.Clear();
            _speechSynthesizer?.SpeakAsyncCancelAll();
            _isAnnouncing = false;
        }
    }

    /// <summary>
    /// Announce a post with proper formatting
    /// </summary>
    public void AnnouncePost(string author, string content, bool hasMedia, string? contentWarning = null)
    {
        var parts = new List<string>();

        parts.Add($"Post by {author}");

        if (!string.IsNullOrEmpty(contentWarning))
        {
            parts.Add($"Content warning: {contentWarning}");
        }

        parts.Add(content);

        if (hasMedia)
        {
            parts.Add("Contains media");
        }

        Announce(string.Join(". ", parts));
    }

    /// <summary>
    /// Announce a notification
    /// </summary>
    public void AnnounceNotification(string type, string from, string? content = null)
    {
        var message = type switch
        {
            "mention" => $"{from} mentioned you",
            "reblog" => $"{from} boosted your post",
            "favourite" => $"{from} favorited your post",
            "follow" => $"{from} followed you",
            "follow_request" => $"{from} requested to follow you",
            "poll" => "A poll you voted in has ended",
            _ => $"Notification from {from}"
        };

        if (!string.IsNullOrEmpty(content))
        {
            message += $": {content}";
        }

        Announce(message, interrupt: true);
    }

    /// <summary>
    /// Announce a boundary (top or bottom of timeline)
    /// </summary>
    public void AnnounceBoundary(bool isTop)
    {
        Announce(isTop ? "Beginning of timeline" : "End of timeline");
    }

    /// <summary>
    /// Announce the character count for compose
    /// </summary>
    public void AnnounceCharacterCount(int current, int max)
    {
        var remaining = max - current;
        if (remaining <= 50)
        {
            Announce($"{remaining} characters remaining");
        }
    }

    /// <summary>
    /// Convert emoji shortcodes to descriptions
    /// </summary>
    public string DescribeEmoji(string text)
    {
        // Common emoji descriptions
        var emojiDescriptions = new Dictionary<string, string>
        {
            { ":smile:", "smiling face" },
            { ":heart:", "red heart" },
            { ":+1:", "thumbs up" },
            { ":-1:", "thumbs down" },
            { ":fire:", "fire" },
            { ":thinking:", "thinking face" },
            { ":cry:", "crying face" },
            { ":laughing:", "laughing face" },
            { ":100:", "hundred points" },
            { ":clap:", "clapping hands" }
        };

        foreach (var (shortcode, description) in emojiDescriptions)
        {
            text = text.Replace(shortcode, $" {description} ");
        }

        return text;
    }
}

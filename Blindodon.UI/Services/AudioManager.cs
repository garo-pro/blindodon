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

using System.IO;
using NAudio.Wave;
using Serilog;

namespace Blindodon.Services;

/// <summary>
/// Manages audio feedback for accessibility
/// </summary>
public class AudioManager : IDisposable
{
    private readonly Dictionary<SoundEvent, string> _soundPaths = new();
    private readonly Dictionary<SoundEvent, float> _volumes = new();
    private readonly object _playLock = new();
    private bool _enabled = true;
    private float _masterVolume = 1.0f;
    private string _soundPackPath;

    /// <summary>
    /// Sound events that can trigger audio feedback
    /// </summary>
    public enum SoundEvent
    {
        NewPost,
        NewMention,
        NewDirectMessage,
        NewNotification,
        PostSent,
        Error,
        FavoriteAdded,
        BoostSent,
        BoundaryReached,
        MediaLoaded,
        FilterActivated,
        Connected,
        Disconnected
    }

    public AudioManager()
    {
        _soundPackPath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Sounds", "default");
        LoadSoundPack(_soundPackPath);
    }

    /// <summary>
    /// Gets or sets whether audio feedback is enabled
    /// </summary>
    public bool Enabled
    {
        get => _enabled;
        set
        {
            _enabled = value;
            Log.Information("Audio feedback {State}", value ? "enabled" : "disabled");
        }
    }

    /// <summary>
    /// Gets or sets the master volume (0.0 to 1.0)
    /// </summary>
    public float MasterVolume
    {
        get => _masterVolume;
        set
        {
            _masterVolume = Math.Clamp(value, 0.0f, 1.0f);
            Log.Debug("Master volume set to {Volume}", _masterVolume);
        }
    }

    /// <summary>
    /// Load a sound pack from a directory
    /// </summary>
    public void LoadSoundPack(string path)
    {
        _soundPackPath = path;
        _soundPaths.Clear();

        if (!Directory.Exists(path))
        {
            Log.Warning("Sound pack directory not found: {Path}", path);
            return;
        }

        // Map sound events to file names
        var soundFiles = new Dictionary<SoundEvent, string>
        {
            { SoundEvent.NewPost, "new_post.wav" },
            { SoundEvent.NewMention, "mention.wav" },
            { SoundEvent.NewDirectMessage, "dm.wav" },
            { SoundEvent.NewNotification, "notification.wav" },
            { SoundEvent.PostSent, "sent.wav" },
            { SoundEvent.Error, "error.wav" },
            { SoundEvent.FavoriteAdded, "favorite.wav" },
            { SoundEvent.BoostSent, "boost.wav" },
            { SoundEvent.BoundaryReached, "boundary.wav" },
            { SoundEvent.MediaLoaded, "media.wav" },
            { SoundEvent.FilterActivated, "filter.wav" },
            { SoundEvent.Connected, "connected.wav" },
            { SoundEvent.Disconnected, "disconnected.wav" }
        };

        foreach (var (soundEvent, fileName) in soundFiles)
        {
            var filePath = Path.Combine(path, fileName);
            if (File.Exists(filePath))
            {
                _soundPaths[soundEvent] = filePath;
                _volumes[soundEvent] = 1.0f; // Default volume per sound
            }
        }

        Log.Information("Loaded {Count} sounds from pack: {Path}", _soundPaths.Count, path);
    }

    /// <summary>
    /// Play a sound event
    /// </summary>
    public void Play(SoundEvent soundEvent)
    {
        if (!_enabled)
            return;

        if (!_soundPaths.TryGetValue(soundEvent, out var filePath))
        {
            Log.Debug("No sound configured for event: {Event}", soundEvent);
            return;
        }

        Task.Run(() => PlaySoundFile(filePath, soundEvent));
    }

    /// <summary>
    /// Set the volume for a specific sound event
    /// </summary>
    public void SetEventVolume(SoundEvent soundEvent, float volume)
    {
        _volumes[soundEvent] = Math.Clamp(volume, 0.0f, 1.0f);
    }

    private void PlaySoundFile(string filePath, SoundEvent soundEvent)
    {
        try
        {
            lock (_playLock)
            {
                using var audioFile = new AudioFileReader(filePath);
                using var outputDevice = new WaveOutEvent();

                // Apply volume
                var eventVolume = _volumes.GetValueOrDefault(soundEvent, 1.0f);
                audioFile.Volume = _masterVolume * eventVolume;

                outputDevice.Init(audioFile);
                outputDevice.Play();

                // Wait for playback to complete (with timeout)
                while (outputDevice.PlaybackState == PlaybackState.Playing)
                {
                    Thread.Sleep(10);
                }
            }
        }
        catch (Exception ex)
        {
            Log.Warning(ex, "Failed to play sound: {Event}", soundEvent);
        }
    }

    /// <summary>
    /// Play a custom sound file
    /// </summary>
    public void PlayCustom(string filePath)
    {
        if (!_enabled || !File.Exists(filePath))
            return;

        Task.Run(() =>
        {
            try
            {
                lock (_playLock)
                {
                    using var audioFile = new AudioFileReader(filePath);
                    using var outputDevice = new WaveOutEvent();

                    audioFile.Volume = _masterVolume;
                    outputDevice.Init(audioFile);
                    outputDevice.Play();

                    while (outputDevice.PlaybackState == PlaybackState.Playing)
                    {
                        Thread.Sleep(10);
                    }
                }
            }
            catch (Exception ex)
            {
                Log.Warning(ex, "Failed to play custom sound: {Path}", filePath);
            }
        });
    }

    public void Dispose()
    {
        // Cleanup if needed
    }
}

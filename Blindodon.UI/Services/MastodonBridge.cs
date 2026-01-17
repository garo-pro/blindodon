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

using System.Collections.Concurrent;
using System.IO;
using System.IO.Pipes;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using Serilog;

namespace Blindodon.Services;

/// <summary>
/// IPC bridge for communication with the Rust core
/// </summary>
public class MastodonBridge : IDisposable
{
    private const string PipeName = "blindodon_ipc";
    private NamedPipeClientStream? _pipe;
    private StreamReader? _reader;
    private StreamWriter? _writer;
    private readonly ConcurrentDictionary<string, TaskCompletionSource<IpcMessage>> _pendingRequests = new();
    private CancellationTokenSource? _readCancellation;
    private Task? _readTask;
    private bool _isConnected;

    /// <summary>
    /// Event raised when an event is received from the Rust core
    /// </summary>
    public event EventHandler<IpcEvent>? EventReceived;

    /// <summary>
    /// Event raised when the connection state changes
    /// </summary>
    public event EventHandler<bool>? ConnectionStateChanged;

    /// <summary>
    /// Gets whether the bridge is connected
    /// </summary>
    public bool IsConnected => _isConnected;

    /// <summary>
    /// Connect to the Rust core
    /// </summary>
    public async Task<bool> ConnectAsync(int timeoutMs = 5000)
    {
        try
        {
            Log.Information("Connecting to Rust core via named pipe: {PipeName}", PipeName);

            _pipe = new NamedPipeClientStream(".", PipeName, PipeDirection.InOut, PipeOptions.Asynchronous);

            await _pipe.ConnectAsync(timeoutMs);

            _reader = new StreamReader(_pipe);
            _writer = new StreamWriter(_pipe) { AutoFlush = true };

            _isConnected = true;
            ConnectionStateChanged?.Invoke(this, true);

            // Start reading messages
            _readCancellation = new CancellationTokenSource();
            _readTask = Task.Run(() => ReadMessagesAsync(_readCancellation.Token));

            Log.Information("Connected to Rust core");

            // Send a ping to verify connection
            var pingResult = await SendRequestAsync("ping", null);
            Log.Debug("Ping response: {Response}", pingResult);

            return true;
        }
        catch (Exception ex)
        {
            Log.Error(ex, "Failed to connect to Rust core");
            _isConnected = false;
            return false;
        }
    }

    /// <summary>
    /// Disconnect from the Rust core
    /// </summary>
    public void Disconnect()
    {
        _readCancellation?.Cancel();
        _isConnected = false;

        _writer?.Dispose();
        _reader?.Dispose();
        _pipe?.Dispose();

        _writer = null;
        _reader = null;
        _pipe = null;

        ConnectionStateChanged?.Invoke(this, false);
        Log.Information("Disconnected from Rust core");
    }

    /// <summary>
    /// Send a request and wait for a response
    /// </summary>
    public async Task<JObject?> SendRequestAsync(string method, object? parameters, int timeoutMs = 30000)
    {
        if (!_isConnected || _writer == null)
        {
            Log.Warning("Cannot send request: not connected");
            return null;
        }

        var request = new IpcMessage
        {
            Id = Guid.NewGuid().ToString(),
            Type = "request",
            Method = method,
            Params = parameters != null ? JObject.FromObject(parameters) : null
        };

        var tcs = new TaskCompletionSource<IpcMessage>();
        _pendingRequests[request.Id] = tcs;

        try
        {
            var json = JsonConvert.SerializeObject(request);
            Log.Debug("Sending request: {Method} ({Id})", method, request.Id);

            await _writer.WriteLineAsync(json);

            using var cts = new CancellationTokenSource(timeoutMs);
            var completedTask = await Task.WhenAny(tcs.Task, Task.Delay(timeoutMs, cts.Token));

            if (completedTask != tcs.Task)
            {
                Log.Warning("Request timed out: {Method} ({Id})", method, request.Id);
                return null;
            }

            var response = await tcs.Task;

            if (response.Error != null)
            {
                Log.Warning("Request error: {Method} - {Error}", method, response.Error.Message);
                return null;
            }

            return response.Result;
        }
        finally
        {
            _pendingRequests.TryRemove(request.Id, out _);
        }
    }

    /// <summary>
    /// Send shutdown command to the Rust core
    /// </summary>
    public async Task SendShutdown()
    {
        await SendRequestAsync("shutdown", null, 2000);
    }

    private async Task ReadMessagesAsync(CancellationToken ct)
    {
        try
        {
            while (!ct.IsCancellationRequested && _reader != null)
            {
                var line = await _reader.ReadLineAsync(ct);

                if (string.IsNullOrEmpty(line))
                    continue;

                try
                {
                    var message = JsonConvert.DeserializeObject<IpcMessage>(line);

                    if (message == null)
                        continue;

                    Log.Debug("Received message: {Type} {Method}", message.Type, message.Method);

                    if (message.Type == "response" && _pendingRequests.TryGetValue(message.Id, out var tcs))
                    {
                        tcs.TrySetResult(message);
                    }
                    else if (message.Type == "event")
                    {
                        HandleEvent(message);
                    }
                }
                catch (JsonException ex)
                {
                    Log.Warning(ex, "Failed to parse message: {Line}", line);
                }
            }
        }
        catch (OperationCanceledException)
        {
            // Expected on shutdown
        }
        catch (Exception ex)
        {
            Log.Error(ex, "Error reading messages");
            _isConnected = false;
            ConnectionStateChanged?.Invoke(this, false);
        }
    }

    private void HandleEvent(IpcMessage message)
    {
        var eventData = new IpcEvent
        {
            EventType = message.Method ?? "unknown",
            Data = message.Params
        };

        Log.Debug("Received event: {EventType}", eventData.EventType);

        EventReceived?.Invoke(this, eventData);
    }

    public void Dispose()
    {
        Disconnect();
    }
}

/// <summary>
/// IPC message structure
/// </summary>
public class IpcMessage
{
    [JsonProperty("id")]
    public string Id { get; set; } = "";

    [JsonProperty("type")]
    public string Type { get; set; } = "";

    [JsonProperty("method")]
    public string? Method { get; set; }

    [JsonProperty("params")]
    public JObject? Params { get; set; }

    [JsonProperty("result")]
    public JObject? Result { get; set; }

    [JsonProperty("error")]
    public IpcError? Error { get; set; }
}

/// <summary>
/// IPC error structure
/// </summary>
public class IpcError
{
    [JsonProperty("code")]
    public int Code { get; set; }

    [JsonProperty("message")]
    public string Message { get; set; } = "";

    [JsonProperty("data")]
    public JObject? Data { get; set; }
}

/// <summary>
/// IPC event structure
/// </summary>
public class IpcEvent
{
    public string EventType { get; set; } = "";
    public JObject? Data { get; set; }
}

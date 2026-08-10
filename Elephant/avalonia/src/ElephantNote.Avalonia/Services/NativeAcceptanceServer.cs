using System.Net;
using System.Net.Sockets;
using System.Text;
using System.Text.Json;

namespace ElephantNote.Avalonia.Services;

/// <summary>
/// Local-only acceptance control channel. It is created only when the test
/// harness provides ELEPHANTNOTE_ACCEPTANCE_SOCKET; it is never enabled for a
/// normal user launch and it does not expose a network listener.
/// </summary>
public sealed class NativeAcceptanceServer : IAsyncDisposable
{
    private readonly string _socketPath;
    private readonly Socket _listener;
    private readonly Func<JsonElement, Task<object?>> _handler;
    private readonly CancellationTokenSource _shutdown = new();
    private readonly Task _acceptLoop;
    private int _disposed;

    private NativeAcceptanceServer(
        string socketPath,
        Socket listener,
        Func<JsonElement, Task<object?>> handler)
    {
        _socketPath = socketPath;
        _listener = listener;
        _handler = handler;
        _acceptLoop = AcceptLoopAsync();
    }

    public static NativeAcceptanceServer Start(
        string socketPath,
        Func<JsonElement, Task<object?>> handler)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(socketPath);
        ArgumentNullException.ThrowIfNull(handler);
        if (OperatingSystem.IsWindows())
            throw new PlatformNotSupportedException("The native acceptance socket is only available on Unix platforms.");

        var directory = Path.GetDirectoryName(socketPath);
        if (!string.IsNullOrWhiteSpace(directory)) Directory.CreateDirectory(directory);
        if (File.Exists(socketPath)) File.Delete(socketPath);

        var listener = new Socket(AddressFamily.Unix, SocketType.Stream, ProtocolType.Unspecified);
        listener.Bind(new UnixDomainSocketEndPoint(socketPath));
        listener.Listen(1);
        return new NativeAcceptanceServer(socketPath, listener, handler);
    }

    public async ValueTask DisposeAsync()
    {
        if (Interlocked.Exchange(ref _disposed, 1) != 0) return;
        _shutdown.Cancel();
        _listener.Dispose();
        try { await _acceptLoop.ConfigureAwait(false); }
        catch (OperationCanceledException) { }
        catch (ObjectDisposedException) { }
        _shutdown.Dispose();
        TryDeleteSocket();
    }

    private async Task AcceptLoopAsync()
    {
        try
        {
            while (!_shutdown.IsCancellationRequested)
            {
                var client = await _listener.AcceptAsync(_shutdown.Token).ConfigureAwait(false);
                _ = HandleClientAsync(client);
            }
        }
        catch (OperationCanceledException) when (_shutdown.IsCancellationRequested) { }
        catch (ObjectDisposedException) when (_shutdown.IsCancellationRequested) { }
    }

    private async Task HandleClientAsync(Socket client)
    {
        await using var stream = new NetworkStream(client, ownsSocket: true);
        using var reader = new StreamReader(stream, Encoding.UTF8, false, 4096, leaveOpen: true);
        await using var writer = new StreamWriter(stream, new UTF8Encoding(false), 4096, leaveOpen: true)
        {
            AutoFlush = true
        };

        while (!_shutdown.IsCancellationRequested)
        {
            var line = await reader.ReadLineAsync(_shutdown.Token).ConfigureAwait(false);
            if (line is null) return;
            try
            {
                using var document = JsonDocument.Parse(line);
                var result = await _handler(document.RootElement.Clone()).ConfigureAwait(false);
                await writer.WriteLineAsync(JsonSerializer.Serialize(new { ok = true, result })).ConfigureAwait(false);
            }
            catch (Exception error)
            {
                await writer.WriteLineAsync(JsonSerializer.Serialize(new { ok = false, error = error.Message })).ConfigureAwait(false);
            }
        }
    }

    private void TryDeleteSocket()
    {
        try
        {
            if (File.Exists(_socketPath)) File.Delete(_socketPath);
        }
        catch
        {
            // Shutdown must not mask the application close path.
        }
    }
}

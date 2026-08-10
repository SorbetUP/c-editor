using System.Text.Json;
using ElephantNote.Avalonia.Services;

namespace ElephantNote.Avalonia.Muya;

/// <summary>
/// NativeWebView-backed implementation of <see cref="IMuyaHost"/>.
///
/// This type is the boundary adapter between the shell and Muya JS. It does
/// not render Markdown, own the caret, maintain selection/IME state, or write
/// files. It only forwards commands and stores the latest Markdown snapshot
/// reported by Muya so the shell can persist it.
/// </summary>
public class NativeWebViewMuyaHost : IMuyaHost
{
    private readonly MuyaEditorHostOptions _options;
    private readonly SemaphoreSlim _operationGate = new(1, 1);
    private MuyaDocument? _document;
    private IMuyaWebViewAdapter? _webView;
    private bool _webViewReady;
    private bool _disposed;
    private TaskCompletionSource<bool>? _readySignal;

    public NativeWebViewMuyaHost(MuyaEditorHostOptions? options = null)
    {
        _options = options ?? new MuyaEditorHostOptions();
        if (_options.LocalMuyaResource is not null && !MuyaResourceLocator.IsLocal(_options.LocalMuyaResource))
            throw new ArgumentException("The Muya resource must use the file or avares scheme.", nameof(options));
        if (_options.ReadyTimeout <= TimeSpan.Zero)
            throw new ArgumentOutOfRangeException(nameof(options), "Muya ReadyTimeout must be positive.");
    }

    public MuyaDocument? Document => _document;
    public string? Content => _document?.Content;
    public Uri? LocalMuyaResource => _options.LocalMuyaResource;
    public bool IsOpen => _document is not null;
    public bool IsNativeWebViewAttached => _webView is not null;
    public bool IsNativeWebViewReady => _webViewReady;

    public event EventHandler<MuyaDocumentOpenedEventArgs>? DocumentOpened;
    public event EventHandler<MuyaContentChangedEventArgs>? ContentChanged;
    public event EventHandler<MuyaSaveRequestedEventArgs>? SaveRequested;
    public event EventHandler<MuyaEditorErrorEventArgs>? Error;

    public async Task OpenDocumentAsync(
        MuyaOpenDocumentRequest request,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(request);
        ThrowIfDisposed();
        await _operationGate.WaitAsync(cancellationToken).ConfigureAwait(false);
        try
        {
            var document = new MuyaDocument(request.DocumentId, request.Content, request.RelativePath, request.Title);
            _document = document;
            if (_webViewReady) await SendOpenDocumentAsync(document, cancellationToken).ConfigureAwait(false);
            DocumentOpened?.Invoke(this, new MuyaDocumentOpenedEventArgs(document));
        }
        catch (OperationCanceledException)
        {
            throw;
        }
        catch (Exception exception)
        {
            PublishError(new MuyaEditorError("open-document-failed", exception.Message, request.DocumentId, Exception: exception));
            throw;
        }
        finally
        {
            _operationGate.Release();
        }
    }

    public async Task SetMarkdownAsync(
        string markdown,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(markdown);
        ThrowIfDisposed();
        await _operationGate.WaitAsync(cancellationToken).ConfigureAwait(false);
        try
        {
            var document = _document ?? throw new InvalidOperationException("Open a Muya document before setting Markdown.");
            _document = document with { Content = markdown };
            if (_webViewReady) await SendContentAsync(markdown, cancellationToken).ConfigureAwait(false);
            ContentChanged?.Invoke(this, new MuyaContentChangedEventArgs(document.DocumentId, markdown, false));
        }
        catch (OperationCanceledException)
        {
            throw;
        }
        catch (Exception exception)
        {
            PublishError(new MuyaEditorError("set-content-failed", exception.Message, _document?.DocumentId, Exception: exception));
            throw;
        }
        finally
        {
            _operationGate.Release();
        }
    }

    /// <summary>
    /// Compatibility spelling for callers that still call Markdown content
    /// simply "content". <see cref="IMuyaHost.SetMarkdownAsync"/> is canonical.
    /// </summary>
    public Task SetContentAsync(string content, CancellationToken cancellationToken = default) =>
        SetMarkdownAsync(content, cancellationToken);

    public async Task<bool> FocusAsync(CancellationToken cancellationToken = default)
    {
        ThrowIfDisposed();
        await _operationGate.WaitAsync(cancellationToken).ConfigureAwait(false);
        try
        {
            if (!_webViewReady || _webView is null) return false;
            await _webView.InvokeScriptAsync(
                "window.__ELEPHANT_MUYA__?.focus?.();",
                cancellationToken).ConfigureAwait(false);
            return true;
        }
        catch (OperationCanceledException)
        {
            throw;
        }
        catch (Exception exception)
        {
            PublishError(new MuyaEditorError("focus-failed", exception.Message, _document?.DocumentId, Exception: exception));
            throw;
        }
        finally
        {
            _operationGate.Release();
        }
    }

    public async Task<bool> WaitUntilReadyAsync(CancellationToken cancellationToken = default)
    {
        ThrowIfDisposed();
        var signal = _readySignal;
        if (_webViewReady) return true;
        if (signal is null || _webView is null) return false;

        try
        {
            return await signal.Task.WaitAsync(_options.ReadyTimeout, cancellationToken).ConfigureAwait(false);
        }
        catch (TimeoutException)
        {
            PublishError(new MuyaEditorError(
                "native-webview-ready-timeout",
                $"Muya did not complete its JavaScript handshake within {_options.ReadyTimeout.TotalSeconds:0.#} seconds.",
                _document?.DocumentId));
            return false;
        }
    }

    public async Task<bool> AttachNativeWebViewAsync(
        IMuyaWebViewAdapter webView,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(webView);
        ThrowIfDisposed();
        var resource = _options.LocalMuyaResource;
        if (resource is null || !MuyaResourceLocator.Exists(resource)) return false;

        await _operationGate.WaitAsync(cancellationToken).ConfigureAwait(false);
        try
        {
            DetachNativeWebView();
            _webView = webView;
            _readySignal = new TaskCompletionSource<bool>(TaskCreationOptions.RunContinuationsAsynchronously);
            _webView.NavigationCompleted += OnNavigationCompleted;
            _webView.WebMessageReceived += OnWebMessageReceived;
            _webView.Navigate(resource);
            return true;
        }
        catch (OperationCanceledException)
        {
            DetachNativeWebView();
            throw;
        }
        catch (Exception exception)
        {
            DetachNativeWebView();
            PublishError(new MuyaEditorError("native-webview-attach-failed", exception.Message, Exception: exception));
            throw;
        }
        finally
        {
            _operationGate.Release();
        }
    }

    public void DetachNativeWebView()
    {
        if (_webView is null) return;
        _webView.NavigationCompleted -= OnNavigationCompleted;
        _webView.WebMessageReceived -= OnWebMessageReceived;
        _webView = null;
        _webViewReady = false;
        _readySignal?.TrySetResult(false);
        _readySignal = null;
    }

    public async Task HandleWebViewMessageAsync(string body, CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(body);
        ThrowIfDisposed();
        MuyaProtocolMessage message;
        try
        {
            message = MuyaProtocol.Parse(body);
        }
        catch (Exception exception) when (exception is JsonException or InvalidDataException)
        {
            PublishError(new MuyaEditorError("invalid-webview-message", exception.Message, Exception: exception));
            return;
        }

        await _operationGate.WaitAsync(cancellationToken).ConfigureAwait(false);
        try
        {
            switch (message.Type)
            {
                case MuyaProtocol.Ready:
                    _webViewReady = true;
                    _readySignal?.TrySetResult(true);
                    if (_document is not null) await SendOpenDocumentAsync(_document, cancellationToken).ConfigureAwait(false);
                    break;
                case MuyaProtocol.ContentChanged:
                    HandleContentChanged(message);
                    break;
                case MuyaProtocol.SaveRequested:
                    HandleSaveRequested(message);
                    break;
                case MuyaProtocol.Error:
                    PublishError(new MuyaEditorError(
                        message.Code ?? "muya-webview-error",
                        message.Message ?? "Muya reported an unknown error.",
                        message.DocumentId,
                        message.RequestId));
                    break;
                default:
                    PublishError(new MuyaEditorError("unknown-webview-message", $"Unknown Muya message type '{message.Type}'.", message.DocumentId, message.RequestId));
                    break;
            }
        }
        finally
        {
            _operationGate.Release();
        }
    }

    public ValueTask DisposeAsync()
    {
        if (_disposed) return ValueTask.CompletedTask;
        _disposed = true;
        DetachNativeWebView();
        _operationGate.Dispose();
        return ValueTask.CompletedTask;
    }

    private void HandleContentChanged(MuyaProtocolMessage message)
    {
        var document = TryGetCurrentDocument(message);
        if (document is null) return;
        if (message.Content is null)
        {
            PublishError(new MuyaEditorError("content-missing", "Muya content-changed messages require content.", document.DocumentId));
            return;
        }

        _document = document with { Content = message.Content };
        ContentChanged?.Invoke(this, new MuyaContentChangedEventArgs(
            document.DocumentId,
            message.Content,
            true,
            message.ValidationEngine,
            message.ValidationRevision));
    }

    private void HandleSaveRequested(MuyaProtocolMessage message)
    {
        var document = TryGetCurrentDocument(message);
        if (document is null) return;
        if (message.Content is null)
        {
            PublishError(new MuyaEditorError("content-missing", "Muya save-request messages require content.", document.DocumentId, message.RequestId));
            return;
        }

        _document = document with { Content = message.Content };
        var request = new MuyaSaveRequest(
            message.RequestId ?? Guid.NewGuid().ToString("N"),
            document.DocumentId,
            message.Content,
            message.ValidationEngine,
            message.ValidationRevision);
        SaveRequested?.Invoke(this, new MuyaSaveRequestedEventArgs(request));
    }

    private MuyaDocument? TryGetCurrentDocument(MuyaProtocolMessage message)
    {
        if (_document is null)
        {
            PublishError(new MuyaEditorError("document-not-open", "Muya sent a message before a document was opened.", message.DocumentId, message.RequestId));
            return null;
        }

        if (message.DocumentId is not null && message.DocumentId != _document.DocumentId)
        {
            PublishError(new MuyaEditorError("document-mismatch", "Muya message belongs to a different document.", message.DocumentId, message.RequestId));
            return null;
        }

        return _document;
    }

    private void OnNavigationCompleted(object? sender, EventArgs e)
    {
        // Navigation only proves that the page loaded. The JS runtime owns
        // protocol readiness and sends `ready` after it creates the receiver.
        // Sending the document here races that bootstrap and can lose the
        // first open-document message.
        var requestId = AppLog.Start("muya.navigation");
        var timer = System.Diagnostics.Stopwatch.StartNew();
        AppLog.Complete("muya.navigation", requestId, timer, "ready=awaited");
    }

    private async void OnWebMessageReceived(object? sender, MuyaWebViewMessageEventArgs e)
    {
        await HandleWebViewMessageAsync(e.Body).ConfigureAwait(false);
    }

    private Task SendOpenDocumentAsync(MuyaDocument document, CancellationToken cancellationToken) =>
        SendAsync(MuyaProtocol.OpenDocument, document, cancellationToken);

    private Task SendContentAsync(string content, CancellationToken cancellationToken)
    {
        var document = _document ?? throw new InvalidOperationException("Open a Muya document before sending content.");
        return SendAsync(MuyaProtocol.SetContent, document with { Content = content }, cancellationToken);
    }

    private async Task SendAsync(string type, MuyaDocument document, CancellationToken cancellationToken)
    {
        var webView = _webView ?? throw new InvalidOperationException("The Muya NativeWebView is not attached.");
        var json = MuyaProtocol.Serialize(type, document);
        await webView.InvokeScriptAsync($"window.__ELEPHANT_MUYA_HOST__?.receive({json});", cancellationToken).ConfigureAwait(false);
    }

    private void PublishError(MuyaEditorError error) => Error?.Invoke(this, new MuyaEditorErrorEventArgs(error));

    private void ThrowIfDisposed()
    {
        if (_disposed) throw new ObjectDisposedException(nameof(NativeWebViewMuyaHost));
    }
}

/// <summary>
/// Compatibility façade for the original scaffold. New code should depend on
/// <see cref="IMuyaHost"/> and construct <see cref="NativeWebViewMuyaHost"/>.
/// </summary>
public sealed class MuyaEditorHost : NativeWebViewMuyaHost
{
    public MuyaEditorHost(MuyaEditorHostOptions? options = null)
        : base(options)
    {
    }
}

public static class MuyaResourceLocator
{
    public static bool IsLocal(Uri resource) =>
        resource.IsAbsoluteUri &&
        (resource.Scheme.Equals(Uri.UriSchemeFile, StringComparison.OrdinalIgnoreCase) ||
         resource.Scheme.Equals("avares", StringComparison.OrdinalIgnoreCase));

    public static bool Exists(Uri resource) =>
        IsLocal(resource) &&
        (resource.Scheme.Equals("avares", StringComparison.OrdinalIgnoreCase) || File.Exists(resource.LocalPath));
}

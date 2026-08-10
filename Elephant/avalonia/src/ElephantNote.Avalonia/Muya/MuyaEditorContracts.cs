using System.ComponentModel;

namespace ElephantNote.Avalonia.Muya;

public sealed record MuyaOpenDocumentRequest
{
    public MuyaOpenDocumentRequest(string documentId, string content, string? relativePath = null, string? title = null)
    {
        DocumentId = Require(documentId, nameof(documentId));
        ArgumentNullException.ThrowIfNull(content);
        Content = content;
        RelativePath = relativePath;
        Title = title;
    }

    public string DocumentId { get; }
    public string Content { get; }
    public string? RelativePath { get; }
    public string? Title { get; }

    private static string Require(string value, string parameterName) =>
        string.IsNullOrWhiteSpace(value)
            ? throw new ArgumentException("A Muya document id is required.", parameterName)
            : value;
}

public sealed record MuyaDocument(
    string DocumentId,
    string Content,
    string? RelativePath,
    string? Title);

/// <summary>
/// Metadata emitted by the Rust/WASM parser after it validates a Markdown
/// snapshot. It is not an editor state and it is never used as the source of
/// truth for DOM, caret, selection, IME, or persistence.
/// </summary>
public sealed record MuyaValidationMetadata(string? Engine, ulong? Revision);

public sealed record MuyaSaveRequest
{
    public MuyaSaveRequest(
        string requestId,
        string documentId,
        string content,
        string? validationEngine = null,
        ulong? validationRevision = null)
    {
        RequestId = Require(requestId, nameof(requestId));
        DocumentId = Require(documentId, nameof(documentId));
        Content = content ?? throw new ArgumentNullException(nameof(content));
        Validation = new MuyaValidationMetadata(validationEngine, validationRevision);
    }

    public string RequestId { get; }
    public string DocumentId { get; }
    public string Content { get; }
    public MuyaValidationMetadata Validation { get; }
    public string? ValidationEngine => Validation.Engine;
    public ulong? ValidationRevision => Validation.Revision;

    // Compatibility aliases for the existing shell. They intentionally expose
    // validation metadata only; there is no Rust-owned editor mirror here.
    [EditorBrowsable(EditorBrowsableState.Never)]
    public string? Engine => ValidationEngine;

    [EditorBrowsable(EditorBrowsableState.Never)]
    public ulong? RustRevision => ValidationRevision;

    private static string Require(string value, string parameterName) =>
        string.IsNullOrWhiteSpace(value)
            ? throw new ArgumentException("A Muya value is required.", parameterName)
            : value;
}

public sealed record MuyaEditorError(
    string Code,
    string Message,
    string? DocumentId = null,
    string? RequestId = null,
    Exception? Exception = null);

public sealed class MuyaDocumentOpenedEventArgs(MuyaDocument document) : EventArgs
{
    public MuyaDocument Document { get; } = document ?? throw new ArgumentNullException(nameof(document));
}

public sealed class MuyaContentChangedEventArgs(
    string documentId,
    string content,
    bool isUserEdit,
    string? validationEngine = null,
    ulong? validationRevision = null) : EventArgs
{
    public string DocumentId { get; } = Require(documentId, nameof(documentId));
    public string Content { get; } = content ?? throw new ArgumentNullException(nameof(content));
    public bool IsUserEdit { get; } = isUserEdit;
    public MuyaValidationMetadata Validation { get; } = new(validationEngine, validationRevision);
    public string? ValidationEngine => Validation.Engine;
    public ulong? ValidationRevision => Validation.Revision;

    [EditorBrowsable(EditorBrowsableState.Never)]
    public string? Engine => ValidationEngine;

    [EditorBrowsable(EditorBrowsableState.Never)]
    public ulong? RustRevision => ValidationRevision;

    private static string Require(string value, string parameterName) =>
        string.IsNullOrWhiteSpace(value)
            ? throw new ArgumentException("A Muya document id is required.", parameterName)
            : value;
}

public sealed class MuyaSaveRequestedEventArgs(MuyaSaveRequest request) : EventArgs
{
    public MuyaSaveRequest Request { get; } = request ?? throw new ArgumentNullException(nameof(request));
}

public sealed class MuyaEditorErrorEventArgs(MuyaEditorError error) : EventArgs
{
    public MuyaEditorError Error { get; } = error ?? throw new ArgumentNullException(nameof(error));
}

public sealed class MuyaWebViewMessageEventArgs(string body) : EventArgs
{
    public string Body { get; } = body ?? throw new ArgumentNullException(nameof(body));
}

public sealed record MuyaEditorHostOptions
{
    public Uri? LocalMuyaResource { get; init; }
    public TimeSpan ReadyTimeout { get; init; } = TimeSpan.FromSeconds(10);
}

/// <summary>
/// Public Muya boundary. The host stores only the latest Markdown snapshot
/// observed from Muya; Muya JS owns the live editor state and Avalonia owns
/// filesystem/lifecycle/persistence concerns.
/// </summary>
public interface IMuyaHost : IAsyncDisposable
{
    MuyaDocument? Document { get; }
    string? Content { get; }
    bool IsOpen { get; }
    bool IsNativeWebViewAttached { get; }
    bool IsNativeWebViewReady { get; }

    event EventHandler<MuyaContentChangedEventArgs>? ContentChanged;
    event EventHandler<MuyaSaveRequestedEventArgs>? SaveRequested;
    event EventHandler<MuyaEditorErrorEventArgs>? Error;

    Task OpenDocumentAsync(
        MuyaOpenDocumentRequest request,
        CancellationToken cancellationToken = default);

    Task SetMarkdownAsync(
        string markdown,
        CancellationToken cancellationToken = default);

    /// <summary>Focuses the Muya JS editor when a ready NativeWebView exists.</summary>
    Task<bool> FocusAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// Waits for Muya's real JavaScript protocol handshake. Navigation alone
    /// is not proof that the editor bridge exists.
    /// </summary>
    Task<bool> WaitUntilReadyAsync(CancellationToken cancellationToken = default);
}

/// <summary>
/// Platform-neutral boundary for a NativeWebView control. This is an adapter
/// boundary, not a second document or editor state.
/// </summary>
public interface IMuyaWebViewAdapter
{
    Uri? Source { get; }

    event EventHandler? NavigationCompleted;
    event EventHandler<MuyaWebViewMessageEventArgs>? WebMessageReceived;

    void Navigate(Uri url);
    Task<string?> InvokeScriptAsync(string script, CancellationToken cancellationToken = default);
}

// Kept for source compatibility with the initial Avalonia scaffold.
[EditorBrowsable(EditorBrowsableState.Never)]
public interface IMuyaNativeWebView : IMuyaWebViewAdapter
{
}

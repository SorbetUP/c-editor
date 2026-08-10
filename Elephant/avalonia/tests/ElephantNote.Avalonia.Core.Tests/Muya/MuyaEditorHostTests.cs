using ElephantNote.Avalonia.Muya;

namespace ElephantNote.Avalonia.Core.Tests.Muya;

public sealed class MuyaHostContractTests
{
    [Fact]
    public async Task PublicHostContractOwnsOnlyTheLatestMarkdownSnapshot()
    {
        await using IMuyaHost host = new NativeWebViewMuyaHost();
        MuyaContentChangedEventArgs? changed = null;
        host.ContentChanged += (_, args) => changed = args;

        await host.OpenDocumentAsync(new MuyaOpenDocumentRequest("note-1", "# Draft", "notes/draft.md", "Draft"));
        await host.SetMarkdownAsync("# Updated");

        Assert.Equal("note-1", host.Document?.DocumentId);
        Assert.Equal("# Updated", host.Content);
        Assert.Equal("# Updated", changed?.Content);
        Assert.False(changed?.IsUserEdit);
        Assert.Null(changed?.ValidationRevision);
    }

    [Fact]
    public async Task NativeWebViewAdapterReceivesCommandsAndFocusWithoutBecomingTheStateOwner()
    {
        var path = Path.Combine(Path.GetTempPath(), $"muya-{Guid.NewGuid():N}.html");
        await File.WriteAllTextAsync(path, "<!doctype html>");
        try
        {
            var adapter = new RecordingWebViewAdapter();
            var concreteHost = new NativeWebViewMuyaHost(new MuyaEditorHostOptions { LocalMuyaResource = new Uri(path) });
            await using IMuyaHost host = concreteHost;
            await host.OpenDocumentAsync(new MuyaOpenDocumentRequest("note-1", "before"));

            Assert.True(await concreteHost.AttachNativeWebViewAsync(adapter));
            adapter.CompleteNavigation();
            Assert.Empty(adapter.Scripts);
            await concreteHost.HandleWebViewMessageAsync("{\"type\":\"ready\"}");
            await WaitUntilAsync(() => adapter.Scripts.Count >= 1);

            Assert.Equal(new Uri(path), adapter.NavigatedTo);
            Assert.Contains("\"type\":\"open-document\"", adapter.Scripts[0]);
            Assert.Contains("\"content\":\"before\"", adapter.Scripts[0]);

            Assert.True(await host.FocusAsync());
            Assert.Contains("__ELEPHANT_MUYA__?.focus?.()", adapter.Scripts[^1]);

            await host.SetMarkdownAsync("after");
            Assert.Contains("\"type\":\"set-content\"", adapter.Scripts[^1]);
            Assert.Equal("after", host.Content);
        }
        finally
        {
            File.Delete(path);
        }
    }

    [Fact]
    public async Task MissingLocalResourceLeavesTheHostWithoutAWebView()
    {
        await using var host = new NativeWebViewMuyaHost(new MuyaEditorHostOptions
        {
            LocalMuyaResource = new Uri(Path.Combine(Path.GetTempPath(), $"missing-{Guid.NewGuid():N}.html"))
        });

        Assert.False(await host.AttachNativeWebViewAsync(new RecordingWebViewAdapter()));
        Assert.False(host.IsNativeWebViewAttached);
        Assert.False(await host.FocusAsync());
    }

    [Fact]
    public async Task WebViewMessagesUpdateTheSnapshotAndCarryValidationMetadataOnly()
    {
        await using IMuyaHost host = new NativeWebViewMuyaHost();
        await host.OpenDocumentAsync(new MuyaOpenDocumentRequest("note-1", "before"));
        MuyaContentChangedEventArgs? changed = null;
        MuyaSaveRequest? saved = null;
        MuyaEditorError? error = null;
        host.ContentChanged += (_, args) => changed = args;
        host.SaveRequested += (_, args) => saved = args.Request;
        host.Error += (_, args) => error = args.Error;

        var concreteHost = (NativeWebViewMuyaHost)host;
        await concreteHost.HandleWebViewMessageAsync("{\"type\":\"content-changed\",\"documentId\":\"note-1\",\"content\":\"edited\",\"engine\":\"muya-js+muya-rust\",\"rustRevision\":1}");
        await concreteHost.HandleWebViewMessageAsync("{\"type\":\"save-request\",\"documentId\":\"note-1\",\"requestId\":\"save-1\",\"content\":\"edited\",\"engine\":\"muya-js+muya-rust\",\"rustRevision\":1}");
        await concreteHost.HandleWebViewMessageAsync("{\"type\":\"error\",\"documentId\":\"note-1\",\"code\":\"render-failed\",\"message\":\"Muya could not render\"}");

        Assert.Equal("edited", host.Content);
        Assert.Equal("edited", changed?.Content);
        Assert.True(changed?.IsUserEdit);
        Assert.Equal("muya-js+muya-rust", changed?.ValidationEngine);
        Assert.Equal((ulong?)1, changed?.ValidationRevision);
        Assert.Equal("save-1", saved?.RequestId);
        Assert.Equal("edited", saved?.Content);
        Assert.Equal("muya-js+muya-rust", saved?.ValidationEngine);
        Assert.Equal((ulong?)1, saved?.ValidationRevision);
        Assert.Equal("render-failed", error?.Code);
        Assert.Equal("Muya could not render", error?.Message);
    }

    [Fact]
    public async Task InvalidOrForeignMessagesAreReportedAsErrors()
    {
        await using var host = new NativeWebViewMuyaHost();
        var errors = new List<MuyaEditorError>();
        host.Error += (_, args) => errors.Add(args.Error);

        await host.HandleWebViewMessageAsync("not-json");
        await host.HandleWebViewMessageAsync("{\"type\":\"content-changed\",\"documentId\":\"note-1\",\"content\":\"edited\"}");

        Assert.Equal(new[] { "invalid-webview-message", "document-not-open" }, errors.Select(item => item.Code));

        await host.OpenDocumentAsync(new MuyaOpenDocumentRequest("note-1", "before"));
        await host.HandleWebViewMessageAsync("{\"type\":\"content-changed\",\"documentId\":\"other\",\"content\":\"edited\"}");

        Assert.Equal("document-mismatch", errors[^1].Code);
        Assert.Equal("before", host.Content);
    }

    [Fact]
    public async Task CompatibilityFacadeStillImplementsThePublicContract()
    {
        await using IMuyaHost host = new MuyaEditorHost();

        await host.OpenDocumentAsync(new MuyaOpenDocumentRequest("note-1", "before"));
        await ((MuyaEditorHost)host).SetContentAsync("after");

        Assert.Equal("after", host.Content);
    }

    private static async Task WaitUntilAsync(Func<bool> condition)
    {
        for (var attempt = 0; attempt < 50; attempt++)
        {
            if (condition()) return;
            await Task.Delay(10);
        }

        Assert.True(condition(), "The NativeWebView adapter did not receive the expected script.");
    }

    // This records the transport boundary only. The production host above is
    // never replaced by a mock Muya implementation or a fake editor state.
    private sealed class RecordingWebViewAdapter : IMuyaWebViewAdapter
    {
        public Uri? Source { get; private set; }
        public Uri? NavigatedTo { get; private set; }
        public List<string> Scripts { get; } = [];

        public event EventHandler? NavigationCompleted;
        public event EventHandler<MuyaWebViewMessageEventArgs>? WebMessageReceived;

        public void Navigate(Uri url)
        {
            Source = url;
            NavigatedTo = url;
        }

        public Task<string?> InvokeScriptAsync(string script, CancellationToken cancellationToken = default)
        {
            cancellationToken.ThrowIfCancellationRequested();
            Scripts.Add(script);
            return Task.FromResult<string?>(null);
        }

        public void CompleteNavigation() => NavigationCompleted?.Invoke(this, EventArgs.Empty);

        public void Send(string body) => WebMessageReceived?.Invoke(this, new MuyaWebViewMessageEventArgs(body));
    }
}

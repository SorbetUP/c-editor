using Avalonia.Controls;

namespace ElephantNote.Avalonia.Muya;

/// <summary>
/// Adapts Avalonia's platform WebView control to the shell-independent Muya
/// protocol. This is the only web surface in the native application and owns
/// no Markdown/editor state.
/// </summary>
public sealed class AvaloniaMuyaWebView : NativeWebView, IMuyaWebViewAdapter
{
    private event EventHandler? NavigationCompletedForMuya;
    private event EventHandler<MuyaWebViewMessageEventArgs>? WebMessageReceivedForMuya;

    public AvaloniaMuyaWebView()
    {
        base.NavigationCompleted += (_, _) => NavigationCompletedForMuya?.Invoke(this, EventArgs.Empty);
        base.WebMessageReceived += (_, args) =>
            WebMessageReceivedForMuya?.Invoke(this, new MuyaWebViewMessageEventArgs(args.Body ?? string.Empty));
    }

    Uri? IMuyaWebViewAdapter.Source => Source;

    event EventHandler? IMuyaWebViewAdapter.NavigationCompleted
    {
        add => NavigationCompletedForMuya += value;
        remove => NavigationCompletedForMuya -= value;
    }

    event EventHandler<MuyaWebViewMessageEventArgs>? IMuyaWebViewAdapter.WebMessageReceived
    {
        add => WebMessageReceivedForMuya += value;
        remove => WebMessageReceivedForMuya -= value;
    }

    void IMuyaWebViewAdapter.Navigate(Uri url) => base.Navigate(url);

    async Task<string?> IMuyaWebViewAdapter.InvokeScriptAsync(
        string script,
        CancellationToken cancellationToken)
    {
        cancellationToken.ThrowIfCancellationRequested();
        return await InvokeScript(script).ConfigureAwait(false);
    }
}

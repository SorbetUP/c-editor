using Avalonia.Input.Platform;

namespace ElephantNote.Avalonia.Services.Platform;

public sealed class NativeClipboardService : IClipboardService
{
    private readonly Func<IClipboard?> _clipboardProvider;

    public NativeClipboardService(Func<IClipboard?> clipboardProvider)
    {
        ArgumentNullException.ThrowIfNull(clipboardProvider);
        _clipboardProvider = clipboardProvider;
    }

    public async Task<string?> GetTextAsync(CancellationToken cancellationToken = default)
    {
        cancellationToken.ThrowIfCancellationRequested();
        try
        {
            return await RequireClipboard().TryGetTextAsync();
        }
        catch (NotSupportedException error)
        {
            throw new PlatformCapabilityUnavailableException(
                PlatformCapability.Clipboard,
                "The native clipboard does not support reading text.",
                error);
        }
    }

    public async Task SetTextAsync(string text, CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(text);
        cancellationToken.ThrowIfCancellationRequested();
        try
        {
            await RequireClipboard().SetTextAsync(text);
        }
        catch (NotSupportedException error)
        {
            throw new PlatformCapabilityUnavailableException(
                PlatformCapability.Clipboard,
                "The native clipboard does not support writing text.",
                error);
        }
    }

    private IClipboard RequireClipboard() =>
        _clipboardProvider() ?? throw new PlatformCapabilityUnavailableException(
            PlatformCapability.Clipboard,
            "The native clipboard is unavailable because no Avalonia top-level is attached.");
}

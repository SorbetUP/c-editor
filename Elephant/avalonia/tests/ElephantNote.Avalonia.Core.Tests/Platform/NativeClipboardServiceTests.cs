using ElephantNote.Avalonia.Services.Platform;

namespace ElephantNote.Avalonia.Core.Tests.Platform;

public sealed class NativeClipboardServiceTests
{
    [Fact]
    public async Task ReadingWithoutAnAttachedTopLevelReportsUnavailableClipboard()
    {
        var service = new NativeClipboardService(() => null);

        var error = await Assert.ThrowsAsync<PlatformCapabilityUnavailableException>(
            () => service.GetTextAsync());

        Assert.Equal(PlatformCapability.Clipboard, error.Capability);
    }

    [Fact]
    public async Task WritingWithoutAnAttachedTopLevelReportsUnavailableClipboard()
    {
        var service = new NativeClipboardService(() => null);

        var error = await Assert.ThrowsAsync<PlatformCapabilityUnavailableException>(
            () => service.SetTextAsync("text"));

        Assert.Equal(PlatformCapability.Clipboard, error.Capability);
    }
}

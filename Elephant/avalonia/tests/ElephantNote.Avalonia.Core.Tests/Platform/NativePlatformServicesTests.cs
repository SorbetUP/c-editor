using ElephantNote.Avalonia.Services.Platform;

namespace ElephantNote.Avalonia.Core.Tests.Platform;

public sealed class NativePlatformServicesTests
{
    [Fact]
    public void GroupsRealNativeCapabilitiesAndExplicitUnavailableBoundaries()
    {
        IPlatformServices services = new NativePlatformServices();

        Assert.IsType<NativeFileStore>(services.FileStore);
        Assert.IsType<UnavailableSyncService>(services.Sync);
        Assert.IsType<NativeExternalFileOpener>(services.ExternalFileOpener);
        Assert.IsType<NativeClipboardService>(services.Clipboard);
        Assert.IsType<NativeExecutionService>(services.Execution);
    }
}

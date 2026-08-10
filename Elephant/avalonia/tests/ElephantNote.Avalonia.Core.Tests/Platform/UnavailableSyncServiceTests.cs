using ElephantNote.Avalonia.Services.Platform;

namespace ElephantNote.Avalonia.Core.Tests.Platform;

public sealed class UnavailableSyncServiceTests
{
    [Fact]
    public async Task ReportsTheMissingNativeCapabilityInsteadOfReturningSuccess()
    {
        var error = await Assert.ThrowsAsync<PlatformCapabilityUnavailableException>(
            () => new UnavailableSyncService().SyncAsync());

        Assert.Equal(PlatformCapability.Sync, error.Capability);
        Assert.Contains("not available", error.Message, StringComparison.OrdinalIgnoreCase);
    }
}

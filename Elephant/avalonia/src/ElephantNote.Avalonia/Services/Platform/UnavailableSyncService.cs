namespace ElephantNote.Avalonia.Services.Platform;

/// <summary>Explicit native boundary until a real synchronization transport is provided.</summary>
public sealed class UnavailableSyncService : ISyncService
{
    public Task SyncAsync(CancellationToken cancellationToken = default)
    {
        cancellationToken.ThrowIfCancellationRequested();
        throw new PlatformCapabilityUnavailableException(
            PlatformCapability.Sync,
            "Vault synchronization is not available in the native Avalonia host.");
    }
}

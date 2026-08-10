namespace ElephantNote.Avalonia.Services.Platform;

/// <summary>Represents vault synchronization independently of its transport.</summary>
public interface ISyncService
{
    Task SyncAsync(CancellationToken cancellationToken = default);
}

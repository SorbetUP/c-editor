namespace ElephantNote.Avalonia.Services.Platform;

/// <summary>Opens a local file using the host platform.</summary>
public interface IExternalFileOpener
{
    Task OpenAsync(string path, CancellationToken cancellationToken = default);
}

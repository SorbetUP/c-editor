namespace ElephantNote.Avalonia.Services.Platform;

/// <summary>Stores text without coupling callers to a local filesystem or a web backend.</summary>
public interface IFileStore
{
    Task<string> ReadTextAsync(string path, CancellationToken cancellationToken = default);

    Task WriteTextAsync(string path, string content, CancellationToken cancellationToken = default);

    Task<bool> ExistsAsync(string path, CancellationToken cancellationToken = default);
}

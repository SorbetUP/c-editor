using System.Text;

namespace ElephantNote.Avalonia.Services.Platform;

public sealed class NativeFileStore : IFileStore
{
    public Task<string> ReadTextAsync(string path, CancellationToken cancellationToken = default)
    {
        cancellationToken.ThrowIfCancellationRequested();
        return File.ReadAllTextAsync(RequirePath(path), Encoding.UTF8, cancellationToken);
    }

    public async Task WriteTextAsync(string path, string content, CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(content);
        cancellationToken.ThrowIfCancellationRequested();

        var fullPath = RequirePath(path);
        var directory = Path.GetDirectoryName(fullPath);
        if (!string.IsNullOrEmpty(directory)) Directory.CreateDirectory(directory);
        await File.WriteAllTextAsync(fullPath, content, Encoding.UTF8, cancellationToken);
    }

    public Task<bool> ExistsAsync(string path, CancellationToken cancellationToken = default)
    {
        cancellationToken.ThrowIfCancellationRequested();
        return Task.FromResult(File.Exists(RequirePath(path)));
    }

    private static string RequirePath(string path)
    {
        if (string.IsNullOrWhiteSpace(path)) throw new ArgumentException("A file path is required.", nameof(path));
        if (path.Contains('\0')) throw new ArgumentException("A file path cannot contain a null character.", nameof(path));
        return Path.GetFullPath(path);
    }
}

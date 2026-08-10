using System.Diagnostics;
using System.Text;
using ElephantNote.Avalonia.Domain;

namespace ElephantNote.Avalonia.Services.Storage;

/// <summary>
/// Local implementation of <see cref="IFileStore"/>.
/// The temporary file is always created next to the destination so the final
/// rename remains atomic on the same filesystem.
/// </summary>
public sealed class LocalFileStore : IFileStore
{
    private static readonly UTF8Encoding StrictUtf8 = new(encoderShouldEmitUTF8Identifier: false, throwOnInvalidBytes: true);
    private readonly string _rootPath;

    public LocalFileStore(string rootPath)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(rootPath);
        _rootPath = Path.GetFullPath(rootPath);
        Directory.CreateDirectory(_rootPath);
    }

    public string RootPath => _rootPath;

    public Task<bool> ExistsAsync(RelativePath path, CancellationToken cancellationToken = default)
    {
        cancellationToken.ThrowIfCancellationRequested();
        var fullPath = ResolvePath(path, allowRoot: true);
        return Task.FromResult(File.Exists(fullPath) || Directory.Exists(fullPath));
    }

    public async Task<byte[]> ReadBytesAsync(RelativePath path, CancellationToken cancellationToken = default)
    {
        var requestId = AppLog.Start("storage.read", Describe(path));
        var timer = Stopwatch.StartNew();
        try
        {
            var fullPath = ResolvePath(path, allowRoot: false);
            await using var stream = new FileStream(
                fullPath,
                FileMode.Open,
                FileAccess.Read,
                FileShare.ReadWrite | FileShare.Delete,
                bufferSize: 64 * 1024,
                options: FileOptions.Asynchronous | FileOptions.SequentialScan);
            await using var buffer = new MemoryStream();
            await stream.CopyToAsync(buffer, cancellationToken);
            var result = buffer.ToArray();
            AppLog.Complete("storage.read", requestId, timer, $"bytes={result.Length}");
            return result;
        }
        catch (Exception error)
        {
            AppLog.Fail("storage.read", requestId, timer, error);
            throw;
        }
    }

    public async Task<string> ReadTextAsync(RelativePath path, CancellationToken cancellationToken = default)
    {
        var bytes = await ReadBytesAsync(path, cancellationToken);
        return StrictUtf8.GetString(bytes);
    }

    public Task WriteTextAtomicallyAsync(
        RelativePath path,
        string content,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(content);
        return WriteBytesAtomicallyAsync(path, StrictUtf8.GetBytes(content), cancellationToken);
    }

    public async Task WriteBytesAtomicallyAsync(
        RelativePath path,
        ReadOnlyMemory<byte> content,
        CancellationToken cancellationToken = default)
    {
        var requestId = AppLog.Start("storage.write", Describe(path));
        var timer = Stopwatch.StartNew();
        string? temporary = null;
        try
        {
            var fullPath = ResolvePath(path, allowRoot: false);
            var directory = Path.GetDirectoryName(fullPath)
                ?? throw new IOException("The requested file has no parent directory.");
            Directory.CreateDirectory(directory);
            RejectReparsePoints(fullPath);

            temporary = Path.Combine(directory, $".{Path.GetFileName(fullPath)}.{Guid.NewGuid():N}.tmp");
            await using (var stream = new FileStream(
                temporary,
                FileMode.CreateNew,
                FileAccess.Write,
                FileShare.None,
                bufferSize: 64 * 1024,
                options: FileOptions.Asynchronous | FileOptions.SequentialScan))
            {
                await stream.WriteAsync(content, cancellationToken);
                await stream.FlushAsync(cancellationToken);
                stream.Flush(flushToDisk: true);
            }

            cancellationToken.ThrowIfCancellationRequested();
            File.Move(temporary, fullPath, overwrite: true);
            temporary = null;
            AppLog.Complete("storage.write", requestId, timer, $"bytes={content.Length}");
        }
        catch (Exception error)
        {
            AppLog.Fail("storage.write", requestId, timer, error);
            throw;
        }
        finally
        {
            if (temporary is not null && File.Exists(temporary)) File.Delete(temporary);
        }
    }

    public Task EnsureDirectoryAsync(RelativePath path, CancellationToken cancellationToken = default)
    {
        var requestId = AppLog.Start("storage.directory.create", Describe(path));
        var timer = Stopwatch.StartNew();
        try
        {
            cancellationToken.ThrowIfCancellationRequested();
            var fullPath = ResolvePath(path, allowRoot: true);
            Directory.CreateDirectory(fullPath);
            RejectReparsePoints(fullPath);
            AppLog.Complete("storage.directory.create", requestId, timer);
            return Task.CompletedTask;
        }
        catch (Exception error)
        {
            AppLog.Fail("storage.directory.create", requestId, timer, error);
            throw;
        }
    }

    public Task<IReadOnlyList<StoredFile>> EnumerateAsync(
        RelativePath directory = default,
        string searchPattern = "*",
        bool recursive = false,
        CancellationToken cancellationToken = default)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(searchPattern);
        if (searchPattern.Contains(Path.DirectorySeparatorChar) || searchPattern.Contains(Path.AltDirectorySeparatorChar))
        {
            throw new ArgumentException("A search pattern cannot contain a directory separator.", nameof(searchPattern));
        }

        var requestId = AppLog.Start("storage.enumerate", Describe(directory));
        var timer = Stopwatch.StartNew();
        try
        {
            var fullDirectory = ResolvePath(directory, allowRoot: true);
            var options = new EnumerationOptions
            {
                RecurseSubdirectories = recursive,
                IgnoreInaccessible = false,
                ReturnSpecialDirectories = false,
                AttributesToSkip = FileAttributes.ReparsePoint
            };
            var entries = new List<StoredFile>();
            foreach (var info in new DirectoryInfo(fullDirectory).EnumerateFileSystemInfos(searchPattern, options))
            {
                cancellationToken.ThrowIfCancellationRequested();
                var relative = RelativePath.Parse(Path.GetRelativePath(_rootPath, info.FullName));
                entries.Add(new StoredFile(relative, info is DirectoryInfo, info.LastWriteTimeUtc));
            }

            AppLog.Complete("storage.enumerate", requestId, timer, $"count={entries.Count}");
            return Task.FromResult<IReadOnlyList<StoredFile>>(entries);
        }
        catch (Exception error)
        {
            AppLog.Fail("storage.enumerate", requestId, timer, error);
            throw;
        }
    }

    private string ResolvePath(RelativePath path, bool allowRoot)
    {
        if (!allowRoot && path.IsRoot) throw new ArgumentException("A file path is required.", nameof(path));

        var value = path.Value ?? string.Empty;
        var candidate = Path.GetFullPath(Path.Combine(_rootPath, value.Replace('/', Path.DirectorySeparatorChar)));
        var relative = Path.GetRelativePath(_rootPath, candidate);
        if (Path.IsPathRooted(relative) || relative == ".." || relative.StartsWith($"..{Path.DirectorySeparatorChar}", StringComparison.Ordinal))
        {
            throw new UnauthorizedAccessException("The requested path leaves the active vault.");
        }

        RejectReparsePoints(candidate);
        return candidate;
    }

    private void RejectReparsePoints(string path)
    {
        var current = Path.GetFullPath(path);
        while (current.Length > _rootPath.Length)
        {
            if ((File.Exists(current) || Directory.Exists(current)) && File.GetAttributes(current).HasFlag(FileAttributes.ReparsePoint))
            {
                throw new UnauthorizedAccessException("The requested path crosses a symbolic link or reparse point.");
            }

            current = Directory.GetParent(current)?.FullName ?? _rootPath;
        }
    }

    private static string Describe(RelativePath path) => $"relativePath={path.Value}";
}

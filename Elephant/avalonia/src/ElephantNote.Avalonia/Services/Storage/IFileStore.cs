using ElephantNote.Avalonia.Domain;

namespace ElephantNote.Avalonia.Services.Storage;

/// <summary>
/// Root-scoped filesystem boundary for local vault persistence.
/// Every path accepted by this contract is relative to <see cref="RootPath"/>.
/// </summary>
public interface IFileStore
{
    string RootPath { get; }

    Task<bool> ExistsAsync(RelativePath path, CancellationToken cancellationToken = default);

    Task<byte[]> ReadBytesAsync(RelativePath path, CancellationToken cancellationToken = default);

    Task<string> ReadTextAsync(RelativePath path, CancellationToken cancellationToken = default);

    Task WriteBytesAtomicallyAsync(
        RelativePath path,
        ReadOnlyMemory<byte> content,
        CancellationToken cancellationToken = default);

    Task WriteTextAtomicallyAsync(
        RelativePath path,
        string content,
        CancellationToken cancellationToken = default);

    Task EnsureDirectoryAsync(RelativePath path, CancellationToken cancellationToken = default);

    Task<IReadOnlyList<StoredFile>> EnumerateAsync(
        RelativePath directory = default,
        string searchPattern = "*",
        bool recursive = false,
        CancellationToken cancellationToken = default);
}

public sealed record StoredFile(
    RelativePath Path,
    bool IsDirectory,
    DateTimeOffset LastWriteTimeUtc);

using ElephantNote.Avalonia.Domain;

namespace ElephantNote.Avalonia.Services.Storage;

/// <summary>
/// Vault-scoped entry point for consumers that need local file persistence.
/// It leaves the existing VaultRepository API untouched while making the
/// storage dependency explicit for new native vault services.
/// </summary>
public sealed class VaultFileStoreAdapter
{
    public VaultFileStoreAdapter(VaultDescriptor vault)
        : this(new LocalFileStore(vault.Path))
    {
    }

    public VaultFileStoreAdapter(IFileStore fileStore)
    {
        ArgumentNullException.ThrowIfNull(fileStore);
        FileStore = fileStore;
    }

    public IFileStore FileStore { get; }

    public Task<string> ReadNoteAsync(string relativePath, CancellationToken cancellationToken = default)
    {
        var path = RelativePath.Parse(relativePath);
        EnsureMarkdown(path);
        return FileStore.ReadTextAsync(path, cancellationToken);
    }

    public Task SaveNoteAsync(
        string relativePath,
        string content,
        CancellationToken cancellationToken = default)
    {
        var path = RelativePath.Parse(relativePath);
        EnsureMarkdown(path);
        return FileStore.WriteTextAtomicallyAsync(path, content, cancellationToken);
    }

    private static void EnsureMarkdown(RelativePath path)
    {
        if (!path.Value.EndsWith(".md", StringComparison.OrdinalIgnoreCase))
        {
            throw new InvalidDataException("Only Markdown notes can be opened or saved.");
        }
    }
}

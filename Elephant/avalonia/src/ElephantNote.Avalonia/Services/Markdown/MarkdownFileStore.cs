using ElephantNote.Avalonia.Domain;
using ElephantNote.Avalonia.Services.Storage;

namespace ElephantNote.Avalonia.Services.Markdown;

public sealed class MarkdownFileStore
{
    public async Task<MarkdownDocument> ReadAsync(string vaultRoot, RelativePath path, CancellationToken cancellationToken = default)
    {
        var markdown = await new LocalFileStore(vaultRoot).ReadTextAsync(path, cancellationToken);
        return MarkdownDocument.Parse(markdown);
    }

    public Task WriteAsync(string vaultRoot, RelativePath path, MarkdownDocument document, CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(document);
        return new LocalFileStore(vaultRoot).WriteTextAtomicallyAsync(path, document.ToMarkdown(), cancellationToken);
    }
}

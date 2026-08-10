using System.Text.Json;
using System.Text.Json.Serialization;
using ElephantNote.Avalonia.Domain;
using ElephantNote.Avalonia.Services.Storage;

namespace ElephantNote.Avalonia.Services.Markdown;

public sealed class WorkspaceFileStore
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        WriteIndented = true,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
        PropertyNameCaseInsensitive = true
    };

    public async Task<WorkspaceDocument> LoadAsync(string vaultRoot, CancellationToken cancellationToken = default)
    {
        var store = new LocalFileStore(vaultRoot);
        var path = ResolveWorkspacePath(vaultRoot);
        var relativePath = RelativePath.Parse(Path.GetRelativePath(store.RootPath, path));
        if (!await store.ExistsAsync(relativePath, cancellationToken))
        {
            return WorkspaceDocument.Create(new DirectoryInfo(store.RootPath).Name);
        }

        var json = await store.ReadTextAsync(relativePath, cancellationToken);
        if (json.StartsWith('\uFEFF')) json = json[1..];
        var source = JsonSerializer.Deserialize<WorkspaceDocument>(
            json,
            JsonOptions)
            ?? throw new JsonException("The workspace JSON is empty.");
        var result = WorkspaceMigrations.Migrate(source);
        if (result.Changed) await SaveAsync(vaultRoot, result.Workspace, cancellationToken);
        return result.Workspace;
    }

    public async Task SaveAsync(string vaultRoot, WorkspaceDocument workspace, CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(workspace);
        var store = new LocalFileStore(vaultRoot);
        var path = ResolveWorkspacePath(vaultRoot);
        var relativePath = RelativePath.Parse(Path.GetRelativePath(store.RootPath, path));
        var json = JsonSerializer.Serialize(workspace, JsonOptions);
        await store.WriteTextAtomicallyAsync(relativePath, json, cancellationToken);
    }

    public static string WorkspacePath(string vaultRoot) =>
        Path.Combine(Path.GetFullPath(vaultRoot), ".elephantnote", "workspace.json");

    public static string CanonicalWorkspacePath(string vaultRoot) =>
        Path.Combine(Path.GetFullPath(vaultRoot), ".elephantnote", "config", "workspace.json");

    private static string ResolveWorkspacePath(string vaultRoot)
    {
        var canonical = CanonicalWorkspacePath(vaultRoot);
        return File.Exists(canonical) ? canonical : WorkspacePath(vaultRoot);
    }
}

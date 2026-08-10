namespace ElephantNote.Avalonia.Domain;

public static class WorkspaceMigrations
{
    public const int CurrentSchemaVersion = 2;

    public static WorkspaceMigrationResult Migrate(WorkspaceDocument source)
    {
        ArgumentNullException.ThrowIfNull(source);
        var workspace = source.Copy();
        var applied = new List<int>();
        var version = workspace.SchemaVersion ?? 0;

        if (version == 0)
        {
            workspace.Sidebar = FlattenSidebar(workspace.Sidebar);
            workspace.SchemaVersion = 1;
            applied.Add(1);
            version = 1;
        }
        if (version == 1)
        {
            workspace.Sync ??= new WorkspaceSyncState();
            workspace.Features ??= [];
            workspace.SchemaVersion = 2;
            applied.Add(2);
        }

        return new WorkspaceMigrationResult(workspace, applied.Count > 0, applied);
    }

    private static List<SidebarEntry> FlattenSidebar(IEnumerable<SidebarEntry> items)
    {
        var result = new List<SidebarEntry>();
        foreach (var item in items)
        {
            if (IsSidebarEntry(item))
            {
                var copy = item.Copy();
                copy.Items = null;
                copy.Id ??= CreateId($"{copy.Type}-{copy.Path}");
                copy.Title = string.IsNullOrWhiteSpace(copy.Title) ? TitleFromPath(copy.Path) : copy.Title;
                result.Add(copy);
                continue;
            }

            foreach (var child in item.Items ?? [])
            {
                if (!IsSidebarEntry(child)) continue;
                var copy = child.Copy();
                copy.Items = null;
                copy.Type = copy.Type == "note" ? "note" : "folder";
                copy.Id ??= CreateId($"{copy.Type}-{copy.Path}");
                copy.Title = string.IsNullOrWhiteSpace(copy.Title) ? TitleFromPath(copy.Path) : copy.Title;
                copy.Collapsed = false;
                result.Add(copy);
            }
        }
        return result;
    }

    private static bool IsSidebarEntry(SidebarEntry item) =>
        item.Path is { IsRoot: false } && (item.Type is "note" or "folder");

    private static string TitleFromPath(RelativePath path) =>
        path.FileName.EndsWith(".md", StringComparison.OrdinalIgnoreCase)
            ? path.FileName[..^3]
            : path.FileName;

    private static string CreateId(string value)
    {
        var id = string.Join('-', value
            .ToLowerInvariant()
            .Split(['/', '\\', ' ', '.', ':', '_', '-'], StringSplitOptions.RemoveEmptyEntries)
            .Select(part => new string(part.Where(char.IsLetterOrDigit).ToArray()))
            .Where(part => part.Length > 0));
        return id.Length == 0 ? "vault" : id;
    }
}

public sealed record WorkspaceMigrationResult(
    WorkspaceDocument Workspace,
    bool Changed,
    IReadOnlyList<int> AppliedMigrations);

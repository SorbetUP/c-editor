using System.Text.Json;
using ElephantNote.Avalonia.Domain;

namespace ElephantNote.Avalonia.Core.Tests.Domain;

public sealed class WorkspaceMigrationTests
{
    [Fact]
    public void MigratesNestedSidebarAndKeepsUnknownRootAndChildFields()
    {
        var source = JsonSerializer.Deserialize<WorkspaceDocument>("""
            {
              "version": 1,
              "vaultName": "Personal",
              "futureSetting": { "enabled": true },
              "sidebar": [{
                "id": "compatibility",
                "title": "Compatibility",
                "items": [{
                  "id": "note-1",
                  "title": "Note",
                  "type": "note",
                  "path": "Notes/Note.md",
                  "futureChild": "preserved"
                }]
              }]
            }
            """)!;

        var result = WorkspaceMigrations.Migrate(source);

        Assert.True(result.Changed);
        Assert.Equal(2, result.Workspace.SchemaVersion.GetValueOrDefault());
        Assert.Equal("true", result.Workspace.Extra["futureSetting"].GetProperty("enabled").GetRawText());
        Assert.Single(result.Workspace.Sidebar);
        Assert.Equal("Notes/Note.md", result.Workspace.Sidebar[0].Path.Value);
        Assert.Equal("preserved", result.Workspace.Sidebar[0].Extra["futureChild"].GetString());
        Assert.Null(source.SchemaVersion);
        Assert.NotNull(result.Workspace.Sync);
        Assert.NotNull(result.Workspace.Features);
    }

    [Fact]
    public void CurrentWorkspaceIsNotRewrittenByMigration()
    {
        var workspace = WorkspaceDocument.Create("Personal");
        workspace.SchemaVersion = WorkspaceMigrations.CurrentSchemaVersion;
        workspace.Extra["custom"] = JsonDocument.Parse("{\"keep\":true}").RootElement.Clone();

        var result = WorkspaceMigrations.Migrate(workspace);

        Assert.False(result.Changed);
        Assert.Equal("true", result.Workspace.Extra["custom"].GetProperty("keep").GetRawText());
    }
}

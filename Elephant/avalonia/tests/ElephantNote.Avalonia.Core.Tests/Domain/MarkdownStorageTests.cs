using System.Text;
using ElephantNote.Avalonia.Domain;
using ElephantNote.Avalonia.Services.Markdown;

namespace ElephantNote.Avalonia.Core.Tests.Domain;

public sealed class MarkdownStorageTests
{
    [Fact]
    public async Task CurrentWorkspaceJsonIsReadWithoutRewritingUnknownFormattingOrKeys()
    {
        var root = Path.Combine(Path.GetTempPath(), $"elephant-domain-{Guid.NewGuid():N}");
        try
        {
            var path = WorkspaceFileStore.WorkspacePath(root);
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            const string json = "{\n  \"schemaVersion\": 2,\n  \"sidebar\": [],\n  \"custom\": { \"keep\": true }\n}\n";
            await File.WriteAllTextAsync(path, json, Encoding.UTF8);

            var workspace = await new WorkspaceFileStore().LoadAsync(root);

            Assert.Equal(2, workspace.SchemaVersion.GetValueOrDefault());
            Assert.Equal(json, await File.ReadAllTextAsync(path));
            Assert.Equal("true", workspace.Extra["custom"].GetProperty("keep").GetRawText());
        }
        finally
        {
            if (Directory.Exists(root)) Directory.Delete(root, recursive: true);
        }
    }

    [Fact]
    public async Task MarkdownStoreWritesEditedBodyWithOriginalFrontmatter()
    {
        var root = Path.Combine(Path.GetTempPath(), $"elephant-markdown-{Guid.NewGuid():N}");
        try
        {
            var document = MarkdownDocument.Parse("---\ntitle: Keep\nunknown: yes\n---\n\nOriginal").WithBody("\nUpdated");
            await new MarkdownFileStore().WriteAsync(root, RelativePath.Parse("Notes/Note.md"), document);

            Assert.Equal("---\ntitle: Keep\nunknown: yes\n---\n\nUpdated", await File.ReadAllTextAsync(Path.Combine(root, "Notes", "Note.md")));
        }
        finally
        {
            if (Directory.Exists(root)) Directory.Delete(root, recursive: true);
        }
    }

    [Fact]
    public async Task ExistingCompatibilityWorkspaceLocationIsPreserved()
    {
        var root = Path.Combine(Path.GetTempPath(), $"elephant-workspace-{Guid.NewGuid():N}");
        try
        {
            var compatibilityPath = WorkspaceFileStore.WorkspacePath(root);
            Directory.CreateDirectory(Path.GetDirectoryName(compatibilityPath)!);
            await File.WriteAllTextAsync(compatibilityPath, "{\"schemaVersion\":2,\"sidebar\":[]}");

            await new WorkspaceFileStore().SaveAsync(root, new WorkspaceDocument { SchemaVersion = 2 });

            Assert.True(File.Exists(compatibilityPath));
            Assert.False(File.Exists(WorkspaceFileStore.CanonicalWorkspacePath(root)));
        }
        finally
        {
            if (Directory.Exists(root)) Directory.Delete(root, recursive: true);
        }
    }
}

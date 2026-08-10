using System.Collections.Concurrent;
using System.Text;
using ElephantNote.Avalonia.Domain;
using ElephantNote.Avalonia.Services;
using ElephantNote.Avalonia.Services.Storage;

namespace ElephantNote.Avalonia.Core.Tests.Storage;

public sealed class FileStoreTests
{
    [Theory]
    [InlineData("../outside.md")]
    [InlineData("folder/../../outside.md")]
    [InlineData("/tmp/outside.md")]
    [InlineData("C:/outside.md")]
    public async Task RelativePathsCannotEscapeTheVault(string value)
    {
        var root = CreateRoot();
        try
        {
            var adapter = new VaultFileStoreAdapter(new VaultDescriptor("test", "Test", root));
            await Assert.ThrowsAsync<ArgumentException>(() => adapter.SaveNoteAsync(value, "escape"));
            Assert.False(File.Exists(Path.Combine(Path.GetDirectoryName(root)!, "outside.md")));
        }
        finally
        {
            DeleteRoot(root);
        }
    }

    [Fact]
    public async Task RefusesAReparsePointInsideTheVault()
    {
        var root = CreateRoot();
        var outside = CreateRoot();
        try
        {
            await File.WriteAllTextAsync(Path.Combine(outside, "outside.md"), "must stay outside");
            var link = Path.Combine(root, "linked");
            Directory.CreateSymbolicLink(link, outside);
            var store = new LocalFileStore(root);

            await Assert.ThrowsAsync<UnauthorizedAccessException>(() =>
                store.ReadTextAsync(RelativePath.Parse("linked/outside.md")));
        }
        finally
        {
            DeleteRoot(root);
            DeleteRoot(outside);
        }
    }

    [Fact]
    public async Task ReadsAndWritesUtf8FilesRelativeToTheVaultRoot()
    {
        var root = CreateRoot();
        try
        {
            var store = new LocalFileStore(root);
            var path = RelativePath.Parse("Notes/évidence.md");

            await store.WriteTextAtomicallyAsync(path, "# Native\n\nÉléphant");

            Assert.Equal("# Native\n\nÉléphant", await store.ReadTextAsync(path));
            Assert.True(await store.ExistsAsync(path));
            Assert.Equal(Path.GetFullPath(root), store.RootPath);
            Assert.Contains(await store.EnumerateAsync(recursive: true), entry => entry.Path == path);
        }
        finally
        {
            DeleteRoot(root);
        }
    }

    [Fact]
    public async Task AtomicWritesNeverExposePartialContentOrTemporaryFiles()
    {
        var root = CreateRoot();
        try
        {
            var store = new LocalFileStore(root);
            var path = RelativePath.Parse("Notes/atomic.md");
            var first = new string('A', 256 * 1024);
            var second = new string('B', 256 * 1024);
            await store.WriteTextAtomicallyAsync(path, first);

            var observed = new ConcurrentBag<string>();
            var writer = Task.Run(async () =>
            {
                for (var index = 0; index < 30; index++)
                {
                    await store.WriteTextAtomicallyAsync(path, index % 2 == 0 ? second : first);
                }
            });
            var readers = Enumerable.Range(0, 4).Select(_ => Task.Run(async () =>
            {
                for (var index = 0; index < 30; index++)
                {
                    observed.Add(await store.ReadTextAsync(path));
                }
            }));

            await Task.WhenAll(readers.Append(writer));

            Assert.NotEmpty(observed);
            Assert.All(observed, value => Assert.True(value == first || value == second));
            Assert.Empty(Directory.EnumerateFiles(root, "*.tmp", SearchOption.AllDirectories));
            Assert.Equal(first, await store.ReadTextAsync(path));
        }
        finally
        {
            DeleteRoot(root);
        }
    }

    [Fact]
    public async Task ReadErrorsAreWrittenToTheConfiguredApplicationLog()
    {
        var root = CreateRoot();
        var logDirectory = Path.Combine(root, "logs");
        try
        {
            AppLog.Configure(logDirectory);
            var store = new LocalFileStore(Path.Combine(root, "vault"));
            var missing = RelativePath.Parse("missing.md");

            await Assert.ThrowsAsync<FileNotFoundException>(() => store.ReadTextAsync(missing));

            var log = await File.ReadAllTextAsync(AppLog.Path, Encoding.UTF8);
            Assert.Contains("action=storage.read state=start", log);
            Assert.Contains("action=storage.read state=error", log);
            Assert.Contains("FileNotFoundException", log);
        }
        finally
        {
            DeleteRoot(root);
        }
    }

    [Fact]
    public async Task VaultAdapterUsesTheSameRootScopedStoreWithoutChangingVaultRepository()
    {
        var root = CreateRoot();
        try
        {
            var vault = new VaultDescriptor("test", "Test", root);
            var adapter = new VaultFileStoreAdapter(vault);

            await adapter.SaveNoteAsync("Notes/Native.md", "# Native");

            Assert.Equal("# Native", await adapter.ReadNoteAsync("Notes/Native.md"));
            await Assert.ThrowsAsync<InvalidDataException>(() => adapter.SaveNoteAsync("settings.json", "{}"));
        }
        finally
        {
            DeleteRoot(root);
        }
    }

    private static string CreateRoot()
    {
        var root = Path.Combine(Path.GetTempPath(), $"elephant-filestore-{Guid.NewGuid():N}");
        Directory.CreateDirectory(root);
        return root;
    }

    private static void DeleteRoot(string root)
    {
        if (Directory.Exists(root)) Directory.Delete(root, recursive: true);
    }
}

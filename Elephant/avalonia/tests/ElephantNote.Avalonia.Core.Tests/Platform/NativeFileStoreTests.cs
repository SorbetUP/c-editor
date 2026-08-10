using ElephantNote.Avalonia.Services.Platform;

namespace ElephantNote.Avalonia.Core.Tests.Platform;

public sealed class NativeFileStoreTests
{
    [Fact]
    public async Task WritesReadsAndReportsPresenceForRealTextFiles()
    {
        using var fixture = new PlatformTestDirectory();
        var path = System.IO.Path.Combine(fixture.Root, "nested", "note.md");
        IFileStore store = new NativeFileStore();

        Assert.False(await store.ExistsAsync(path));
        await store.WriteTextAsync(path, "# Native\n\nHello");

        Assert.True(await store.ExistsAsync(path));
        Assert.Equal("# Native\n\nHello", await store.ReadTextAsync(path));
    }

    [Fact]
    public async Task ReportsFalseForAPathThatDoesNotExist()
    {
        using var fixture = new PlatformTestDirectory();
        IFileStore store = new NativeFileStore();

        Assert.False(await store.ExistsAsync(System.IO.Path.Combine(fixture.Root, "missing.md")));
    }
}

using ElephantNote.Avalonia.Services;
using ElephantNote.Avalonia.Domain;

namespace ElephantNote.Avalonia.Core.Tests;

public sealed class VaultRepositoryTests
{
    [Fact]
    public async Task VaultRoundTripListsReadsWritesAndSearchesMarkdown()
    {
        using var fixture = new TestFixture();
        var vault = await fixture.Repository.SelectVaultAsync(fixture.VaultPath);
        Directory.CreateDirectory(Path.Combine(fixture.VaultPath, "Projects"));
        await File.WriteAllTextAsync(Path.Combine(fixture.VaultPath, "Projects", "Roadmap.md"), "# Roadmap\n\nRelease native UI");

        var entries = await fixture.Repository.ListEntriesAsync(vault, "Projects");
        Assert.Single(entries);
        Assert.Equal("Roadmap", entries[0].Title);
        Assert.Equal("# Roadmap", (await fixture.Repository.ReadNoteAsync(vault, "Projects/Roadmap.md")).Split('\n')[0]);

        await fixture.Repository.SaveNoteAsync(vault, "Projects/Roadmap.md", "# Roadmap\n\nAvalonia migration");
        var results = await fixture.Repository.SearchAsync(vault, "Avalonia");
        Assert.Single(results);
        Assert.Equal("Projects/Roadmap.md", results[0].Path);
    }

    [Fact]
    public async Task VaultPathsCannotEscapeTheSelectedRoot()
    {
        using var fixture = new TestFixture();
        var vault = await fixture.Repository.SelectVaultAsync(fixture.VaultPath);
        await Assert.ThrowsAsync<UnauthorizedAccessException>(() => fixture.Repository.ReadNoteAsync(vault, "../outside.md"));
    }

    [Fact]
    public void PreferencesRoundTripPreservesUnknownSharedKeys()
    {
        using var fixture = new TestFixture();
        var path = Path.Combine(fixture.Repository.ConfigDirectory, "preferences.json");
        File.WriteAllText(path, "{\"autoSave\":false,\"unknownSharedKey\":{\"keep\":true}}");
        var preferences = new PreferencesRepository(fixture.Repository.ConfigDirectory).Load();
        preferences.AutoSave = true;
        new PreferencesRepository(fixture.Repository.ConfigDirectory).Save(preferences);

        var json = File.ReadAllText(path);
        Assert.Contains("unknownSharedKey", json);
        Assert.Contains("\"autoSave\": true", json);
    }

    private sealed class TestFixture : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), $"elephant-avalonia-test-{Guid.NewGuid():N}");
        private readonly string? _previousConfigDirectory;

        public TestFixture()
        {
            Directory.CreateDirectory(VaultPath);
            _previousConfigDirectory = Environment.GetEnvironmentVariable("ELEPHANTNOTE_CONFIG_DIR");
            Environment.SetEnvironmentVariable("ELEPHANTNOTE_CONFIG_DIR", Path.Combine(_root, "config"));
            Repository = new VaultRepository();
        }

        public string VaultPath => Path.Combine(_root, "vault");
        public VaultRepository Repository { get; }

        public void Dispose()
        {
            Environment.SetEnvironmentVariable("ELEPHANTNOTE_CONFIG_DIR", _previousConfigDirectory);
            Directory.Delete(_root, recursive: true);
        }
    }
}

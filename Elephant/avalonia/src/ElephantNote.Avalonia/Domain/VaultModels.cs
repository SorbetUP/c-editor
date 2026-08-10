namespace ElephantNote.Avalonia.Domain;

public sealed record VaultDescriptor(
    string Id,
    string Name,
    string Path,
    string Icon = "",
    string LastOpenedAt = "0");

public sealed record VaultEntry(
    string Path,
    string Title,
    string Preview,
    bool IsDirectory,
    DateTime UpdatedAt)
{
    public string Kind => IsDirectory ? "folder" : "note";
    public string Glyph => IsDirectory ? "▰" : "▤";
}

public sealed record SearchResult(
    string Path,
    string Title,
    string Excerpt,
    int Score);

public sealed class VaultConfig
{
    public List<VaultDescriptor> Vaults { get; set; } = [];
    public string? ActiveVaultId { get; set; }
}

public sealed class AvaloniaPreferences
{
    public string Theme { get; set; } = "light";
    public bool AutoSave { get; set; }
    public int AutoSaveDelayMilliseconds { get; set; } = 5000;
}

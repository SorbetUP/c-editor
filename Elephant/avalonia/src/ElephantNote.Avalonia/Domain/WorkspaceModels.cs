using System.Text.Json;
using System.Text.Json.Serialization;

namespace ElephantNote.Avalonia.Domain;

public sealed class WorkspaceDocument
{
    [JsonPropertyName("version")]
    public int? Version { get; set; }

    [JsonPropertyName("schemaVersion")]
    public int? SchemaVersion { get; set; }

    [JsonPropertyName("vaultName")]
    public string? VaultName { get; set; }

    [JsonPropertyName("sidebar")]
    public List<SidebarEntry> Sidebar { get; set; } = [];

    [JsonPropertyName("sync")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public WorkspaceSyncState? Sync { get; set; }

    [JsonPropertyName("features")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public Dictionary<string, JsonElement>? Features { get; set; }

    [JsonExtensionData]
    public Dictionary<string, JsonElement> Extra { get; set; } = [];

    public static WorkspaceDocument Create(string vaultName) => new()
    {
        Version = 1,
        VaultName = string.IsNullOrWhiteSpace(vaultName) ? "Personal" : vaultName,
        Sidebar =
        [
            new SidebarEntry
            {
                Id = "getting-started",
                Title = "Getting started",
                Type = "folder",
                Path = RelativePath.Parse("Getting Started"),
                Collapsed = false,
                Items = [new SidebarEntry { Id = "welcome", Title = "Welcome", Type = "note", Path = RelativePath.Parse("Getting Started") }]
            }
        ]
    };

    internal WorkspaceDocument Copy() => new()
    {
        Version = Version,
        SchemaVersion = SchemaVersion,
        VaultName = VaultName,
        Sidebar = Sidebar.Select(item => item.Copy()).ToList(),
        Sync = Sync?.Copy(),
        Features = Features?.ToDictionary(item => item.Key, item => item.Value.Clone()),
        Extra = Extra.ToDictionary(item => item.Key, item => item.Value.Clone(), StringComparer.OrdinalIgnoreCase)
    };
}

public sealed class SidebarEntry
{
    [JsonPropertyName("id")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public string? Id { get; set; }

    [JsonPropertyName("title")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public string? Title { get; set; }

    [JsonPropertyName("type")]
    public string Type { get; set; } = "note";

    [JsonPropertyName("path")]
    public RelativePath Path { get; set; }

    [JsonPropertyName("collapsed")]
    public bool Collapsed { get; set; }

    [JsonPropertyName("items")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<SidebarEntry>? Items { get; set; }

    [JsonExtensionData]
    public Dictionary<string, JsonElement> Extra { get; set; } = [];

    internal SidebarEntry Copy() => new()
    {
        Id = Id,
        Title = Title,
        Type = Type,
        Path = Path,
        Collapsed = Collapsed,
        Items = Items?.Select(item => item.Copy()).ToList(),
        Extra = Extra.ToDictionary(item => item.Key, item => item.Value.Clone(), StringComparer.OrdinalIgnoreCase)
    };
}

public sealed class WorkspaceSyncState
{
    [JsonPropertyName("enabled")]
    public bool Enabled { get; set; }

    [JsonPropertyName("provider")]
    public string Provider { get; set; } = "git";

    [JsonPropertyName("lastRunAt")]
    public string LastRunAt { get; set; } = "";

    [JsonExtensionData]
    public Dictionary<string, JsonElement> Extra { get; set; } = [];

    internal WorkspaceSyncState Copy() => new()
    {
        Enabled = Enabled,
        Provider = Provider,
        LastRunAt = LastRunAt,
        Extra = Extra.ToDictionary(item => item.Key, item => item.Value.Clone(), StringComparer.OrdinalIgnoreCase)
    };
}

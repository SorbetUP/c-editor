namespace ElephantNote.Avalonia.Domain;

public sealed class MarkdownFrontmatter
{
    private readonly string _prefix;
    private readonly IReadOnlyDictionary<string, string> _values;

    internal MarkdownFrontmatter(string raw, string prefix)
    {
        Raw = raw;
        _prefix = prefix;
        _values = ParseValues(raw);
    }

    /// <summary>The exact YAML-like content between the two delimiters.</summary>
    public string Raw { get; }
    public string Content => Raw;
    public IReadOnlyDictionary<string, string> Values => _values;
    public string? Title => Get("title");
    public string? Type => Get("type");
    public string? CreatedAt => Get("createdAt");
    public string? UpdatedAt => Get("updatedAt");
    public IReadOnlyList<string> Tags => ParseTags();

    public string? Get(string key) => _values.TryGetValue(key, out var value) ? value : null;

    internal string Prefix => _prefix;

    private static IReadOnlyDictionary<string, string> ParseValues(string raw)
    {
        var values = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);
        foreach (var line in raw.Split('\n'))
        {
            var trimmed = line.TrimEnd('\r').Trim();
            if (trimmed.Length == 0 || trimmed.StartsWith('#')) continue;
            var separator = trimmed.IndexOf(':');
            if (separator <= 0) continue;
            var key = trimmed[..separator].Trim();
            var value = trimmed[(separator + 1)..].Trim();
            values[key] = Unquote(value);
        }
        return values;
    }

    private IReadOnlyList<string> ParseTags()
    {
        var raw = Get("tags");
        if (!string.IsNullOrWhiteSpace(raw)) return ParseTagList(raw);

        var lines = Raw.Split('\n');
        var tagsLine = Array.FindIndex(lines, line => line.TrimEnd('\r').Trim().Equals("tags:", StringComparison.OrdinalIgnoreCase));
        if (tagsLine < 0) return [];
        var tags = new List<string>();
        for (var index = tagsLine + 1; index < lines.Length; index++)
        {
            var line = lines[index].TrimEnd('\r');
            if (line.TrimStart().Length > 0 && !line.TrimStart().StartsWith('-')) break;
            var value = line.Trim().TrimStart('-').Trim();
            if (value.Length > 0) tags.Add(Unquote(value).TrimStart('#'));
        }
        return tags;
    }

    private static IReadOnlyList<string> ParseTagList(string raw)
    {
        if (!raw.StartsWith('[') || !raw.EndsWith(']')) return [Unquote(raw)];
        var values = new List<string>();
        var current = new System.Text.StringBuilder();
        var quote = '\0';
        foreach (var character in raw[1..^1])
        {
            if ((character == '"' || character == '\'') && quote == '\0') quote = character;
            else if (character == quote) quote = '\0';
            if (character == ',' && quote == '\0')
            {
                AddTag(values, current.ToString());
                current.Clear();
            }
            else current.Append(character);
        }
        AddTag(values, current.ToString());
        return values;
    }

    private static void AddTag(ICollection<string> values, string value)
    {
        var tag = Unquote(value.Trim());
        if (tag.Length > 0) values.Add(tag.TrimStart('#'));
    }

    private static string Unquote(string value)
    {
        if (value.Length >= 2 && ((value[0] == '"' && value[^1] == '"') || (value[0] == '\'' && value[^1] == '\'')))
        {
            return value[1..^1];
        }
        return value;
    }

}

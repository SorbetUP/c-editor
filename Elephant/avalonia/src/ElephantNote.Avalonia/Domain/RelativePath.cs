using System.Text.Json;
using System.Text.Json.Serialization;

namespace ElephantNote.Avalonia.Domain;

/// <summary>A normalized path relative to a vault root.</summary>
[JsonConverter(typeof(RelativePathJsonConverter))]
public readonly record struct RelativePath
{
    public RelativePath(string value) => Value = Normalize(value);

    public string Value { get; }
    public bool IsRoot => string.IsNullOrEmpty(Value);
    public IReadOnlyList<string> Segments => string.IsNullOrEmpty(Value) ? [] : Value.Split('/');
    public string FileName => Segments.LastOrDefault() ?? "";

    public static RelativePath Empty => new("");

    public static RelativePath Parse(string value) => new(value);

    public static bool TryParse(string? value, out RelativePath path)
    {
        try
        {
            path = new RelativePath(value ?? throw new ArgumentNullException(nameof(value)));
            return true;
        }
        catch (ArgumentException)
        {
            path = Empty;
            return false;
        }
    }

    public RelativePath Combine(RelativePath child) =>
        IsRoot ? child : child.IsRoot ? this : new RelativePath($"{Value}/{child.Value}");

    public RelativePath Combine(string child) => Combine(Parse(child));

    public RelativePath Parent
    {
        get
        {
            var value = Value ?? "";
            var separator = value.LastIndexOf('/');
            return separator < 0 ? Empty : new RelativePath(value[..separator]);
        }
    }

    public override string ToString() => Value ?? "";

    private static string Normalize(string value)
    {
        ArgumentNullException.ThrowIfNull(value);
        var normalized = value.Replace('\\', '/');
        if (normalized.StartsWith('/') || Path.IsPathRooted(value) || HasDrivePrefix(normalized))
        {
            throw new ArgumentException("A vault path must be relative.", nameof(value));
        }

        var segments = new List<string>();
        foreach (var segment in normalized.Split('/'))
        {
            if (segment is "") continue;
            if (segment == "..")
            {
                throw new ArgumentException("A vault path cannot contain '..'.", nameof(value));
            }
            if (segment == ".") continue;
            if (segment.Contains('\0'))
            {
                throw new ArgumentException("A vault path cannot contain a null character.", nameof(value));
            }
            segments.Add(segment);
        }
        return string.Join('/', segments);
    }

    private static bool HasDrivePrefix(string value) =>
        value.Length >= 2 && char.IsAsciiLetter(value[0]) && value[1] == ':';
}

public sealed class RelativePathJsonConverter : JsonConverter<RelativePath>
{
    public override RelativePath Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
    {
        if (reader.TokenType != JsonTokenType.String)
        {
            throw new JsonException("A relative path must be a JSON string.");
        }
        try
        {
            return RelativePath.Parse(reader.GetString() ?? "");
        }
        catch (ArgumentException error)
        {
            throw new JsonException(error.Message, error);
        }
    }

    public override void Write(Utf8JsonWriter writer, RelativePath value, JsonSerializerOptions options) =>
        writer.WriteStringValue(value.Value ?? "");
}

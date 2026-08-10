using System.Text.Json;
using System.Text.Json.Serialization;

namespace ElephantNote.Avalonia.Muya;

internal sealed record MuyaProtocolMessage
{
    public string Type { get; init; } = "";
    public string? DocumentId { get; init; }
    public string? RequestId { get; init; }
    public string? Content { get; init; }
    public string? Code { get; init; }
    public string? Message { get; init; }
    [JsonPropertyName("engine")]
    public string? ValidationEngine { get; init; }

    // The browser bundle currently calls this wire field rustRevision. Its
    // meaning at this boundary is only a validation revision; it is not a
    // second Rust-owned editor state and is never used for persistence.
    [JsonPropertyName("rustRevision")]
    public ulong? ValidationRevision { get; init; }
}

internal static class MuyaProtocol
{
    public const string Ready = "ready";
    public const string OpenDocument = "open-document";
    public const string SetContent = "set-content";
    public const string ContentChanged = "content-changed";
    public const string SaveRequested = "save-request";
    public const string Error = "error";

    private static readonly JsonSerializerOptions JsonOptions = new(JsonSerializerDefaults.Web)
    {
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull
    };

    public static string Serialize(string type, MuyaDocument document) => JsonSerializer.Serialize(new
    {
        type,
        documentId = document.DocumentId,
        relativePath = document.RelativePath,
        title = document.Title,
        content = document.Content
    }, JsonOptions);

    public static MuyaProtocolMessage Parse(string body)
    {
        var message = JsonSerializer.Deserialize<MuyaProtocolMessage>(body, JsonOptions)
            ?? throw new InvalidDataException("Muya sent an empty message.");
        if (string.IsNullOrWhiteSpace(message.Type)) throw new InvalidDataException("Muya message type is required.");
        return message;
    }
}

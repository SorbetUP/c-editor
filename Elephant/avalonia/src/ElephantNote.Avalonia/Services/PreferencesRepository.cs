using System.Text.Json;
using System.Text.Json.Nodes;
using ElephantNote.Avalonia.Domain;

namespace ElephantNote.Avalonia.Services;

public sealed class PreferencesRepository
{
    private static readonly JsonSerializerOptions JsonOptions = new() { WriteIndented = true };
    private readonly string _path;

    public PreferencesRepository(string directory)
    {
        Directory.CreateDirectory(directory);
        _path = System.IO.Path.Combine(directory, "preferences.json");
    }

    public AvaloniaPreferences Load()
    {
        var requestId = AppLog.Start("preferences.load");
        var timer = System.Diagnostics.Stopwatch.StartNew();
        if (!File.Exists(_path))
        {
            AppLog.Complete("preferences.load", requestId, timer, "defaults=true");
            return new AvaloniaPreferences();
        }
        try
        {
            var document = JsonNode.Parse(File.ReadAllText(_path))?.AsObject() ?? new JsonObject();
            var preferences = new AvaloniaPreferences
            {
                Theme = ReadString(document, "theme", "light"),
                AutoSave = ReadBool(document, "autoSave", false),
                AutoSaveDelayMilliseconds = ReadInt(document, "autoSaveDelay", 5000)
            };
            AppLog.Complete("preferences.load", requestId, timer);
            return preferences;
        }
        catch (Exception error)
        {
            AppLog.Fail("preferences.load", requestId, timer, error);
            return new AvaloniaPreferences();
        }
    }

    public void Save(AvaloniaPreferences preferences)
    {
        var requestId = AppLog.Start("preferences.save");
        var timer = System.Diagnostics.Stopwatch.StartNew();
        try
        {
            var document = File.Exists(_path)
                ? JsonNode.Parse(File.ReadAllText(_path))?.AsObject() ?? new JsonObject()
                : new JsonObject();
            document["theme"] = preferences.Theme;
            document["autoSave"] = preferences.AutoSave;
            document["autoSaveDelay"] = preferences.AutoSaveDelayMilliseconds;
            WriteAtomically(_path, document.ToJsonString(JsonOptions));
            AppLog.Complete("preferences.save", requestId, timer);
        }
        catch (Exception error)
        {
            AppLog.Fail("preferences.save", requestId, timer, error);
            throw;
        }
    }

    private static string ReadString(JsonObject document, string key, string fallback) =>
        document[key]?.GetValue<string>() ?? fallback;

    private static bool ReadBool(JsonObject document, string key, bool fallback) =>
        document[key]?.GetValue<bool>() ?? fallback;

    private static int ReadInt(JsonObject document, string key, int fallback) =>
        document[key]?.GetValue<int>() ?? fallback;

    private static void WriteAtomically(string path, string content)
    {
        var temporary = $"{path}.{Guid.NewGuid():N}.tmp";
        try
        {
            File.WriteAllText(temporary, content);
            File.Move(temporary, path, overwrite: true);
        }
        finally
        {
            if (File.Exists(temporary)) File.Delete(temporary);
        }
    }
}

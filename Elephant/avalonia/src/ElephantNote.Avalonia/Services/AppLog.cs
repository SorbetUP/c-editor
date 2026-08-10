using System.Diagnostics;
using System.Text;

namespace ElephantNote.Avalonia.Services;

public static class AppLog
{
    private static readonly object Gate = new();
    private static string _path = System.IO.Path.Combine(System.IO.Path.GetTempPath(), "elephant-avalonia.log");

    public static string Path => _path;

    public static void Configure(string directory)
    {
        Directory.CreateDirectory(directory);
        _path = System.IO.Path.Combine(directory, "avalonia.log");
        Write("runtime", "configured", $"logPath={_path}");
    }

    public static string Start(string action, string detail = "")
    {
        var id = Guid.NewGuid().ToString("N");
        Write(action, "start", $"requestId={id} {detail}".Trim());
        return id;
    }

    public static void Complete(string action, string requestId, Stopwatch timer, string detail = "")
    {
        Write(action, "done", $"requestId={requestId} durationMs={timer.ElapsedMilliseconds} {detail}".Trim());
    }

    public static void Fail(string action, string requestId, Stopwatch timer, Exception error)
    {
        Write(action, "error", $"requestId={requestId} durationMs={timer.ElapsedMilliseconds} type={error.GetType().Name} message={error.Message}");
    }

    private static void Write(string action, string state, string detail)
    {
        var line = $"{DateTimeOffset.UtcNow:O} action={action} state={state} {detail}{Environment.NewLine}";
        lock (Gate)
        {
            try
            {
                File.AppendAllText(_path, line, Encoding.UTF8);
            }
            catch
            {
                Debug.WriteLine(line);
            }
        }
    }
}

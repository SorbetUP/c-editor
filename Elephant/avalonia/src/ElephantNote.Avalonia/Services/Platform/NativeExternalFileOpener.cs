using System.ComponentModel;
using System.Diagnostics;

namespace ElephantNote.Avalonia.Services.Platform;

public sealed class NativeExternalFileOpener : IExternalFileOpener
{
    public Task OpenAsync(string path, CancellationToken cancellationToken = default)
    {
        cancellationToken.ThrowIfCancellationRequested();
        var fullPath = RequireExistingPath(path);
        var startInfo = new ProcessStartInfo
        {
            FileName = fullPath,
            UseShellExecute = true
        };

        try
        {
            if (Process.Start(startInfo) is null)
            {
                throw new PlatformCapabilityUnavailableException(
                    PlatformCapability.ExternalFileOpener,
                    $"The operating system did not open '{fullPath}'.");
            }
        }
        catch (PlatformCapabilityUnavailableException)
        {
            throw;
        }
        catch (InvalidOperationException error)
        {
            throw new PlatformCapabilityUnavailableException(
                PlatformCapability.ExternalFileOpener,
                $"The native file opener is unavailable for '{fullPath}'.",
                error);
        }
        catch (Win32Exception error)
        {
            throw new PlatformCapabilityUnavailableException(
                PlatformCapability.ExternalFileOpener,
                $"The native file opener could not open '{fullPath}'.",
                error);
        }

        return Task.CompletedTask;
    }

    private static string RequireExistingPath(string path)
    {
        if (string.IsNullOrWhiteSpace(path)) throw new ArgumentException("A file path is required.", nameof(path));
        if (path.Contains('\0')) throw new ArgumentException("A file path cannot contain a null character.", nameof(path));

        var fullPath = Path.GetFullPath(path);
        if (!File.Exists(fullPath) && !Directory.Exists(fullPath))
        {
            throw new FileNotFoundException($"The file or directory to open does not exist: {fullPath}", fullPath);
        }
        return fullPath;
    }
}

using Avalonia.Input.Platform;

namespace ElephantNote.Avalonia.Services.Platform;

public sealed class NativePlatformServices : IPlatformServices
{
    public NativePlatformServices(
        Func<IClipboard?>? clipboardProvider = null,
        IFileStore? fileStore = null,
        ISyncService? sync = null,
        IExternalFileOpener? externalFileOpener = null,
        IClipboardService? clipboard = null,
        IExecutionService? execution = null)
    {
        FileStore = fileStore ?? new NativeFileStore();
        Sync = sync ?? new UnavailableSyncService();
        ExternalFileOpener = externalFileOpener ?? new NativeExternalFileOpener();
        Clipboard = clipboard ?? new NativeClipboardService(clipboardProvider ?? (() => null));
        Execution = execution ?? new NativeExecutionService();
    }

    public IFileStore FileStore { get; }

    public ISyncService Sync { get; }

    public IExternalFileOpener ExternalFileOpener { get; }

    public IClipboardService Clipboard { get; }

    public IExecutionService Execution { get; }
}

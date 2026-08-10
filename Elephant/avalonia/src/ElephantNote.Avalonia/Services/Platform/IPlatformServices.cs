namespace ElephantNote.Avalonia.Services.Platform;

/// <summary>Groups platform capabilities so application services depend on contracts, not OS APIs.</summary>
public interface IPlatformServices
{
    IFileStore FileStore { get; }

    ISyncService Sync { get; }

    IExternalFileOpener ExternalFileOpener { get; }

    IClipboardService Clipboard { get; }

    IExecutionService Execution { get; }
}

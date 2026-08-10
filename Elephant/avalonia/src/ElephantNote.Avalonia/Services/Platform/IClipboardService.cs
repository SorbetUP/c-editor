namespace ElephantNote.Avalonia.Services.Platform;

/// <summary>Exposes text clipboard operations without exposing a UI toolkit to callers.</summary>
public interface IClipboardService
{
    Task<string?> GetTextAsync(CancellationToken cancellationToken = default);

    Task SetTextAsync(string text, CancellationToken cancellationToken = default);
}

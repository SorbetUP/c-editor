namespace ElephantNote.Avalonia.Core.Tests.Platform;

internal sealed class PlatformTestDirectory : IDisposable
{
    public PlatformTestDirectory()
    {
        Root = System.IO.Path.Combine(System.IO.Path.GetTempPath(), $"elephant-platform-test-{Guid.NewGuid():N}");
        Directory.CreateDirectory(Root);
    }

    public string Root { get; }

    public void Dispose()
    {
        if (Directory.Exists(Root)) Directory.Delete(Root, recursive: true);
    }
}

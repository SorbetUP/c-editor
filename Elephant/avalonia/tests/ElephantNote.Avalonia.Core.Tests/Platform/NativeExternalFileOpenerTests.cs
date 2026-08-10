using ElephantNote.Avalonia.Services.Platform;

namespace ElephantNote.Avalonia.Core.Tests.Platform;

public sealed class NativeExternalFileOpenerTests
{
    [Fact]
    public async Task MissingPathIsReportedBeforeLaunchingAnExternalProcess()
    {
        var path = System.IO.Path.Combine(System.IO.Path.GetTempPath(), $"missing-{Guid.NewGuid():N}.txt");

        var error = await Assert.ThrowsAsync<FileNotFoundException>(
            () => new NativeExternalFileOpener().OpenAsync(path));

        Assert.Equal(path, error.FileName);
    }
}

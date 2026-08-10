using ElephantNote.Avalonia.Services.Platform;

namespace ElephantNote.Avalonia.Core.Tests.Platform;

public sealed class NativeExecutionServiceTests
{
    [Fact]
    public async Task ExecutesARealPlatformProcessAndCapturesItsOutput()
    {
        var request = OperatingSystem.IsWindows()
            ? new ExecutionRequest("cmd.exe", ["/d", "/c", "echo Elephant"])
            : new ExecutionRequest("/bin/sh", ["-c", "printf Elephant"]);

        var result = await new NativeExecutionService().ExecuteAsync(request);

        Assert.Equal(0, result.ExitCode);
        Assert.Equal("Elephant", result.StandardOutput.Trim());
        Assert.Empty(result.StandardError);
    }

    [Fact]
    public async Task MissingExecutableIsReportedAsUnavailableExecution()
    {
        var request = new ExecutionRequest(
            System.IO.Path.Combine(System.IO.Path.GetTempPath(), $"missing-{Guid.NewGuid():N}"),
            []);

        var error = await Assert.ThrowsAsync<PlatformCapabilityUnavailableException>(
            () => new NativeExecutionService().ExecuteAsync(request));

        Assert.Equal(PlatformCapability.Execution, error.Capability);
    }
}

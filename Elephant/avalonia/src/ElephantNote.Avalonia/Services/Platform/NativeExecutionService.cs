using System.ComponentModel;
using System.Diagnostics;

namespace ElephantNote.Avalonia.Services.Platform;

public sealed class NativeExecutionService : IExecutionService
{
    public async Task<ExecutionResult> ExecuteAsync(ExecutionRequest request, CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(request);
        if (string.IsNullOrWhiteSpace(request.FileName)) throw new ArgumentException("An executable path is required.", nameof(request));
        ArgumentNullException.ThrowIfNull(request.Arguments);
        cancellationToken.ThrowIfCancellationRequested();

        var startInfo = new ProcessStartInfo
        {
            FileName = request.FileName,
            UseShellExecute = false,
            RedirectStandardOutput = true,
            RedirectStandardError = true
        };
        if (!string.IsNullOrWhiteSpace(request.WorkingDirectory)) startInfo.WorkingDirectory = request.WorkingDirectory;
        foreach (var argument in request.Arguments)
        {
            ArgumentNullException.ThrowIfNull(argument);
            startInfo.ArgumentList.Add(argument);
        }

        using var process = new Process { StartInfo = startInfo };
        try
        {
            if (!process.Start())
            {
                throw new PlatformCapabilityUnavailableException(
                    PlatformCapability.Execution,
                    $"The operating system did not start '{request.FileName}'.");
            }
        }
        catch (PlatformCapabilityUnavailableException)
        {
            throw;
        }
        catch (InvalidOperationException error)
        {
            throw new PlatformCapabilityUnavailableException(
                PlatformCapability.Execution,
                $"Native process execution is unavailable for '{request.FileName}'.",
                error);
        }
        catch (Win32Exception error)
        {
            throw new PlatformCapabilityUnavailableException(
                PlatformCapability.Execution,
                $"The executable '{request.FileName}' could not be started.",
                error);
        }

        try
        {
            var outputTask = process.StandardOutput.ReadToEndAsync(cancellationToken);
            var errorTask = process.StandardError.ReadToEndAsync(cancellationToken);
            await process.WaitForExitAsync(cancellationToken);
            return new ExecutionResult(process.ExitCode, await outputTask, await errorTask);
        }
        catch (OperationCanceledException)
        {
            if (!process.HasExited) process.Kill(entireProcessTree: true);
            throw;
        }
    }
}

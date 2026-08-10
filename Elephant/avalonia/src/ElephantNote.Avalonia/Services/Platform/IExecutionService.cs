namespace ElephantNote.Avalonia.Services.Platform;

public interface IExecutionService
{
    Task<ExecutionResult> ExecuteAsync(ExecutionRequest request, CancellationToken cancellationToken = default);
}

public sealed record ExecutionRequest(
    string FileName,
    IReadOnlyList<string> Arguments,
    string? WorkingDirectory = null);

public sealed record ExecutionResult(
    int ExitCode,
    string StandardOutput,
    string StandardError);

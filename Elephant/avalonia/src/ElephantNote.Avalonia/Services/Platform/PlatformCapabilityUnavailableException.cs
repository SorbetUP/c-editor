namespace ElephantNote.Avalonia.Services.Platform;

public sealed class PlatformCapabilityUnavailableException : NotSupportedException
{
    public PlatformCapabilityUnavailableException(PlatformCapability capability, string message)
        : base(message) => Capability = capability;

    public PlatformCapabilityUnavailableException(PlatformCapability capability, string message, Exception innerException)
        : base(message, innerException) => Capability = capability;

    public PlatformCapability Capability { get; }
}

global using Xunit;

// AppLog is intentionally process-wide because it is the native app log sink.
// Serialize this small test assembly so filesystem tests cannot reconfigure
// the sink while another repository test is tearing down its temporary vault.
[assembly: CollectionBehavior(DisableTestParallelization = true)]

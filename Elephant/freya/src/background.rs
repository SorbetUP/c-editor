//! Small bridge for blocking native work initiated by the Freya UI.
//!
//! Filesystem scans, SQLite rebuilds and child-process protocols are blocking
//! Rust APIs today. Running them directly inside Freya callbacks stalls input
//! and rendering. This helper executes the blocking closure on an OS worker
//! thread, then waits cooperatively in a local Freya future before applying the
//! result back on the UI state boundary.

use freya::prelude::spawn;
use futures_timer::Delay;
use std::{
    panic::{catch_unwind, AssertUnwindSafe},
    sync::mpsc::{self, TryRecvError},
    thread,
    time::Duration,
};

const POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Debug)]
pub(crate) enum BackgroundError {
    WorkerPanicked,
    WorkerDisconnected,
    SpawnFailed(String),
}

impl std::fmt::Display for BackgroundError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WorkerPanicked => formatter.write_str("background worker panicked"),
            Self::WorkerDisconnected => formatter.write_str("background worker disconnected"),
            Self::SpawnFailed(error) => write!(formatter, "unable to start background worker: {error}"),
        }
    }
}

/// Execute a blocking operation without blocking Freya's event loop.
///
/// `apply` always runs from Freya's local task after the worker result crosses
/// the channel. The worker result itself is wrapped so a panic cannot silently
/// strand the UI in a loading state.
pub(crate) fn run<T, Work, Apply>(label: &'static str, work: Work, apply: Apply)
where
    T: Send + 'static,
    Work: FnOnce() -> T + Send + 'static,
    Apply: FnOnce(Result<T, BackgroundError>) + 'static,
{
    let (sender, receiver) = mpsc::sync_channel(1);
    let spawn_result = thread::Builder::new()
        .name(format!("elephant-{label}"))
        .spawn(move || {
            let outcome = catch_unwind(AssertUnwindSafe(work))
                .map_err(|_| BackgroundError::WorkerPanicked);
            let _ = sender.send(outcome);
        });

    if let Err(error) = spawn_result {
        apply(Err(BackgroundError::SpawnFailed(error.to_string())));
        return;
    }

    spawn(async move {
        let mut apply = Some(apply);
        loop {
            match receiver.try_recv() {
                Ok(result) => {
                    if let Some(apply) = apply.take() {
                        apply(result);
                    }
                    return;
                }
                Err(TryRecvError::Empty) => Delay::new(POLL_INTERVAL).await,
                Err(TryRecvError::Disconnected) => {
                    if let Some(apply) = apply.take() {
                        apply(Err(BackgroundError::WorkerDisconnected));
                    }
                    return;
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_keeps_background_failures_visible() {
        assert!(BackgroundError::WorkerPanicked.to_string().contains("panicked"));
        assert!(BackgroundError::WorkerDisconnected
            .to_string()
            .contains("disconnected"));
    }
}

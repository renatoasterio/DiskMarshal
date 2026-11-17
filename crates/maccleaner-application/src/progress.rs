//! Progress reporting and cancellation for long-running use cases.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use maccleaner_domain::shared::ByteSize;

/// A flag shared between whoever started an operation and the operation itself.
///
/// Cloning it yields another handle to the same flag.
#[derive(Debug, Clone, Default)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    /// Creates a token that has not been cancelled.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Asks the operation to stop as soon as it can.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    /// Returns `true` once [`cancel`](Self::cancel) has been called.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

/// How far a scan has gone.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ScanProgress {
    /// Items seen so far.
    pub items_seen: u64,
    /// Disk space used by the items seen so far.
    pub bytes_seen: ByteSize,
}

/// Receives progress updates. Updates may arrive from several threads at once.
pub trait ProgressSink: Send + Sync {
    /// Called whenever the operation has new totals to show.
    fn on_progress(&self, progress: ScanProgress);
}

/// A sink for callers that do not show progress.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoProgress;

impl ProgressSink for NoProgress {
    fn on_progress(&self, _progress: ScanProgress) {}
}

#[cfg(test)]
mod tests {
    use super::CancellationToken;

    #[test]
    fn clones_share_the_same_flag() {
        let token = CancellationToken::new();
        let handle = token.clone();
        assert!(!handle.is_cancelled());
        token.cancel();
        assert!(handle.is_cancelled());
    }
}

//! One piece of background work and the handle the interface keeps to it.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

use egui::Context;
use maccleaner_application::progress::{CancellationToken, ProgressSink, ScanProgress};
use maccleaner_domain::shared::ByteSize;

/// What a task has to show when the interface looks at it.
pub(crate) enum TaskPoll<T> {
    /// Still working.
    Pending,
    /// Finished with this value. Reported only once.
    Ready(T),
    /// The worker stopped without a result, which only a crash can cause.
    Lost,
}

/// The latest totals reported by the worker.
#[derive(Default)]
struct SharedProgress {
    items: AtomicU64,
    bytes: AtomicU64,
}

impl ProgressSink for SharedProgress {
    fn on_progress(&self, progress: ScanProgress) {
        self.items.store(progress.items_seen, Ordering::Relaxed);
        self.bytes
            .store(progress.bytes_seen.as_bytes(), Ordering::Relaxed);
    }
}

/// Work running on its own thread. Dropping the handle cancels the work.
pub(crate) struct Task<T> {
    result: Receiver<T>,
    progress: Arc<SharedProgress>,
    cancellation: CancellationToken,
}

impl<T: Send + 'static> Task<T> {
    /// Starts `work` on a new thread. The interface is repainted when it ends.
    pub(crate) fn spawn(
        ctx: &Context,
        work: impl FnOnce(&dyn ProgressSink, &CancellationToken) -> T + Send + 'static,
    ) -> Self {
        let (sender, result) = mpsc::channel();
        let progress = Arc::new(SharedProgress::default());
        let cancellation = CancellationToken::new();
        let (worker_progress, worker_cancellation, repaint) =
            (Arc::clone(&progress), cancellation.clone(), ctx.clone());
        thread::spawn(move || {
            let value = work(worker_progress.as_ref(), &worker_cancellation);
            // The receiver is gone when the task was dropped; nobody wants the value then.
            let _ = sender.send(value);
            repaint.request_repaint();
        });
        Self {
            result,
            progress,
            cancellation,
        }
    }
}

impl<T> Task<T> {
    /// Looks for the result without waiting.
    pub(crate) fn poll(&self) -> TaskPoll<T> {
        match self.result.try_recv() {
            Ok(value) => TaskPoll::Ready(value),
            Err(TryRecvError::Empty) => TaskPoll::Pending,
            Err(TryRecvError::Disconnected) => TaskPoll::Lost,
        }
    }

    /// The latest totals reported by the worker.
    pub(crate) fn progress(&self) -> ScanProgress {
        ScanProgress {
            items_seen: self.progress.items.load(Ordering::Relaxed),
            bytes_seen: ByteSize::from_bytes(self.progress.bytes.load(Ordering::Relaxed)),
        }
    }
}

impl<T> Drop for Task<T> {
    fn drop(&mut self) {
        self.cancellation.cancel();
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::{Duration, Instant};

    use maccleaner_application::progress::ScanProgress;
    use maccleaner_domain::shared::ByteSize;

    use super::{Task, TaskPoll};

    fn wait_for<T>(task: &Task<T>) -> TaskPoll<T> {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match task.poll() {
                TaskPoll::Pending if Instant::now() < deadline => {
                    thread::sleep(Duration::from_millis(2));
                }
                other => return other,
            }
        }
    }

    #[test]
    fn delivers_the_result_once_and_the_progress_along_the_way() {
        let task = Task::spawn(&egui::Context::default(), |progress, _| {
            progress.on_progress(ScanProgress {
                items_seen: 7,
                bytes_seen: ByteSize::from_bytes(70),
            });
            42
        });

        assert!(matches!(wait_for(&task), TaskPoll::Ready(42)));
        assert_eq!(task.progress().items_seen, 7);
        assert!(matches!(task.poll(), TaskPoll::Lost));
    }

    #[test]
    fn dropping_the_handle_cancels_the_work() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let task = Task::spawn(&egui::Context::default(), move |_, cancellation| {
            while !cancellation.is_cancelled() {
                thread::sleep(Duration::from_millis(1));
            }
            let _ = sender.send(());
        });

        drop(task);

        assert!(receiver.recv_timeout(Duration::from_secs(5)).is_ok());
    }
}

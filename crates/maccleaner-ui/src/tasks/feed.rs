//! A stream of values produced in the background at a steady pace.

use std::sync::mpsc::{self, Receiver, TrySendError};
use std::thread;
use std::time::Duration;

use egui::Context;
use maccleaner_application::progress::CancellationToken;

/// Values kept while nobody is reading. A window that stays hidden for hours
/// must not pile up readings without limit; the newest ones are dropped.
const BACKLOG: usize = 240;

/// Values produced on their own thread, one per interval. Dropping the feed
/// stops the thread.
pub(crate) struct Feed<T> {
    values: Receiver<T>,
    cancellation: CancellationToken,
}

impl<T: Send + 'static> Feed<T> {
    /// Starts calling `produce` every `interval`. The interface is repainted
    /// each time a value arrives.
    pub(crate) fn spawn(
        ctx: &Context,
        interval: Duration,
        mut produce: impl FnMut() -> T + Send + 'static,
    ) -> Self {
        let (sender, values) = mpsc::sync_channel(BACKLOG);
        let cancellation = CancellationToken::new();
        let (worker_cancellation, repaint) = (cancellation.clone(), ctx.clone());
        thread::spawn(move || {
            while !worker_cancellation.is_cancelled() {
                match sender.try_send(produce()) {
                    Ok(()) => repaint.request_repaint(),
                    Err(TrySendError::Full(_)) => {}
                    Err(TrySendError::Disconnected(_)) => break,
                }
                thread::sleep(interval);
            }
        });
        Self {
            values,
            cancellation,
        }
    }

    /// Takes every value that arrived since the last call, oldest first.
    pub(crate) fn drain(&self) -> Vec<T> {
        self.values.try_iter().collect()
    }
}

impl<T> Drop for Feed<T> {
    fn drop(&mut self) {
        self.cancellation.cancel();
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::{Duration, Instant};

    use super::Feed;

    #[test]
    fn delivers_values_in_order_until_it_is_dropped() {
        let mut next = 0;
        let feed = Feed::spawn(
            &egui::Context::default(),
            Duration::from_millis(2),
            move || {
                next += 1;
                next
            },
        );

        let mut received = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(5);
        while received.len() < 3 && Instant::now() < deadline {
            received.extend(feed.drain());
            thread::sleep(Duration::from_millis(2));
        }

        assert_eq!(received.get(..3), Some(&[1, 2, 3][..]));
    }
}

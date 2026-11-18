//! Spreading slow, independent work over the processor cores.

use std::num::NonZero;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use crate::ApplicationError;
use crate::progress::CancellationToken;

/// Applies `work` to every item, on as many threads as there are cores, and
/// returns the results in the order of the items.
///
/// # Errors
///
/// Fails with [`ApplicationError::Cancelled`] when the work is cancelled
/// before every item is done.
pub(crate) fn map_in_parallel<T: Sync, R: Send>(
    items: &[T],
    cancellation: &CancellationToken,
    work: impl Fn(&T) -> R + Sync,
) -> Result<Vec<R>, ApplicationError> {
    let cores = thread::available_parallelism().map_or(1, NonZero::get);
    let next = AtomicUsize::new(0);
    let worker = || {
        let mut done = Vec::new();
        while !cancellation.is_cancelled() {
            let index = next.fetch_add(1, Ordering::Relaxed);
            let Some(item) = items.get(index) else { break };
            done.push((index, work(item)));
        }
        done
    };
    let mut results: Vec<(usize, R)> = thread::scope(|scope| {
        let workers: Vec<_> = (0..cores.min(items.len()))
            .map(|_| scope.spawn(worker))
            .collect();
        // A worker that panicked contributes nothing; the count below notices.
        workers
            .into_iter()
            .flat_map(|worker| worker.join().unwrap_or_default())
            .collect()
    });
    if results.len() != items.len() {
        return Err(ApplicationError::Cancelled);
    }
    results.sort_by_key(|(index, _)| *index);
    Ok(results.into_iter().map(|(_, result)| result).collect())
}

#[cfg(test)]
mod tests {
    use super::map_in_parallel;
    use crate::ApplicationError;
    use crate::progress::CancellationToken;

    #[test]
    fn keeps_the_results_in_the_order_of_the_items() {
        let items: Vec<u32> = (0..500).collect();
        let doubled = map_in_parallel(&items, &CancellationToken::new(), |item| item * 2).unwrap();
        assert_eq!(
            doubled,
            items.iter().map(|item| item * 2).collect::<Vec<_>>()
        );
    }

    #[test]
    fn nothing_to_do_is_not_an_error() {
        let nothing: [u32; 0] = [];
        assert!(
            map_in_parallel(&nothing, &CancellationToken::new(), |item| *item)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn stops_when_cancelled() {
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let result = map_in_parallel(&[1, 2, 3], &cancellation, |item| *item);
        assert!(matches!(result, Err(ApplicationError::Cancelled)));
    }
}

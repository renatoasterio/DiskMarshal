//! A background search and what the interface keeps about it.

use egui::Context;
use maccleaner_application::ApplicationError;
use maccleaner_application::progress::{CancellationToken, ProgressSink, ScanProgress};

use crate::i18n::pt_br;
use crate::tasks::{Task, TaskPoll};

/// A search that runs off the interface thread, with the failure of the
/// last attempt kept as text for the screen.
pub(crate) struct Search<T> {
    task: Option<Task<Result<T, ApplicationError>>>,
    failure: Option<String>,
}

impl<T> Default for Search<T> {
    fn default() -> Self {
        Self {
            task: None,
            failure: None,
        }
    }
}

impl<T: Send + 'static> Search<T> {
    /// Starts `work` in the background, replacing any search under way.
    pub(crate) fn start(
        &mut self,
        ctx: &Context,
        work: impl FnOnce(&dyn ProgressSink, &CancellationToken) -> Result<T, ApplicationError>
        + Send
        + 'static,
    ) {
        self.failure = None;
        self.task = Some(Task::spawn(ctx, work));
    }
}

impl<T> Search<T> {
    /// Stops the search under way. What it had found is discarded.
    pub(crate) fn cancel(&mut self) {
        self.task = None;
    }

    /// Takes the result of a search that has just ended. A failure is kept
    /// as text instead, and a cancelled search leaves nothing behind.
    pub(crate) fn poll(&mut self) -> Option<T> {
        let ended = match self.task.as_ref()?.poll() {
            TaskPoll::Pending => return None,
            TaskPoll::Ready(outcome) => Some(outcome),
            TaskPoll::Lost => None,
        };
        self.task = None;
        if let Some(outcome) = ended {
            return self.settle(outcome);
        }
        self.failure = Some(pt_br::SCAN_LOST.to_owned());
        None
    }

    /// Records the outcome of a search that ran somewhere else and returns
    /// its result, when it has one.
    pub(crate) fn settle(&mut self, outcome: Result<T, ApplicationError>) -> Option<T> {
        match outcome {
            Ok(value) => {
                self.failure = None;
                Some(value)
            }
            Err(ApplicationError::Cancelled) => None,
            Err(error) => {
                self.failure = Some(pt_br::application_error(&error));
                None
            }
        }
    }

    /// Totals of the search under way, when there is one.
    pub(crate) fn progress(&self) -> Option<ScanProgress> {
        self.task.as_ref().map(Task::progress)
    }

    /// Why the last search failed, when it did.
    pub(crate) fn failure(&self) -> Option<&str> {
        self.failure.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use maccleaner_application::ApplicationError;

    use super::Search;

    #[test]
    fn a_result_clears_the_failure_of_the_attempt_before() {
        let mut search = Search::<u32>::default();
        assert_eq!(
            search.settle(Err(ApplicationError::RootNotFound(PathBuf::from("/gone")))),
            None
        );
        assert!(search.failure().is_some());

        assert_eq!(search.settle(Ok(7)), Some(7));
        assert!(search.failure().is_none());
    }

    #[test]
    fn a_cancelled_search_is_not_a_failure() {
        let mut search = Search::<u32>::default();
        assert_eq!(search.settle(Err(ApplicationError::Cancelled)), None);
        assert!(search.failure().is_none());
        assert!(search.progress().is_none());
    }
}

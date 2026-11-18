//! Use case: clean the areas only an administrator may change.

use crate::ports::{HistoryRepository, PrivilegeError, PrivilegedCleaner};
use crate::use_cases::DeletionReport;

/// Cleans system areas through the privileged cleaner and records what was
/// removed in the history of the user.
pub struct CleanSystemAreas<'a> {
    cleaner: &'a dyn PrivilegedCleaner,
    history: &'a dyn HistoryRepository,
}

impl<'a> CleanSystemAreas<'a> {
    /// Creates the use case over the privileged cleaner and the history.
    #[must_use]
    pub fn new(cleaner: &'a dyn PrivilegedCleaner, history: &'a dyn HistoryRepository) -> Self {
        Self { cleaner, history }
    }

    /// Cleans what the administrator rules named in `rule_ids` select.
    /// macOS authenticates the user first, showing `prompt` in its dialog.
    ///
    /// # Errors
    ///
    /// Fails when the user does not authenticate or the cleanup cannot run.
    pub fn execute(
        &self,
        rule_ids: &[String],
        prompt: &str,
    ) -> Result<DeletionReport, PrivilegeError> {
        let mut report = DeletionReport::default();
        for outcome in self.cleaner.clean(rule_ids, prompt)? {
            if outcome.is_removed() && self.history.append(&outcome).is_err() {
                report.unrecorded += 1;
            }
            report.record(outcome);
        }
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Mutex;

    use maccleaner_domain::deletion::{DeletionOutcome, DeletionStatus, HistoryEntry, SkipReason};
    use maccleaner_domain::shared::ByteSize;

    use super::CleanSystemAreas;
    use crate::ApplicationError;
    use crate::ports::{HistoryRepository, PrivilegeError, PrivilegedCleaner};

    /// A cleaner that answers with a fixed result.
    struct FixedCleaner(Result<Vec<DeletionOutcome>, PrivilegeError>);

    impl PrivilegedCleaner for FixedCleaner {
        fn is_available(&self) -> bool {
            true
        }

        fn clean(
            &self,
            _rule_ids: &[String],
            _prompt: &str,
        ) -> Result<Vec<DeletionOutcome>, PrivilegeError> {
            self.0.clone()
        }
    }

    /// A history that remembers what was appended, or refuses everything.
    #[derive(Default)]
    struct RecordingHistory {
        appended: Mutex<Vec<PathBuf>>,
        is_broken: bool,
    }

    impl HistoryRepository for RecordingHistory {
        fn append(&self, outcome: &DeletionOutcome) -> Result<(), ApplicationError> {
            if self.is_broken {
                return Err(ApplicationError::Cancelled);
            }
            if let Ok(mut appended) = self.appended.lock() {
                appended.push(outcome.target.clone());
            }
            Ok(())
        }

        fn recent(&self, _limit: usize) -> Vec<HistoryEntry> {
            Vec::new()
        }
    }

    fn outcome(name: &str, status: DeletionStatus) -> DeletionOutcome {
        DeletionOutcome {
            target: PathBuf::from("/Library/Caches").join(name),
            status,
            reclaimable: ByteSize::from_bytes(1_000),
        }
    }

    fn rules() -> Vec<String> {
        vec!["system-caches".to_owned()]
    }

    #[test]
    fn records_only_what_was_really_removed() {
        let cleaner = FixedCleaner(Ok(vec![
            outcome("gone", DeletionStatus::Deleted),
            outcome(
                "kept",
                DeletionStatus::Skipped(SkipReason::Failed("busy".to_owned())),
            ),
        ]));
        let history = RecordingHistory::default();

        let report = CleanSystemAreas::new(&cleaner, &history)
            .execute(&rules(), "prompt")
            .unwrap();

        assert_eq!(report.outcomes.len(), 2);
        assert_eq!(report.reclaimed, ByteSize::from_bytes(1_000));
        assert_eq!(report.unrecorded, 0);
        assert_eq!(
            *history.appended.lock().unwrap(),
            [PathBuf::from("/Library/Caches/gone")]
        );
    }

    #[test]
    fn counts_removals_the_history_could_not_keep() {
        let cleaner = FixedCleaner(Ok(vec![outcome("gone", DeletionStatus::Deleted)]));
        let history = RecordingHistory {
            is_broken: true,
            ..RecordingHistory::default()
        };

        let report = CleanSystemAreas::new(&cleaner, &history)
            .execute(&rules(), "prompt")
            .unwrap();

        assert_eq!(report.unrecorded, 1);
    }

    #[test]
    fn a_cancelled_authentication_removes_and_records_nothing() {
        let cleaner = FixedCleaner(Err(PrivilegeError::Cancelled));
        let history = RecordingHistory::default();

        let result = CleanSystemAreas::new(&cleaner, &history).execute(&rules(), "prompt");

        assert_eq!(result.unwrap_err(), PrivilegeError::Cancelled);
        assert!(history.appended.lock().unwrap().is_empty());
    }
}

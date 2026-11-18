//! Use case: remove what an uninstall needs an administrator for.

use std::path::PathBuf;

use crate::ports::{HistoryRepository, PrivilegeError, PrivilegedCleaner};
use crate::use_cases::DeletionReport;

/// Removes the system items of an app through the privileged cleaner and
/// records what was removed in the history of the user.
pub struct UninstallSystemItems<'a> {
    cleaner: &'a dyn PrivilegedCleaner,
    history: &'a dyn HistoryRepository,
}

impl<'a> UninstallSystemItems<'a> {
    /// Creates the use case over the privileged cleaner and the history.
    #[must_use]
    pub fn new(cleaner: &'a dyn PrivilegedCleaner, history: &'a dyn HistoryRepository) -> Self {
        Self { cleaner, history }
    }

    /// Removes `items` of the app whose bundle identifier is `bundle_id`.
    /// macOS authenticates the user first, showing `prompt` in its dialog.
    ///
    /// # Errors
    ///
    /// Fails when the user does not authenticate, or when the removal is
    /// refused or cannot run.
    pub fn execute(
        &self,
        bundle_id: &str,
        items: &[PathBuf],
        prompt: &str,
    ) -> Result<DeletionReport, PrivilegeError> {
        let mut report = DeletionReport::default();
        for outcome in self.cleaner.uninstall(bundle_id, items, prompt)? {
            report.record_in_history(outcome, self.history);
        }
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Mutex;

    use diskmarshal_domain::deletion::{DeletionOutcome, DeletionStatus, HistoryEntry};
    use diskmarshal_domain::shared::ByteSize;

    use super::UninstallSystemItems;
    use crate::ApplicationError;
    use crate::ports::{HistoryRepository, PrivilegeError, PrivilegedCleaner};

    /// A cleaner that removes whatever it is asked to, unless told to fail.
    struct ObedientCleaner(Option<PrivilegeError>);

    impl PrivilegedCleaner for ObedientCleaner {
        fn is_available(&self) -> bool {
            true
        }

        fn clean(
            &self,
            _rule_ids: &[String],
            _prompt: &str,
        ) -> Result<Vec<DeletionOutcome>, PrivilegeError> {
            Ok(Vec::new())
        }

        fn uninstall(
            &self,
            _bundle_id: &str,
            items: &[PathBuf],
            _prompt: &str,
        ) -> Result<Vec<DeletionOutcome>, PrivilegeError> {
            if let Some(failure) = &self.0 {
                return Err(failure.clone());
            }
            Ok(items
                .iter()
                .map(|item| DeletionOutcome {
                    target: item.clone(),
                    status: DeletionStatus::Deleted,
                    reclaimable: ByteSize::from_bytes(500),
                })
                .collect())
        }
    }

    #[derive(Default)]
    struct RecordingHistory(Mutex<Vec<PathBuf>>);

    impl HistoryRepository for RecordingHistory {
        fn append(&self, outcome: &DeletionOutcome) -> Result<(), ApplicationError> {
            if let Ok(mut appended) = self.0.lock() {
                appended.push(outcome.target.clone());
            }
            Ok(())
        }

        fn recent(&self, _limit: usize) -> Vec<HistoryEntry> {
            Vec::new()
        }
    }

    fn items() -> Vec<PathBuf> {
        vec![
            PathBuf::from("/Applications/Editor.app"),
            PathBuf::from("/Library/LaunchDaemons/com.vendor.editor.plist"),
        ]
    }

    #[test]
    fn records_what_the_administrator_removed() {
        let history = RecordingHistory::default();

        let report = UninstallSystemItems::new(&ObedientCleaner(None), &history)
            .execute("com.vendor.editor", &items(), "prompt")
            .unwrap();

        assert_eq!(report.reclaimed, ByteSize::from_bytes(1_000));
        assert_eq!(*history.0.lock().unwrap(), items());
    }

    #[test]
    fn a_cancelled_authentication_removes_and_records_nothing() {
        let history = RecordingHistory::default();
        let cleaner = ObedientCleaner(Some(PrivilegeError::Cancelled));

        let result = UninstallSystemItems::new(&cleaner, &history).execute(
            "com.vendor.editor",
            &items(),
            "prompt",
        );

        assert_eq!(result.unwrap_err(), PrivilegeError::Cancelled);
        assert!(history.0.lock().unwrap().is_empty());
    }
}

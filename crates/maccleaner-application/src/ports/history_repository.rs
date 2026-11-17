//! Port for the record of everything MacCleaner removed.

use maccleaner_domain::deletion::DeletionOutcome;

use crate::ApplicationError;

/// Keeps a durable record of removals, so the user can audit them later.
pub trait HistoryRepository: Send + Sync {
    /// Records one removal that really happened.
    ///
    /// # Errors
    ///
    /// Fails when the record cannot be stored.
    fn append(&self, outcome: &DeletionOutcome) -> Result<(), ApplicationError>;
}

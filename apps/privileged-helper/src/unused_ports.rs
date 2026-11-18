//! Adapters for the two things the helper never does: it has no Trash to
//! move items to, and it keeps no history. The app records, in the history
//! of the user, what the helper reports.

use std::io;
use std::path::Path;

use maccleaner_application::ApplicationError;
use maccleaner_application::ports::{HistoryRepository, TrashBin};
use maccleaner_domain::deletion::{DeletionOutcome, HistoryEntry};

/// A Trash that refuses everything. Administrator rules remove permanently;
/// a rule that asked for the Trash is reported as failed instead.
pub(crate) struct NoTrashBin;

impl TrashBin for NoTrashBin {
    fn move_to_trash(&self, path: &Path) -> Result<(), ApplicationError> {
        Err(ApplicationError::Io {
            path: path.to_path_buf(),
            source: io::Error::new(io::ErrorKind::Unsupported, "the helper has no Trash"),
        })
    }
}

/// A history that keeps nothing.
pub(crate) struct NoHistory;

impl HistoryRepository for NoHistory {
    fn append(&self, _outcome: &DeletionOutcome) -> Result<(), ApplicationError> {
        Ok(())
    }

    fn recent(&self, _limit: usize) -> Vec<HistoryEntry> {
        Vec::new()
    }
}

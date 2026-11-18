//! Adapters for what the helper never does: it has no Trash to move items
//! to, it keeps no history, and in a dry run it changes no setting. The app
//! records, in the history of the user, what the helper reports.

use std::io;
use std::path::Path;

use diskmarshal_application::ApplicationError;
use diskmarshal_application::ports::{
    HistoryRepository, NetworkSettings, NetworkSettingsError, TrashBin,
};
use diskmarshal_domain::deletion::{DeletionOutcome, HistoryEntry};
use diskmarshal_domain::network::NetworkChange;

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

/// Network settings that can be read and never change: what a dry run plugs
/// in, so a request is checked against this Mac without touching it.
pub(crate) struct UnchangedSettings<'a>(pub &'a dyn NetworkSettings);

impl NetworkSettings for UnchangedSettings<'_> {
    fn services(&self) -> Result<Vec<String>, NetworkSettingsError> {
        self.0.services()
    }

    fn apply(&self, _change: &NetworkChange) -> Result<(), NetworkSettingsError> {
        Ok(())
    }
}

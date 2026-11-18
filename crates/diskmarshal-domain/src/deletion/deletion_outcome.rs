//! What happened to each item of a deletion.

use std::path::PathBuf;

use crate::deletion::ProtectionReason;
use crate::shared::ByteSize;

/// Why an approved item was left untouched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    /// The deletion policy forbids removing this path.
    Protected(ProtectionReason),
    /// The item no longer exists.
    Missing,
    /// The item is not the one the user approved: it changed after the preview.
    ChangedSincePreview,
    /// The operating system refused the removal; the text says why.
    Failed(String),
}

/// Final state of one requested removal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeletionStatus {
    /// Removed permanently.
    Deleted,
    /// Moved to the Trash.
    Trashed,
    /// Simulation only: the item passed every check and would be removed.
    WouldDelete,
    /// Left untouched.
    Skipped(SkipReason),
}

/// The result for one requested removal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletionOutcome {
    /// The item, as resolved on disk when it was checked.
    pub target: PathBuf,
    /// What happened to it.
    pub status: DeletionStatus,
    /// Space the preview said the removal would free.
    pub reclaimable: ByteSize,
}

impl DeletionOutcome {
    /// Returns `true` when the item left the disk or went to the Trash.
    #[must_use]
    pub fn is_removed(&self) -> bool {
        matches!(
            self.status,
            DeletionStatus::Deleted | DeletionStatus::Trashed
        )
    }
}

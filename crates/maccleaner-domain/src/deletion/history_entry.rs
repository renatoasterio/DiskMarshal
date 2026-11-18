//! One line of the record of what MacCleaner removed.

use std::path::PathBuf;

use crate::shared::ByteSize;

/// How an item left its place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HistoryAction {
    /// Removed permanently.
    Deleted,
    /// Moved to the Trash.
    Trashed,
}

/// A removal that happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    /// When it happened, in seconds since the Unix epoch.
    pub timestamp_unix_seconds: i64,
    /// What was done.
    pub action: HistoryAction,
    /// The item that was removed.
    pub path: PathBuf,
    /// Space the preview said the removal would free.
    pub reclaimable: ByteSize,
}

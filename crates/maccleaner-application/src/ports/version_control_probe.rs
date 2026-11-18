//! Port for asking a version control system about a folder.

use std::path::Path;

/// Tells whether a folder holds files that are under version control.
pub trait VersionControlProbe: Send + Sync {
    /// Returns `true` when `folder`, inside `project`, has at least one file
    /// tracked by version control, or when that cannot be ruled out.
    ///
    /// A folder with tracked files is real project content, never a build
    /// leftover, so it must not be offered for removal.
    fn has_tracked_files(&self, project: &Path, folder: &str) -> bool;
}

//! Port for reading a folder hierarchy from disk.

use std::path::{Path, PathBuf};

use maccleaner_domain::scan::{FileTree, ScanPolicy};

use crate::ApplicationError;
use crate::progress::{CancellationToken, ProgressSink};

/// What a walk produced.
#[derive(Debug)]
pub struct WalkOutcome {
    /// Every item found, not yet aggregated.
    pub tree: FileTree,
    /// Folders the operating system refused to open.
    pub inaccessible_paths: Vec<PathBuf>,
}

/// Reads every item under a folder without changing anything on disk.
///
/// Implementations must never follow symbolic links, never leave the device
/// `root` is on, and never read the content of cloud-only items.
pub trait FileSystemWalker: Send + Sync {
    /// Walks `root`, honoring `policy`, and returns the items found.
    ///
    /// # Errors
    ///
    /// Fails when `root` does not exist, is not a folder, or cannot be read.
    /// Folders that cannot be opened further down are reported in
    /// [`WalkOutcome::inaccessible_paths`] and do not fail the walk.
    fn walk(
        &self,
        root: &Path,
        policy: &ScanPolicy,
        progress: &dyn ProgressSink,
        cancellation: &CancellationToken,
    ) -> Result<WalkOutcome, ApplicationError>;
}

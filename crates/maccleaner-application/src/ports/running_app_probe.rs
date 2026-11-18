//! Port for finding out which programs are running.

use std::path::PathBuf;

/// Tells which programs are running right now.
///
/// An app that is open recreates its files as soon as they are removed, so
/// the uninstaller refuses it.
pub trait RunningAppProbe: Send + Sync {
    /// Returns the path of the executable of every running program this
    /// user is allowed to see.
    fn running_executables(&self) -> Vec<PathBuf>;
}

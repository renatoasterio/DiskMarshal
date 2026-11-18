//! Port for letting the user choose a folder.

use std::path::PathBuf;

/// Asks the user to choose a folder with the system's own dialog.
pub trait FolderPicker: Send + Sync {
    /// Shows the dialog and waits. Returns `None` when the user cancels.
    fn pick_folder(&self) -> Option<PathBuf>;
}

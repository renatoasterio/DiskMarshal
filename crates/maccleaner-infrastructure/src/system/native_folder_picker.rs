//! Folder picker that uses the macOS open panel.

use std::path::PathBuf;

use maccleaner_application::ports::FolderPicker;

/// Shows the standard macOS panel for choosing a folder.
///
/// The panel must be opened from the main thread, which is where the
/// interface runs.
#[derive(Debug, Clone, Copy, Default)]
pub struct NativeFolderPicker;

impl FolderPicker for NativeFolderPicker {
    fn pick_folder(&self) -> Option<PathBuf> {
        rfd::FileDialog::new().pick_folder()
    }
}

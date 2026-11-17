//! Folders macOS synchronizes with iCloud.

use std::env;
use std::path::PathBuf;

/// Folder, relative to the home folder, where iCloud Drive keeps its content.
const ICLOUD_DRIVE_RELATIVE_PATH: &str = "Library/Mobile Documents";

/// Returns the folders whose content is synchronized with iCloud for the
/// current user. The list is empty when the home folder is unknown.
#[must_use]
pub fn icloud_sync_roots() -> Vec<PathBuf> {
    env::var_os("HOME")
        .map(|home| PathBuf::from(home).join(ICLOUD_DRIVE_RELATIVE_PATH))
        .into_iter()
        .collect()
}

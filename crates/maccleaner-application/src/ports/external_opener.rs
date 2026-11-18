//! Port for handing something over to macOS: Finder or System Settings.

use std::path::Path;

use crate::ApplicationError;

/// Opens things outside MacCleaner.
pub trait ExternalOpener: Send + Sync {
    /// Shows the item at `path` selected in a Finder window.
    ///
    /// # Errors
    ///
    /// Fails when macOS cannot show the item, for example because it is gone.
    fn reveal_in_finder(&self, path: &Path) -> Result<(), ApplicationError>;

    /// Opens the Full Disk Access page of System Settings.
    ///
    /// # Errors
    ///
    /// Fails when System Settings cannot be opened.
    fn open_full_disk_access_settings(&self) -> Result<(), ApplicationError>;
}

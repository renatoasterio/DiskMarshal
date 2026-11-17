//! Port for removing an item permanently.

use std::path::Path;

use crate::ApplicationError;

/// Removes an item, and everything beneath it, for good.
///
/// Implementations must never follow symbolic links and never remove
/// anything that lives on another device.
pub trait FileRemover: Send + Sync {
    /// Removes the item at `path`.
    ///
    /// # Errors
    ///
    /// Fails when the operating system refuses the removal.
    fn remove(&self, path: &Path) -> Result<(), ApplicationError>;
}

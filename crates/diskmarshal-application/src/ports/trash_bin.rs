//! Port for moving an item to the Trash.

use std::path::Path;

use crate::ApplicationError;

/// Moves an item to the user's Trash, from where it can be restored.
pub trait TrashBin: Send + Sync {
    /// Moves the item at `path` to the Trash.
    ///
    /// # Errors
    ///
    /// Fails when the operating system refuses the move.
    fn move_to_trash(&self, path: &Path) -> Result<(), ApplicationError>;
}

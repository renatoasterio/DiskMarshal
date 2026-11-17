//! Port for reading facts about one item without changing it.

use std::path::{Path, PathBuf};

use maccleaner_domain::deletion::ItemFingerprint;

/// Reads the identity and the real location of an item.
pub trait FileInspector: Send + Sync {
    /// Returns the real path of `path`: every symbolic link in its parent
    /// folders is resolved, and the last component is kept as it is.
    ///
    /// Returns `None` when the parent folder does not exist.
    fn resolve(&self, path: &Path) -> Option<PathBuf>;

    /// Returns what identifies the item at `path` right now, without
    /// following a symbolic link at `path` itself.
    ///
    /// Returns `None` when the item does not exist.
    fn fingerprint(&self, path: &Path) -> Option<ItemFingerprint>;
}

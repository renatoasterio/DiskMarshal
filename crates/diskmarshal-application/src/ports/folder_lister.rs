//! Port for reading the names directly inside one folder.

use std::path::Path;

use diskmarshal_domain::shared::ByteSize;

/// One item directly inside a folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderItem {
    /// File name, without the parent path.
    pub name: String,
    /// Whether the item is a real folder. A symbolic link never is.
    pub is_folder: bool,
    /// Space the item itself occupies on disk, without what is inside it.
    pub allocated_size: ByteSize,
}

/// Lists what is directly inside a folder, without entering anything.
///
/// Implementations must never follow symbolic links and never read the
/// content of any item.
pub trait FolderLister: Send + Sync {
    /// Returns the items directly inside `folder`. A folder that does not
    /// exist or cannot be read simply has none.
    fn items(&self, folder: &Path) -> Vec<FolderItem>;
}

//! What the file system reports about a single item.

use crate::scan::CloudState;
use crate::shared::ByteSize;

/// The kind of item found during a scan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntryKind {
    /// A regular folder, which a scan enters.
    Directory,
    /// A regular file.
    File,
    /// A symbolic link. Scans never follow it.
    Symlink,
    /// A folder macOS presents as one item (an app, a Photos library). It is
    /// measured as a whole and never opened in the tree.
    Package,
    /// Sockets, devices and anything else that holds no user data.
    Other,
}

/// One item as reported by the file system, before it joins a [`FileTree`].
///
/// Sizes are the item's own: a directory entry starts without the size of its
/// content, which the tree adds when it aggregates.
///
/// [`FileTree`]: crate::scan::FileTree
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    /// File name, without the parent path.
    pub name: Box<str>,
    /// What the item is.
    pub kind: EntryKind,
    /// Where its content lives in relation to iCloud.
    pub cloud_state: CloudState,
    /// Size of the content, as an application reading it would see.
    pub logical_size: ByteSize,
    /// Space the item really occupies on disk.
    pub allocated_size: ByteSize,
    /// Last modification, in seconds since the Unix epoch, when known.
    pub modified_unix_seconds: Option<i64>,
}

impl FileEntry {
    /// Creates a local directory entry with no size of its own.
    #[must_use]
    pub fn directory(name: &str) -> Self {
        Self {
            name: name.into(),
            kind: EntryKind::Directory,
            cloud_state: CloudState::Local,
            logical_size: ByteSize::ZERO,
            allocated_size: ByteSize::ZERO,
            modified_unix_seconds: None,
        }
    }

    /// Creates a local file entry whose logical and allocated sizes are equal.
    #[must_use]
    pub fn file(name: &str, size: ByteSize) -> Self {
        Self {
            name: name.into(),
            kind: EntryKind::File,
            cloud_state: CloudState::Local,
            logical_size: size,
            allocated_size: size,
            modified_unix_seconds: None,
        }
    }
}

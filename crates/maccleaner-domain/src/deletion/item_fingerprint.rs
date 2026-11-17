//! Identity of an item at the moment the user saw it in a preview.

/// What identified an item when it was offered for deletion.
///
/// The same values are read again right before removal. If any of them
/// differs, the item is no longer the one the user approved and is skipped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemFingerprint {
    /// Device the item lives on.
    pub device: u64,
    /// Inode number: stays the same while the item is the same object.
    pub inode: u64,
    /// Size of the item itself, in bytes.
    pub size: u64,
    /// Last modification, in seconds since the Unix epoch.
    pub modified_unix_seconds: i64,
}

//! A removal the user approved.

use std::path::PathBuf;

use crate::deletion::ItemFingerprint;
use crate::shared::ByteSize;

/// How an item leaves the disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeletionMethod {
    /// Moved to the Trash, from where the user can restore it. Used for
    /// personal files such as duplicates and photos.
    Trash,
    /// Removed at once. Used only for data that can be recreated: caches,
    /// temporary files and build folders.
    Permanent,
}

/// One item the user approved for removal in a preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletionRequest {
    /// Absolute path of the item.
    pub target: PathBuf,
    /// How to remove it.
    pub method: DeletionMethod,
    /// What identified the item when the user approved it.
    pub fingerprint: ItemFingerprint,
    /// Space the preview said the removal would free.
    pub reclaimable: ByteSize,
}

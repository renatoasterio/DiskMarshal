//! One item of a scanned tree.

use crate::scan::{CloudState, EntryKind, FileEntry};
use crate::shared::ByteSize;

/// Identifies a node inside one [`FileTree`](crate::scan::FileTree).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub(super) u32);

impl NodeId {
    pub(super) fn index(self) -> usize {
        self.0 as usize
    }
}

/// One item of the tree, with the totals of everything beneath it.
#[derive(Debug, Clone)]
pub struct FileNode {
    pub(super) entry: FileEntry,
    pub(super) item_count: u64,
    pub(super) cloud_only_size: ByteSize,
    pub(super) parent: Option<NodeId>,
    pub(super) first_child: Option<NodeId>,
    pub(super) last_child: Option<NodeId>,
    pub(super) next_sibling: Option<NodeId>,
}

impl FileNode {
    pub(super) fn new(entry: FileEntry, parent: Option<NodeId>) -> Self {
        let cloud_only_size = if entry.cloud_state.is_cloud_only() {
            entry.logical_size
        } else {
            ByteSize::ZERO
        };
        Self {
            entry,
            item_count: 1,
            cloud_only_size,
            parent,
            first_child: None,
            last_child: None,
            next_sibling: None,
        }
    }

    /// File name, without the parent path.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.entry.name
    }

    /// What the item is.
    #[must_use]
    pub fn kind(&self) -> EntryKind {
        self.entry.kind
    }

    /// Where the item's own content lives in relation to iCloud.
    #[must_use]
    pub fn cloud_state(&self) -> CloudState {
        self.entry.cloud_state
    }

    /// Content size of the item and, after aggregation, of everything beneath it.
    #[must_use]
    pub fn logical_size(&self) -> ByteSize {
        self.entry.logical_size
    }

    /// Disk space used by the item and, after aggregation, by everything beneath it.
    #[must_use]
    pub fn allocated_size(&self) -> ByteSize {
        self.entry.allocated_size
    }

    /// Content size that exists only in iCloud, for the item and everything beneath it.
    #[must_use]
    pub fn cloud_only_size(&self) -> ByteSize {
        self.cloud_only_size
    }

    /// Number of items in this subtree, the node itself included.
    #[must_use]
    pub fn item_count(&self) -> u64 {
        self.item_count
    }

    /// Last modification, in seconds since the Unix epoch, when known.
    #[must_use]
    pub fn modified_unix_seconds(&self) -> Option<i64> {
        self.entry.modified_unix_seconds
    }

    /// The node that contains this one; `None` for the root.
    #[must_use]
    pub fn parent(&self) -> Option<NodeId> {
        self.parent
    }
}

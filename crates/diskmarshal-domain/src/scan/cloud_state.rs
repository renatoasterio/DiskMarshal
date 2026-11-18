//! Where the content of an item lives in relation to iCloud.

/// Whether an item's content is on this Mac, in iCloud, or both.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum CloudState {
    /// The item exists only on this Mac.
    #[default]
    Local,
    /// The item is synchronized with iCloud and its content is downloaded.
    Synced,
    /// The item is a placeholder: its content exists only in iCloud.
    CloudOnly,
}

impl CloudState {
    /// Classifies an item from two facts the file system reports about it.
    ///
    /// A dataless item is always cloud-only, wherever it is.
    #[must_use]
    pub const fn classify(is_dataless: bool, is_inside_sync_root: bool) -> Self {
        match (is_dataless, is_inside_sync_root) {
            (true, _) => Self::CloudOnly,
            (false, true) => Self::Synced,
            (false, false) => Self::Local,
        }
    }

    /// Returns `true` when reading the item's content would trigger a download.
    ///
    /// No part of DiskMarshal may read the content of such an item.
    #[must_use]
    pub const fn is_cloud_only(self) -> bool {
        matches!(self, Self::CloudOnly)
    }
}

#[cfg(test)]
mod tests {
    use super::CloudState;

    #[test]
    fn dataless_items_are_cloud_only_wherever_they_are() {
        assert_eq!(CloudState::classify(true, true), CloudState::CloudOnly);
        assert_eq!(CloudState::classify(true, false), CloudState::CloudOnly);
    }

    #[test]
    fn downloaded_items_inside_a_sync_root_are_synced() {
        assert_eq!(CloudState::classify(false, true), CloudState::Synced);
    }

    #[test]
    fn everything_else_is_local() {
        assert_eq!(CloudState::classify(false, false), CloudState::Local);
        assert!(!CloudState::Local.is_cloud_only());
    }
}

//! Files confirmed to have identical content.

use std::path::PathBuf;

use crate::shared::ByteSize;

/// One of the identical files of a group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateCopy {
    /// Absolute path of the file.
    pub path: PathBuf,
    /// Disk space the file uses.
    pub allocated: ByteSize,
    /// Last modification, in seconds since the Unix epoch, when known.
    pub modified_unix_seconds: Option<i64>,
}

/// Two or more files with identical content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateGroup {
    /// Content size of each copy.
    pub size: ByteSize,
    /// The copies, with the one to keep first.
    pub copies: Vec<DuplicateCopy>,
}

impl DuplicateGroup {
    /// Creates a group and puts the copy to keep first: the oldest one, and
    /// among equally old ones the one with the shortest path. The oldest copy
    /// is most likely the original, and a short path is most likely where the
    /// user meant to keep it.
    #[must_use]
    pub fn new(size: ByteSize, mut copies: Vec<DuplicateCopy>) -> Self {
        copies.sort_by(|a, b| {
            let age = |copy: &DuplicateCopy| copy.modified_unix_seconds.unwrap_or(i64::MAX);
            age(a)
                .cmp(&age(b))
                .then_with(|| a.path.as_os_str().len().cmp(&b.path.as_os_str().len()))
                .then_with(|| a.path.cmp(&b.path))
        });
        Self { size, copies }
    }

    /// Space freed by removing every copy but the one to keep.
    #[must_use]
    pub fn reclaimable(&self) -> ByteSize {
        self.copies.iter().skip(1).map(|copy| copy.allocated).sum()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{DuplicateCopy, DuplicateGroup};
    use crate::shared::ByteSize;

    fn copy(path: &str, modified: Option<i64>) -> DuplicateCopy {
        DuplicateCopy {
            path: PathBuf::from(path),
            allocated: ByteSize::from_bytes(100),
            modified_unix_seconds: modified,
        }
    }

    #[test]
    fn keeps_the_oldest_copy() {
        let group = DuplicateGroup::new(
            ByteSize::from_bytes(100),
            vec![
                copy("/a/new.mov", Some(900)),
                copy("/b/deep/old.mov", Some(100)),
            ],
        );
        assert_eq!(group.copies[0].path, PathBuf::from("/b/deep/old.mov"));
        assert_eq!(group.reclaimable(), ByteSize::from_bytes(100));
    }

    #[test]
    fn among_equally_old_copies_keeps_the_shortest_path() {
        let group = DuplicateGroup::new(
            ByteSize::from_bytes(100),
            vec![
                copy("/Users/t/Downloads/copy of file.pdf", Some(5)),
                copy("/Users/t/Docs/file.pdf", Some(5)),
                copy("/Users/t/unknown-date.pdf", None),
            ],
        );
        assert_eq!(
            group.copies[0].path,
            PathBuf::from("/Users/t/Docs/file.pdf")
        );
        assert_eq!(
            group.copies[2].path,
            PathBuf::from("/Users/t/unknown-date.pdf")
        );
        assert_eq!(group.reclaimable(), ByteSize::from_bytes(200));
    }
}

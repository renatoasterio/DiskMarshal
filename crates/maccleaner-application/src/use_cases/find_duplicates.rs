//! Use case: find files with identical content.

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

use maccleaner_domain::deletion::DeletionRequest;
use maccleaner_domain::duplicates::{
    ContentHash, DuplicateCopy, DuplicateGroup, candidates_by_size,
};
use maccleaner_domain::scan::{LocalFile, ScanPolicy};
use maccleaner_domain::shared::ByteSize;

use crate::ApplicationError;
use crate::ports::{ContentHasher, FileInspector, FileSystemWalker};
use crate::progress::{CancellationToken, ProgressSink};
use crate::use_cases::trash_requests::trash_requests;

/// A group of identical files with the removal of each copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundDuplicates {
    /// The identical files, with the one to keep first.
    pub group: DuplicateGroup,
    /// One removal per copy, in the same order. Copies go to the Trash.
    pub requests: Vec<DeletionRequest>,
}

/// What a search for duplicates found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicatesReport {
    /// The folder that was searched, as resolved on disk.
    pub root: PathBuf,
    /// The groups, the one that frees the most space first.
    pub groups: Vec<FoundDuplicates>,
}

/// Searches a folder for files with identical content.
pub struct FindDuplicates<'a> {
    walker: &'a dyn FileSystemWalker,
    hasher: &'a dyn ContentHasher,
    inspector: &'a dyn FileInspector,
}

impl<'a> FindDuplicates<'a> {
    /// Creates the use case over the adapters it reads through.
    #[must_use]
    pub fn new(
        walker: &'a dyn FileSystemWalker,
        hasher: &'a dyn ContentHasher,
        inspector: &'a dyn FileInspector,
    ) -> Self {
        Self {
            walker,
            hasher,
            inspector,
        }
    }

    /// Searches `root` for identical files of at least `minimum` bytes.
    ///
    /// Files are compared in three steps, each cheaper than the next: same
    /// size, same beginning and end, same whole content. Only files that
    /// survive one step are read by the following one. A file that cannot be
    /// read is left out.
    ///
    /// # Errors
    ///
    /// Fails when `root` cannot be scanned or the search is cancelled.
    pub fn execute(
        &self,
        root: &Path,
        minimum: ByteSize,
        progress: &dyn ProgressSink,
        cancellation: &CancellationToken,
    ) -> Result<DuplicatesReport, ApplicationError> {
        let root = self
            .inspector
            .resolve(root)
            .ok_or_else(|| ApplicationError::RootNotFound(root.to_path_buf()))?;
        let mut tree = self
            .walker
            .walk(&root, &ScanPolicy::default(), progress, cancellation)?
            .tree;
        tree.aggregate();

        let mut groups = Vec::new();
        for (size, same_size) in candidates_by_size(&tree, &root, minimum) {
            let quick = |path: &Path| self.hasher.quick_hash(path);
            for same_start in split_by(same_size, &quick, cancellation)? {
                let full = |path: &Path| self.hasher.full_hash(path);
                for identical in split_by(same_start, &full, cancellation)? {
                    groups.extend(self.found(size, identical));
                }
            }
        }
        groups.sort_by_key(|found| std::cmp::Reverse(found.group.reclaimable()));
        Ok(DuplicatesReport { root, groups })
    }

    /// Turns identical files into a group, when each can still be identified.
    fn found(&self, size: ByteSize, files: Vec<LocalFile>) -> Option<FoundDuplicates> {
        let copies = files
            .into_iter()
            .map(|file| DuplicateCopy {
                path: file.path,
                allocated: file.allocated,
                modified_unix_seconds: file.modified_unix_seconds,
            })
            .collect();
        let group = DuplicateGroup::new(size, copies);
        let files = group
            .copies
            .iter()
            .map(|copy| (copy.path.as_path(), copy.allocated));
        let requests = trash_requests(self.inspector, files)?;
        Some(FoundDuplicates { group, requests })
    }
}

/// Splits `files` into the sets that share a digest, keeping only sets with at
/// least two files. A file whose digest cannot be computed is dropped.
fn split_by(
    files: Vec<LocalFile>,
    digest: &dyn Fn(&Path) -> io::Result<ContentHash>,
    cancellation: &CancellationToken,
) -> Result<Vec<Vec<LocalFile>>, ApplicationError> {
    let mut by_digest: BTreeMap<ContentHash, Vec<LocalFile>> = BTreeMap::new();
    for file in files {
        if cancellation.is_cancelled() {
            return Err(ApplicationError::Cancelled);
        }
        if let Ok(hash) = digest(&file.path) {
            by_digest.entry(hash).or_default().push(file);
        }
    }
    Ok(by_digest
        .into_values()
        .filter(|set| set.len() > 1)
        .collect())
}

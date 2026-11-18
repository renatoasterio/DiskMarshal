//! Use case: measure everything under a folder.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use diskmarshal_domain::scan::{FileTree, ScanPolicy};

use crate::ApplicationError;
use crate::ports::FileSystemWalker;
use crate::progress::{CancellationToken, ProgressSink};

/// The result of scanning a folder.
#[derive(Debug)]
pub struct ScanReport {
    /// The folder that was scanned.
    pub root: PathBuf,
    /// Every item found, with sizes rolled up into each folder.
    pub tree: FileTree,
    /// Folders the operating system refused to open.
    pub inaccessible_paths: Vec<PathBuf>,
    /// How long the scan took.
    pub elapsed: Duration,
}

/// Scans a folder and returns its tree with aggregated sizes.
pub struct ScanDisk<'a> {
    walker: &'a dyn FileSystemWalker,
}

impl<'a> ScanDisk<'a> {
    /// Creates the use case over the given walker.
    #[must_use]
    pub fn new(walker: &'a dyn FileSystemWalker) -> Self {
        Self { walker }
    }

    /// Scans `root`.
    ///
    /// # Errors
    ///
    /// Fails when `root` cannot be scanned or when `cancellation` is triggered
    /// before the scan finishes. A cancelled scan never returns a partial tree.
    pub fn execute(
        &self,
        root: &Path,
        progress: &dyn ProgressSink,
        cancellation: &CancellationToken,
    ) -> Result<ScanReport, ApplicationError> {
        let started = Instant::now();
        let policy = ScanPolicy::for_root(root);
        let outcome = self.walker.walk(root, &policy, progress, cancellation)?;
        if cancellation.is_cancelled() {
            return Err(ApplicationError::Cancelled);
        }

        let mut tree = outcome.tree;
        tree.aggregate();
        Ok(ScanReport {
            root: root.to_path_buf(),
            tree,
            inaccessible_paths: outcome.inaccessible_paths,
            elapsed: started.elapsed(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use diskmarshal_domain::scan::{FileEntry, FileTree, ScanPolicy};
    use diskmarshal_domain::shared::ByteSize;

    use super::ScanDisk;
    use crate::ApplicationError;
    use crate::ports::{FileSystemWalker, WalkOutcome};
    use crate::progress::{CancellationToken, NoProgress, ProgressSink};

    /// A walker that returns a fixed tree and can cancel the token while "walking".
    struct FakeWalker {
        cancels_during_walk: bool,
    }

    impl FileSystemWalker for FakeWalker {
        fn walk(
            &self,
            _root: &Path,
            _policy: &ScanPolicy,
            _progress: &dyn ProgressSink,
            cancellation: &CancellationToken,
        ) -> Result<WalkOutcome, ApplicationError> {
            if self.cancels_during_walk {
                cancellation.cancel();
            }
            let mut tree = FileTree::new(FileEntry::directory("root"));
            tree.add_child(
                FileTree::ROOT,
                FileEntry::file("a", ByteSize::from_bytes(40)),
            );
            tree.add_child(
                FileTree::ROOT,
                FileEntry::file("b", ByteSize::from_bytes(2)),
            );
            Ok(WalkOutcome {
                tree,
                inaccessible_paths: Vec::new(),
            })
        }
    }

    #[test]
    fn returns_the_tree_with_sizes_rolled_up() {
        let walker = FakeWalker {
            cancels_during_walk: false,
        };
        let report = ScanDisk::new(&walker)
            .execute(Path::new("/any"), &NoProgress, &CancellationToken::new())
            .unwrap();

        let root = report.tree.root().unwrap();
        assert_eq!(root.logical_size(), ByteSize::from_bytes(42));
        assert_eq!(root.item_count(), 3);
        assert_eq!(report.root, Path::new("/any"));
    }

    #[test]
    fn a_cancelled_scan_never_returns_a_partial_tree() {
        let walker = FakeWalker {
            cancels_during_walk: true,
        };
        let result = ScanDisk::new(&walker).execute(
            Path::new("/any"),
            &NoProgress,
            &CancellationToken::new(),
        );
        assert!(matches!(result, Err(ApplicationError::Cancelled)));
    }
}

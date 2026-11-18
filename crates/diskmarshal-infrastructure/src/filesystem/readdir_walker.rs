//! Parallel walker built on `readdir` and `lstat`.

use std::collections::HashSet;
use std::fs::{self, DirEntry, Metadata};
use std::io;
use std::os::macos::fs::MetadataExt as _;
use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};

use diskmarshal_application::ApplicationError;
use diskmarshal_application::ports::{FileSystemWalker, WalkOutcome};
use diskmarshal_application::progress::{CancellationToken, ProgressSink, ScanProgress};
use diskmarshal_domain::scan::{CloudState, EntryKind, FileEntry, FileTree, NodeId, ScanPolicy};
use diskmarshal_domain::shared::ByteSize;
use rayon::prelude::*;

use crate::filesystem::STAT_BLOCK_BYTES;

/// `st_flags` bit macOS sets on items whose content exists only in the cloud.
const SF_DATALESS: u32 = 0x4000_0000;

/// Walks a folder with one `readdir` per folder and one `lstat` per item,
/// spreading sibling folders over the rayon thread pool.
#[derive(Debug, Clone, Default)]
pub struct ReaddirWalker {
    sync_roots: Vec<PathBuf>,
}

impl ReaddirWalker {
    /// Creates a walker that reports items under `sync_roots` as synchronized
    /// with iCloud.
    #[must_use]
    pub fn new(sync_roots: Vec<PathBuf>) -> Self {
        Self { sync_roots }
    }
}

impl FileSystemWalker for ReaddirWalker {
    fn walk(
        &self,
        root: &Path,
        policy: &ScanPolicy,
        progress: &dyn ProgressSink,
        cancellation: &CancellationToken,
    ) -> Result<WalkOutcome, ApplicationError> {
        let metadata = fs::symlink_metadata(root).map_err(|source| match source.kind() {
            io::ErrorKind::NotFound => ApplicationError::RootNotFound(root.to_path_buf()),
            _ => ApplicationError::Io {
                path: root.to_path_buf(),
                source,
            },
        })?;
        if !metadata.is_dir() {
            return Err(ApplicationError::RootNotDirectory(root.to_path_buf()));
        }

        let context = WalkContext {
            root_device: metadata.dev(),
            policy,
            sync_roots: &self.sync_roots,
            progress,
            cancellation,
            seen_hard_links: Mutex::default(),
            inaccessible_paths: Mutex::default(),
            items_seen: AtomicU64::new(0),
            bytes_seen: AtomicU64::new(0),
        };
        let children = match context.list(root) {
            Ok(children) => children,
            Err(ListingFailure::Inaccessible(source)) => {
                return Err(ApplicationError::Io {
                    path: root.to_path_buf(),
                    source,
                });
            }
            Err(ListingFailure::CloudOnly) => Vec::new(),
        };

        let mut tree = FileTree::new(FileEntry::directory(&root.to_string_lossy()));
        attach(&mut tree, FileTree::ROOT, children)?;
        let inaccessible_paths = context
            .inaccessible_paths
            .into_inner()
            .unwrap_or_else(PoisonError::into_inner);
        Ok(WalkOutcome {
            tree,
            inaccessible_paths,
        })
    }
}

/// An item and everything found beneath it, before it joins the tree.
struct ScannedEntry {
    entry: FileEntry,
    children: Vec<ScannedEntry>,
}

/// Why a folder could not be listed.
enum ListingFailure {
    /// The folder is a cloud-only placeholder; listing it would download it.
    CloudOnly,
    /// The operating system refused access.
    Inaccessible(io::Error),
}

/// State shared by every thread of one walk.
struct WalkContext<'a> {
    root_device: u64,
    policy: &'a ScanPolicy,
    sync_roots: &'a [PathBuf],
    progress: &'a dyn ProgressSink,
    cancellation: &'a CancellationToken,
    seen_hard_links: Mutex<HashSet<(u64, u64)>>,
    inaccessible_paths: Mutex<Vec<PathBuf>>,
    items_seen: AtomicU64,
    bytes_seen: AtomicU64,
}

impl WalkContext<'_> {
    /// Lists `directory` and, in parallel, every folder beneath it.
    fn list(&self, directory: &Path) -> Result<Vec<ScannedEntry>, ListingFailure> {
        if self.cancellation.is_cancelled() {
            return Ok(Vec::new());
        }
        let reader = fs::read_dir(directory).map_err(|error| {
            if error.raw_os_error() == Some(libc::EDEADLK) {
                ListingFailure::CloudOnly
            } else {
                ListingFailure::Inaccessible(error)
            }
        })?;

        let mut entries = Vec::new();
        let mut folders_to_enter = Vec::new();
        for dir_entry in reader.flatten() {
            // `DirEntry::metadata` does not follow symbolic links.
            let Ok(metadata) = dir_entry.metadata() else {
                continue;
            };
            let path = dir_entry.path();
            let entry = self.describe(&dir_entry, &metadata, &path);
            if self.should_enter(&entry, &metadata, &path) {
                folders_to_enter.push((entries.len(), path));
            }
            entries.push(ScannedEntry {
                entry,
                children: Vec::new(),
            });
        }
        self.report(&entries);

        let listings: Vec<_> = folders_to_enter
            .into_par_iter()
            .map(|(index, path)| {
                let listing = self.list(&path);
                (index, path, listing)
            })
            .collect();
        for (index, path, listing) in listings {
            let Some(folder) = entries.get_mut(index) else {
                continue;
            };
            match listing {
                Ok(children) if folder.entry.kind == EntryKind::Package => {
                    let (logical, allocated) = subtree_sizes(&children);
                    folder.entry.logical_size += logical;
                    folder.entry.allocated_size += allocated;
                }
                Ok(children) => folder.children = children,
                Err(ListingFailure::CloudOnly) => folder.entry.cloud_state = CloudState::CloudOnly,
                Err(ListingFailure::Inaccessible(_)) => self
                    .inaccessible_paths
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push(path),
            }
        }
        Ok(entries)
    }

    fn describe(&self, dir_entry: &DirEntry, metadata: &Metadata, path: &Path) -> FileEntry {
        let name = dir_entry.file_name().to_string_lossy().into_owned();
        let file_type = metadata.file_type();
        let kind = if file_type.is_symlink() {
            EntryKind::Symlink
        } else if file_type.is_dir() {
            if ScanPolicy::is_package(&name) {
                EntryKind::Package
            } else {
                EntryKind::Directory
            }
        } else if file_type.is_file() {
            EntryKind::File
        } else {
            EntryKind::Other
        };

        let is_dataless = metadata.st_flags() & SF_DATALESS != 0;
        let is_inside_sync_root = self.sync_roots.iter().any(|root| path.starts_with(root));
        let allocated = metadata.blocks().saturating_mul(STAT_BLOCK_BYTES);
        let (logical_size, allocated_size) = match kind {
            EntryKind::Directory | EntryKind::Package => (0, allocated),
            EntryKind::File if self.is_repeated_hard_link(metadata) => (0, 0),
            EntryKind::File | EntryKind::Symlink | EntryKind::Other => (metadata.size(), allocated),
        };

        FileEntry {
            name: name.into(),
            kind,
            cloud_state: CloudState::classify(is_dataless, is_inside_sync_root),
            logical_size: ByteSize::from_bytes(logical_size),
            allocated_size: ByteSize::from_bytes(allocated_size),
            modified_unix_seconds: Some(metadata.mtime()),
        }
    }

    /// Returns `true` for the second and later names of a file with several
    /// hard links, so its content is counted only once.
    fn is_repeated_hard_link(&self, metadata: &Metadata) -> bool {
        metadata.nlink() > 1
            && !self
                .seen_hard_links
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .insert((metadata.dev(), metadata.ino()))
    }

    fn should_enter(&self, entry: &FileEntry, metadata: &Metadata, path: &Path) -> bool {
        matches!(entry.kind, EntryKind::Directory | EntryKind::Package)
            && !entry.cloud_state.is_cloud_only()
            && metadata.dev() == self.root_device
            && !self.policy.is_pruned(path)
    }

    fn report(&self, entries: &[ScannedEntry]) {
        if entries.is_empty() {
            return;
        }
        let bytes: ByteSize = entries
            .iter()
            .map(|scanned| scanned.entry.allocated_size)
            .sum();
        let item_count = u64::try_from(entries.len()).unwrap_or(u64::MAX);
        let items_seen = self.items_seen.fetch_add(item_count, Ordering::Relaxed) + item_count;
        let bytes_seen = self
            .bytes_seen
            .fetch_add(bytes.as_bytes(), Ordering::Relaxed)
            + bytes.as_bytes();
        self.progress.on_progress(ScanProgress {
            items_seen,
            bytes_seen: ByteSize::from_bytes(bytes_seen),
        });
    }
}

/// Sums the logical and allocated sizes of `entries` and everything beneath them.
fn subtree_sizes(entries: &[ScannedEntry]) -> (ByteSize, ByteSize) {
    entries.iter().fold(
        (ByteSize::ZERO, ByteSize::ZERO),
        |(logical, allocated), scanned| {
            let (child_logical, child_allocated) = subtree_sizes(&scanned.children);
            (
                logical + scanned.entry.logical_size + child_logical,
                allocated + scanned.entry.allocated_size + child_allocated,
            )
        },
    )
}

/// Moves scanned entries into the tree, parents before children.
fn attach(
    tree: &mut FileTree,
    parent: NodeId,
    entries: Vec<ScannedEntry>,
) -> Result<(), ApplicationError> {
    for scanned in entries {
        let id = tree
            .add_child(parent, scanned.entry)
            .ok_or(ApplicationError::TreeCapacityExceeded)?;
        attach(tree, id, scanned.children)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use diskmarshal_application::ApplicationError;
    use diskmarshal_application::ports::FileSystemWalker;
    use diskmarshal_application::progress::{CancellationToken, NoProgress};
    use diskmarshal_domain::scan::{CloudState, EntryKind, FileNode, FileTree, NodeId, ScanPolicy};
    use diskmarshal_domain::shared::ByteSize;
    use diskmarshal_test_support::TempTree;

    use super::ReaddirWalker;

    fn walk(walker: &ReaddirWalker, root: &Path, policy: &ScanPolicy) -> FileTree {
        let outcome = walker
            .walk(root, policy, &NoProgress, &CancellationToken::new())
            .unwrap();
        let mut tree = outcome.tree;
        tree.aggregate();
        tree
    }

    fn child<'a>(tree: &'a FileTree, parent: NodeId, name: &str) -> (NodeId, &'a FileNode) {
        tree.children(parent)
            .filter_map(|id| tree.node(id).map(|node| (id, node)))
            .find(|(_, node)| node.name() == name)
            .unwrap()
    }

    #[test]
    fn measures_files_and_nested_folders() {
        let fixture = TempTree::new().unwrap();
        fixture.file("a.bin", 1_000).unwrap();
        fixture.file("sub/b.bin", 2_000).unwrap();

        let tree = walk(
            &ReaddirWalker::default(),
            fixture.root(),
            &ScanPolicy::default(),
        );

        let root = tree.root().unwrap();
        assert_eq!(root.logical_size(), ByteSize::from_bytes(3_000));
        assert_eq!(root.item_count(), 4);
        let (_, sub) = child(&tree, FileTree::ROOT, "sub");
        assert_eq!(sub.logical_size(), ByteSize::from_bytes(2_000));
    }

    #[test]
    fn never_follows_symbolic_links() {
        let outside = TempTree::new().unwrap();
        outside.file("big.bin", 50_000).unwrap();
        let fixture = TempTree::new().unwrap();
        fixture.symlink(outside.root(), "link").unwrap();

        let tree = walk(
            &ReaddirWalker::default(),
            fixture.root(),
            &ScanPolicy::default(),
        );

        let (link, node) = child(&tree, FileTree::ROOT, "link");
        assert_eq!(node.kind(), EntryKind::Symlink);
        assert_eq!(tree.children(link).count(), 0);
        assert!(tree.root().unwrap().logical_size() < ByteSize::from_bytes(50_000));
    }

    #[test]
    fn counts_a_hard_linked_file_once() {
        let fixture = TempTree::new().unwrap();
        fixture.file("original.bin", 4_000).unwrap();
        fixture.hard_link("original.bin", "copy.bin").unwrap();

        let tree = walk(
            &ReaddirWalker::default(),
            fixture.root(),
            &ScanPolicy::default(),
        );

        assert_eq!(
            tree.root().unwrap().logical_size(),
            ByteSize::from_bytes(4_000)
        );
    }

    #[test]
    fn measures_a_package_as_a_single_item() {
        let fixture = TempTree::new().unwrap();
        fixture
            .file("Library.photoslibrary/originals/photo.heic", 3_000)
            .unwrap();

        let tree = walk(
            &ReaddirWalker::default(),
            fixture.root(),
            &ScanPolicy::default(),
        );

        let (package, node) = child(&tree, FileTree::ROOT, "Library.photoslibrary");
        assert_eq!(node.kind(), EntryKind::Package);
        assert_eq!(node.logical_size(), ByteSize::from_bytes(3_000));
        assert_eq!(tree.children(package).count(), 0);
    }

    #[test]
    fn does_not_enter_pruned_folders() {
        let fixture = TempTree::new().unwrap();
        fixture.file("keep/a.bin", 100).unwrap();
        fixture.file("skip/b.bin", 900).unwrap();
        let policy = ScanPolicy::pruning([fixture.path("skip")]);

        let tree = walk(&ReaddirWalker::default(), fixture.root(), &policy);

        assert_eq!(
            tree.root().unwrap().logical_size(),
            ByteSize::from_bytes(100)
        );
        let (skipped, _) = child(&tree, FileTree::ROOT, "skip");
        assert_eq!(tree.children(skipped).count(), 0);
    }

    #[test]
    fn marks_items_under_a_sync_root_as_synced() {
        let fixture = TempTree::new().unwrap();
        fixture.file("cloud/doc.txt", 10).unwrap();
        fixture.file("local/doc.txt", 10).unwrap();
        let walker = ReaddirWalker::new(vec![fixture.path("cloud")]);

        let tree = walk(&walker, fixture.root(), &ScanPolicy::default());

        let (cloud, _) = child(&tree, FileTree::ROOT, "cloud");
        let (local, _) = child(&tree, FileTree::ROOT, "local");
        assert_eq!(
            child(&tree, cloud, "doc.txt").1.cloud_state(),
            CloudState::Synced
        );
        assert_eq!(
            child(&tree, local, "doc.txt").1.cloud_state(),
            CloudState::Local
        );
    }

    #[test]
    fn a_cancelled_walk_reads_nothing() {
        let fixture = TempTree::new().unwrap();
        fixture.file("a.bin", 100).unwrap();
        let cancellation = CancellationToken::new();
        cancellation.cancel();

        let outcome = ReaddirWalker::default()
            .walk(
                fixture.root(),
                &ScanPolicy::default(),
                &NoProgress,
                &cancellation,
            )
            .unwrap();

        assert_eq!(outcome.tree.len(), 1);
    }

    #[test]
    fn rejects_a_missing_root_and_a_file_as_root() {
        let fixture = TempTree::new().unwrap();
        let file = fixture.file("a.bin", 1).unwrap();
        let walker = ReaddirWalker::default();
        let token = CancellationToken::new();

        let missing = walker.walk(
            &fixture.path("nope"),
            &ScanPolicy::default(),
            &NoProgress,
            &token,
        );
        assert!(matches!(missing, Err(ApplicationError::RootNotFound(_))));
        let not_a_folder = walker.walk(&file, &ScanPolicy::default(), &NoProgress, &token);
        assert!(matches!(
            not_a_folder,
            Err(ApplicationError::RootNotDirectory(_))
        ));
    }
}

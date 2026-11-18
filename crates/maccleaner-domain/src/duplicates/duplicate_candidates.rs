//! First step of finding duplicates: files that have the same size.

use std::collections::BTreeMap;
use std::path::Path;

use crate::scan::{FileTree, LocalFile, local_files};
use crate::shared::ByteSize;

/// Groups the files of `tree` by content size and keeps the groups with at
/// least two files, largest size first.
///
/// Files smaller than `minimum` are left out, and so is everything
/// [`local_files`] leaves out. `root_path` is the absolute path the tree was
/// scanned from.
#[must_use]
pub fn candidates_by_size(
    tree: &FileTree,
    root_path: &Path,
    minimum: ByteSize,
) -> Vec<(ByteSize, Vec<LocalFile>)> {
    let mut by_size: BTreeMap<ByteSize, Vec<LocalFile>> = BTreeMap::new();
    for file in local_files(tree, root_path) {
        if file.logical_size >= minimum {
            by_size.entry(file.logical_size).or_default().push(file);
        }
    }
    by_size
        .into_iter()
        .rev()
        .filter(|(_, files)| files.len() > 1)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::candidates_by_size;
    use crate::scan::{CloudState, EntryKind, FileEntry, FileTree};
    use crate::shared::ByteSize;

    fn file(name: &str, bytes: u64) -> FileEntry {
        FileEntry::file(name, ByteSize::from_bytes(bytes))
    }

    #[test]
    fn groups_files_of_equal_size_across_folders_largest_first() {
        let mut tree = FileTree::new(FileEntry::directory("/scan"));
        tree.add_child(FileTree::ROOT, file("a.mov", 900));
        tree.add_child(FileTree::ROOT, file("unique.txt", 70));
        let sub = tree
            .add_child(FileTree::ROOT, FileEntry::directory("copies"))
            .unwrap();
        tree.add_child(sub, file("b.mov", 900));
        tree.add_child(sub, file("x.pdf", 300));
        tree.add_child(FileTree::ROOT, file("y.pdf", 300));

        let groups = candidates_by_size(&tree, Path::new("/scan"), ByteSize::from_bytes(1));

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].0, ByteSize::from_bytes(900));
        let paths: Vec<&PathBuf> = groups[0].1.iter().map(|c| &c.path).collect();
        assert!(paths.contains(&&PathBuf::from("/scan/a.mov")));
        assert!(paths.contains(&&PathBuf::from("/scan/copies/b.mov")));
        assert_eq!(groups[1].0, ByteSize::from_bytes(300));
    }

    #[test]
    fn leaves_out_small_empty_and_cloud_only_files() {
        let mut tree = FileTree::new(FileEntry::directory("/scan"));
        tree.add_child(FileTree::ROOT, file("small-1", 10));
        tree.add_child(FileTree::ROOT, file("small-2", 10));
        tree.add_child(FileTree::ROOT, file("empty-1", 0));
        tree.add_child(FileTree::ROOT, file("empty-2", 0));
        let mut cloud = file("cloud.mov", 500);
        cloud.cloud_state = CloudState::CloudOnly;
        tree.add_child(FileTree::ROOT, cloud);
        tree.add_child(FileTree::ROOT, file("local.mov", 500));

        let groups = candidates_by_size(&tree, Path::new("/scan"), ByteSize::from_bytes(100));

        assert!(groups.is_empty());
    }

    #[test]
    fn never_looks_inside_packages_or_follows_links() {
        let mut tree = FileTree::new(FileEntry::directory("/scan"));
        let mut package = FileEntry::directory("Lib.photoslibrary");
        package.kind = EntryKind::Package;
        let package = tree.add_child(FileTree::ROOT, package).unwrap();
        tree.add_child(package, file("inside.heic", 400));
        let mut link = file("link.heic", 400);
        link.kind = EntryKind::Symlink;
        tree.add_child(FileTree::ROOT, link);
        tree.add_child(FileTree::ROOT, file("loose.heic", 400));

        let groups = candidates_by_size(&tree, Path::new("/scan"), ByteSize::from_bytes(1));

        assert!(groups.is_empty());
    }
}

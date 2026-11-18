//! The regular files of a scanned tree whose content is safe to read.

use std::path::{Path, PathBuf};

use crate::scan::{EntryKind, FileTree, NodeId};
use crate::shared::ByteSize;

/// A regular file whose content is on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalFile {
    /// Absolute path of the file.
    pub path: PathBuf,
    /// Size of the content.
    pub logical_size: ByteSize,
    /// Disk space the file uses.
    pub allocated: ByteSize,
    /// Last modification, in seconds since the Unix epoch, when known.
    pub modified_unix_seconds: Option<i64>,
}

/// Lists the regular files of `tree` that can be read without side effects.
///
/// Empty files are left out, and so are cloud-only files: reading their
/// content would make macOS download them. Links are not followed and
/// packages are not entered. `root_path` is the absolute path the tree was
/// scanned from.
#[must_use]
pub fn local_files(tree: &FileTree, root_path: &Path) -> Vec<LocalFile> {
    let mut files = Vec::new();
    let mut pending = vec![FileTree::ROOT];
    while let Some(folder) = pending.pop() {
        for id in tree.children(folder) {
            let Some(node) = tree.node(id) else { continue };
            match node.kind() {
                EntryKind::Directory => pending.push(id),
                EntryKind::File
                    if !node.logical_size().is_zero() && !node.cloud_state().is_cloud_only() =>
                {
                    files.push(LocalFile {
                        path: path_of(tree, id, root_path),
                        logical_size: node.logical_size(),
                        allocated: node.allocated_size(),
                        modified_unix_seconds: node.modified_unix_seconds(),
                    });
                }
                EntryKind::File | EntryKind::Symlink | EntryKind::Package | EntryKind::Other => {}
            }
        }
    }
    files
}

fn path_of(tree: &FileTree, id: NodeId, root_path: &Path) -> PathBuf {
    let mut path = root_path.to_path_buf();
    path.extend(tree.path_segments(id).into_iter().skip(1));
    path
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::local_files;
    use crate::scan::{CloudState, EntryKind, FileEntry, FileTree};
    use crate::shared::ByteSize;

    fn file(name: &str, bytes: u64) -> FileEntry {
        FileEntry::file(name, ByteSize::from_bytes(bytes))
    }

    #[test]
    fn lists_files_of_every_folder_with_their_full_path() {
        let mut tree = FileTree::new(FileEntry::directory("/scan"));
        tree.add_child(FileTree::ROOT, file("a.mov", 900));
        let sub = tree
            .add_child(FileTree::ROOT, FileEntry::directory("copies"))
            .unwrap();
        tree.add_child(sub, file("b.mov", 700));

        let mut paths: Vec<PathBuf> = local_files(&tree, Path::new("/scan"))
            .into_iter()
            .map(|file| file.path)
            .collect();
        paths.sort();

        assert_eq!(
            paths,
            [
                PathBuf::from("/scan/a.mov"),
                PathBuf::from("/scan/copies/b.mov")
            ]
        );
    }

    #[test]
    fn leaves_out_what_cannot_be_read_safely() {
        let mut tree = FileTree::new(FileEntry::directory("/scan"));
        tree.add_child(FileTree::ROOT, file("empty", 0));
        let mut cloud = file("cloud.mov", 500);
        cloud.cloud_state = CloudState::CloudOnly;
        tree.add_child(FileTree::ROOT, cloud);
        let mut package = FileEntry::directory("Lib.photoslibrary");
        package.kind = EntryKind::Package;
        let package = tree.add_child(FileTree::ROOT, package).unwrap();
        tree.add_child(package, file("inside.heic", 400));
        let mut link = file("link.heic", 400);
        link.kind = EntryKind::Symlink;
        tree.add_child(FileTree::ROOT, link);

        assert!(local_files(&tree, Path::new("/scan")).is_empty());
    }
}

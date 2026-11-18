//! Permanent removal that stays inside one folder tree and one device.

use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt as _;
use std::path::Path;

use diskmarshal_application::ApplicationError;
use diskmarshal_application::ports::FileRemover;

/// Removes files and folder trees for good.
///
/// A symbolic link is removed as a link; what it points to is never touched.
/// Folder trees are removed with `std::fs::remove_dir_all`, which walks them
/// with `openat`/`unlinkat` and so cannot be tricked into leaving the tree by
/// a link swapped in during the removal. A tree that contains a folder on
/// another device (a mounted disk) is refused as a whole.
#[derive(Debug, Clone, Copy, Default)]
pub struct SafeFileRemover;

impl FileRemover for SafeFileRemover {
    fn remove(&self, path: &Path) -> Result<(), ApplicationError> {
        remove(path).map_err(|source| ApplicationError::Io {
            path: path.to_path_buf(),
            source,
        })
    }
}

fn remove(path: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_dir() {
        ensure_single_device(path, metadata.dev())?;
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

/// Fails when any folder under `directory` lives on a device other than `device`.
fn ensure_single_device(directory: &Path, device: u64) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        // `DirEntry::metadata` does not follow symbolic links.
        let metadata = entry.metadata()?;
        if !metadata.is_dir() {
            continue;
        }
        if metadata.dev() != device {
            return Err(io::Error::new(
                io::ErrorKind::CrossesDevices,
                format!("{} is on another device", entry.path().display()),
            ));
        }
        ensure_single_device(&entry.path(), device)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use diskmarshal_application::ports::FileRemover;
    use diskmarshal_test_support::TempTree;

    use super::SafeFileRemover;

    #[test]
    fn removes_a_folder_and_everything_beneath_it() {
        let fixture = TempTree::new().unwrap();
        fixture.file("cache/a/b/data.bin", 100).unwrap();
        fixture.file("keep.txt", 1).unwrap();

        SafeFileRemover.remove(&fixture.path("cache")).unwrap();

        assert!(!fixture.path("cache").exists());
        assert!(fixture.path("keep.txt").exists());
    }

    #[test]
    fn a_link_inside_the_tree_never_reaches_what_it_points_to() {
        let outside = TempTree::new().unwrap();
        let sentinel = outside.file("precious/thesis.txt", 10).unwrap();
        let fixture = TempTree::new().unwrap();
        fixture
            .symlink(&outside.path("precious"), "cache/shortcut")
            .unwrap();
        fixture.symlink(&sentinel, "cache/alias.txt").unwrap();

        SafeFileRemover.remove(&fixture.path("cache")).unwrap();

        assert!(!fixture.path("cache").exists());
        assert!(sentinel.exists());
    }

    #[test]
    fn removing_a_link_removes_only_the_link() {
        let outside = TempTree::new().unwrap();
        let sentinel = outside.file("precious/thesis.txt", 10).unwrap();
        let fixture = TempTree::new().unwrap();
        let link = fixture
            .symlink(&outside.path("precious"), "shortcut")
            .unwrap();

        SafeFileRemover.remove(&link).unwrap();

        assert!(link.symlink_metadata().is_err());
        assert!(sentinel.exists());
    }

    #[test]
    fn reports_a_missing_item_as_a_failure() {
        let fixture = TempTree::new().unwrap();
        assert!(SafeFileRemover.remove(&fixture.path("gone")).is_err());
    }
}

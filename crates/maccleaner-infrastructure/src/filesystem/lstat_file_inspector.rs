//! Inspector built on `lstat` and `realpath`.

use std::fs;
use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};

use maccleaner_application::ports::FileInspector;
use maccleaner_domain::deletion::ItemFingerprint;

/// Reads an item's identity with `lstat`, so a symbolic link is described as
/// the link itself and never as whatever it points to.
#[derive(Debug, Clone, Copy, Default)]
pub struct LstatFileInspector;

impl FileInspector for LstatFileInspector {
    fn resolve(&self, path: &Path) -> Option<PathBuf> {
        if !path.is_absolute() {
            // Left as it is, so the deletion policy rejects it as relative.
            return Some(path.to_path_buf());
        }
        match (path.parent(), path.file_name()) {
            (Some(parent), Some(name)) => Some(fs::canonicalize(parent).ok()?.join(name)),
            _ => fs::canonicalize(path).ok(),
        }
    }

    fn fingerprint(&self, path: &Path) -> Option<ItemFingerprint> {
        let metadata = fs::symlink_metadata(path).ok()?;
        Some(ItemFingerprint {
            device: metadata.dev(),
            inode: metadata.ino(),
            size: metadata.size(),
            modified_unix_seconds: metadata.mtime(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use maccleaner_application::ports::FileInspector;
    use maccleaner_test_support::TempTree;

    use super::LstatFileInspector;

    #[test]
    fn resolves_links_in_parent_folders_but_keeps_the_last_name() {
        let fixture = TempTree::new().unwrap();
        let real = fs::canonicalize(fixture.directory("real").unwrap()).unwrap();
        fixture.file("real/data.bin", 1).unwrap();
        fixture.symlink(&real, "shortcut").unwrap();
        fixture
            .symlink(&real.join("data.bin"), "real/alias.bin")
            .unwrap();

        let through_link = LstatFileInspector.resolve(&fixture.path("shortcut/data.bin"));
        assert_eq!(through_link, Some(real.join("data.bin")));
        let final_link = LstatFileInspector.resolve(&fixture.path("real/alias.bin"));
        assert_eq!(final_link, Some(real.join("alias.bin")));
    }

    #[test]
    fn a_missing_parent_cannot_be_resolved() {
        let fixture = TempTree::new().unwrap();
        assert_eq!(LstatFileInspector.resolve(&fixture.path("nope/file")), None);
    }

    #[test]
    fn the_fingerprint_changes_when_the_item_changes() {
        let fixture = TempTree::new().unwrap();
        let file = fixture.file("data.bin", 10).unwrap();
        let before = LstatFileInspector.fingerprint(&file).unwrap();

        fs::write(&file, vec![0_u8; 25]).unwrap();

        let after = LstatFileInspector.fingerprint(&file).unwrap();
        assert_eq!(before.inode, after.inode);
        assert_ne!(before, after);
        assert_eq!(LstatFileInspector.fingerprint(&fixture.path("gone")), None);
    }
}

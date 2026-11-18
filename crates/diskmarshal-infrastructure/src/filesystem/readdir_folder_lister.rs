//! Folder lister built on `readdir`.

use std::fs;
use std::os::unix::fs::MetadataExt as _;
use std::path::Path;

use diskmarshal_application::ports::{FolderItem, FolderLister};
use diskmarshal_domain::shared::ByteSize;

use crate::filesystem::STAT_BLOCK_BYTES;

/// Lists a folder with `readdir`, describing each item with `lstat`, so a
/// symbolic link is listed as the link itself and nothing is entered.
#[derive(Debug, Clone, Copy, Default)]
pub struct ReaddirFolderLister;

impl FolderLister for ReaddirFolderLister {
    fn items(&self, folder: &Path) -> Vec<FolderItem> {
        let Ok(entries) = fs::read_dir(folder) else {
            return Vec::new();
        };
        let mut items: Vec<FolderItem> = entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                // `DirEntry::metadata` does not follow a symbolic link.
                let metadata = entry.metadata().ok()?;
                Some(FolderItem {
                    // A name that is not valid text cannot be the name of an
                    // app or an identifier, so nothing is lost by leaving it out.
                    name: entry.file_name().into_string().ok()?,
                    is_folder: metadata.is_dir(),
                    allocated_size: ByteSize::from_bytes(
                        metadata.blocks().saturating_mul(STAT_BLOCK_BYTES),
                    ),
                })
            })
            .collect();
        items.sort_by(|a, b| a.name.cmp(&b.name));
        items
    }
}

#[cfg(test)]
mod tests {
    use diskmarshal_application::ports::FolderLister;
    use diskmarshal_test_support::TempTree;

    use super::ReaddirFolderLister;

    #[test]
    fn lists_only_what_is_directly_inside_and_never_follows_a_link() {
        let fixture = TempTree::new().unwrap();
        fixture.file("library/settings.plist", 5_000).unwrap();
        fixture.file("library/data/deep/file.bin", 10).unwrap();
        let outside = fixture.directory("outside").unwrap();
        fixture.symlink(&outside, "library/shortcut").unwrap();

        let items = ReaddirFolderLister.items(&fixture.path("library"));

        let seen: Vec<(&str, bool)> = items
            .iter()
            .map(|item| (item.name.as_str(), item.is_folder))
            .collect();
        assert_eq!(
            seen,
            [
                ("data", true),
                ("settings.plist", false),
                ("shortcut", false)
            ]
        );
        assert!(items[1].allocated_size.as_bytes() >= 5_000);
    }

    #[test]
    fn a_folder_that_does_not_exist_has_no_items() {
        let fixture = TempTree::new().unwrap();
        assert!(ReaddirFolderLister.items(&fixture.path("gone")).is_empty());
    }
}

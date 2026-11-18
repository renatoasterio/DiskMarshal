//! Which files are photos worth comparing.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::scan::{FileTree, LocalFile, local_files};
use crate::shared::ByteSize;

/// Extensions of the image formats macOS can open, in lowercase.
const STANDARD_EXTENSIONS: [&str; 10] = [
    "jpg", "jpeg", "png", "heic", "heif", "tif", "tiff", "bmp", "gif", "webp",
];
/// Extensions of camera raw formats, in lowercase.
const RAW_EXTENSIONS: [&str; 10] = [
    "dng", "cr2", "cr3", "nef", "arw", "raf", "orf", "rw2", "srw", "pef",
];

/// Images smaller than this are icons and interface pieces, not photos.
pub const MINIMUM_PHOTO_SIZE: ByteSize = ByteSize::from_bytes(50_000);

fn extension(path: &Path) -> Option<String> {
    Some(path.extension()?.to_str()?.to_lowercase())
}

fn is_raw(path: &Path) -> bool {
    extension(path).is_some_and(|extension| RAW_EXTENSIONS.contains(&extension.as_str()))
}

/// Whether the file name is that of an image macOS can open.
#[must_use]
pub fn is_image_file(path: &Path) -> bool {
    extension(path).is_some_and(|extension| {
        STANDARD_EXTENSIONS.contains(&extension.as_str())
            || RAW_EXTENSIONS.contains(&extension.as_str())
    })
}

/// Lists the photos of `tree` worth comparing.
///
/// Besides everything [`local_files`] leaves out, images smaller than
/// [`MINIMUM_PHOTO_SIZE`] are skipped. A camera raw file saved next to a
/// developed copy of the same shot (`IMG_1.CR2` beside `IMG_1.JPG`) is
/// skipped too: the pair belongs together and is not a duplicate.
#[must_use]
pub fn photo_candidates(tree: &FileTree, root_path: &Path) -> Vec<LocalFile> {
    let images: Vec<LocalFile> = local_files(tree, root_path)
        .into_iter()
        .filter(|file| is_image_file(&file.path) && file.logical_size >= MINIMUM_PHOTO_SIZE)
        .collect();
    let developed: BTreeSet<(PathBuf, String)> = images
        .iter()
        .filter(|file| !is_raw(&file.path))
        .filter_map(|file| shot(&file.path))
        .collect();
    images
        .into_iter()
        .filter(|file| {
            !is_raw(&file.path) || shot(&file.path).is_none_or(|shot| !developed.contains(&shot))
        })
        .collect()
}

/// What identifies a shot: its folder and its file name without extension.
fn shot(path: &Path) -> Option<(PathBuf, String)> {
    let stem = path.file_stem()?.to_str()?.to_lowercase();
    Some((path.parent()?.to_path_buf(), stem))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{is_image_file, photo_candidates};
    use crate::scan::{FileEntry, FileTree};
    use crate::shared::ByteSize;

    fn tree(files: &[(&str, u64)]) -> FileTree {
        let mut tree = FileTree::new(FileEntry::directory("/photos"));
        for (name, bytes) in files {
            tree.add_child(
                FileTree::ROOT,
                FileEntry::file(name, ByteSize::from_bytes(*bytes)),
            );
        }
        tree
    }

    fn names(tree: &FileTree) -> Vec<String> {
        let mut names: Vec<String> = photo_candidates(tree, Path::new("/photos"))
            .into_iter()
            .filter_map(|file| Some(file.path.file_name()?.to_str()?.to_owned()))
            .collect();
        names.sort();
        names
    }

    #[test]
    fn recognizes_images_whatever_the_case_of_the_extension() {
        assert!(is_image_file(&PathBuf::from("/p/IMG_1.HEIC")));
        assert!(is_image_file(&PathBuf::from("/p/scan.tiff")));
        assert!(is_image_file(&PathBuf::from("/p/raw.CR3")));
        assert!(!is_image_file(&PathBuf::from("/p/clip.mov")));
        assert!(!is_image_file(&PathBuf::from("/p/jpg")));
    }

    #[test]
    fn keeps_photos_and_skips_small_images_and_other_files() {
        let tree = tree(&[
            ("a.jpg", 900_000),
            ("icon.png", 4_000),
            ("notes.txt", 900_000),
        ]);
        assert_eq!(names(&tree), ["a.jpg"]);
    }

    #[test]
    fn skips_the_raw_file_of_a_shot_that_has_a_developed_copy() {
        let tree = tree(&[
            ("IMG_1.CR2", 24_000_000),
            ("img_1.jpg", 6_000_000),
            ("IMG_2.CR2", 24_000_000),
        ]);
        assert_eq!(names(&tree), ["IMG_2.CR2", "img_1.jpg"]);
    }
}

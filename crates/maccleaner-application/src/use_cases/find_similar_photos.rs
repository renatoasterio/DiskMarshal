//! Use case: find photos that look the same.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use maccleaner_domain::deletion::DeletionRequest;
use maccleaner_domain::photos::{
    PhotoSignature, SimilarityGroup, SimilarityTolerance, Thumbnail, group_similar,
    photo_candidates,
};
use maccleaner_domain::scan::{LocalFile, ScanPolicy};
use maccleaner_domain::shared::ByteSize;

use crate::ApplicationError;
use crate::parallel::map_in_parallel;
use crate::ports::{FileInspector, FileSystemWalker, ImageThumbnailer};
use crate::progress::{CancellationToken, ProgressSink, ScanProgress};
use crate::use_cases::trash_requests::trash_requests;

/// Longer side of the pictures the comparison reads. Small on purpose: the
/// comparison looks at the overall picture, not at its detail.
const SIGNATURE_SIDE: u32 = 64;

/// A group of similar photos with the removal of each one.
#[derive(Debug, Clone, PartialEq)]
pub struct FoundSimilarPhotos {
    /// The photos, with the one to keep first.
    pub group: SimilarityGroup,
    /// One removal per photo, in the same order. Photos go to the Trash.
    pub requests: Vec<DeletionRequest>,
    /// One picture per photo, in the same order, for the groups that asked
    /// for them. `None` where the picture could not be made.
    pub previews: Vec<Option<Thumbnail>>,
}

/// What a search for similar photos found.
#[derive(Debug, Clone, PartialEq)]
pub struct SimilarPhotosReport {
    /// The folder that was searched, as resolved on disk.
    pub root: PathBuf,
    /// How many photos were compared.
    pub photos_compared: usize,
    /// The groups, the one that frees the most space first.
    pub groups: Vec<FoundSimilarPhotos>,
}

/// How a search compares photos and what it prepares for the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimilarPhotosOptions {
    /// How different two photos may be and still be gathered.
    pub tolerance: SimilarityTolerance,
    /// How many groups, from the first, get pictures to show.
    pub previewed_groups: usize,
    /// Longer side of those pictures, in pixels.
    pub preview_side: u32,
}

/// Searches a folder for photos that look the same.
pub struct FindSimilarPhotos<'a> {
    walker: &'a dyn FileSystemWalker,
    thumbnailer: &'a dyn ImageThumbnailer,
    inspector: &'a dyn FileInspector,
}

impl<'a> FindSimilarPhotos<'a> {
    /// Creates the use case over the adapters it reads through.
    #[must_use]
    pub fn new(
        walker: &'a dyn FileSystemWalker,
        thumbnailer: &'a dyn ImageThumbnailer,
        inspector: &'a dyn FileInspector,
    ) -> Self {
        Self {
            walker,
            thumbnailer,
            inspector,
        }
    }

    /// Searches `root` for image files that look the same.
    ///
    /// Only loose image files are read. Libraries of the Photos app are
    /// packages, which a scan never enters, and cloud-only files are never
    /// opened. An image that cannot be read is left out. During the
    /// comparison, progress counts the photos read so far.
    ///
    /// # Errors
    ///
    /// Fails when `root` cannot be scanned or the search is cancelled.
    pub fn execute(
        &self,
        root: &Path,
        options: SimilarPhotosOptions,
        progress: &dyn ProgressSink,
        cancellation: &CancellationToken,
    ) -> Result<SimilarPhotosReport, ApplicationError> {
        let root = self
            .inspector
            .resolve(root)
            .ok_or_else(|| ApplicationError::RootNotFound(root.to_path_buf()))?;
        let mut tree = self
            .walker
            .walk(&root, &ScanPolicy::default(), progress, cancellation)?
            .tree;
        tree.aggregate();

        let photos = self.signed(photo_candidates(&tree, &root), progress, cancellation)?;
        let photos_compared = photos.len();
        let groups = group_similar(&photos, options.tolerance)
            .into_iter()
            .filter_map(|group| {
                let files = group
                    .photos
                    .iter()
                    .map(|photo| (photo.path.as_path(), photo.allocated));
                let requests = trash_requests(self.inspector, files)?;
                Some(FoundSimilarPhotos {
                    group,
                    requests,
                    previews: Vec::new(),
                })
            })
            .collect();
        let groups = self.previewed(groups, options, cancellation)?;
        Ok(SimilarPhotosReport {
            root,
            photos_compared,
            groups,
        })
    }

    /// Reads every photo and describes its look.
    fn signed(
        &self,
        files: Vec<LocalFile>,
        progress: &dyn ProgressSink,
        cancellation: &CancellationToken,
    ) -> Result<Vec<(LocalFile, PhotoSignature)>, ApplicationError> {
        let (read, bytes) = (AtomicU64::new(0), AtomicU64::new(0));
        let signatures = map_in_parallel(&files, cancellation, |file| {
            let signature = self
                .thumbnailer
                .thumbnail(&file.path, SIGNATURE_SIDE)
                .ok()
                .map(|thumbnail| PhotoSignature::of(&thumbnail));
            let size = file.logical_size.as_bytes();
            progress.on_progress(ScanProgress {
                items_seen: read.fetch_add(1, Ordering::Relaxed) + 1,
                bytes_seen: ByteSize::from_bytes(bytes.fetch_add(size, Ordering::Relaxed) + size),
            });
            signature
        })?;
        Ok(files
            .into_iter()
            .zip(signatures)
            .filter_map(|(file, signature)| Some((file, signature?)))
            .collect())
    }

    /// Adds the pictures to show to the first groups.
    fn previewed(
        &self,
        mut groups: Vec<FoundSimilarPhotos>,
        options: SimilarPhotosOptions,
        cancellation: &CancellationToken,
    ) -> Result<Vec<FoundSimilarPhotos>, ApplicationError> {
        let paths: Vec<&Path> = groups
            .iter()
            .take(options.previewed_groups)
            .flat_map(|found| found.group.photos.iter().map(|photo| photo.path.as_path()))
            .collect();
        let mut pictures = map_in_parallel(&paths, cancellation, |path| {
            self.thumbnailer.thumbnail(path, options.preview_side).ok()
        })?
        .into_iter();
        for found in groups.iter_mut().take(options.previewed_groups) {
            found.previews = pictures.by_ref().take(found.group.photos.len()).collect();
        }
        Ok(groups)
    }
}

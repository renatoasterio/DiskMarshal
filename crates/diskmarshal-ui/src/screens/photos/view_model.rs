//! State of the photos screen: the search, the pictures to show and the
//! review of what was found.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use diskmarshal_application::ApplicationError;
use diskmarshal_application::progress::{
    CancellationToken, NoProgress, ProgressSink, ScanProgress,
};
use diskmarshal_application::use_cases::{
    FindSimilarPhotos, SimilarPhotosOptions, SimilarPhotosReport,
};
use diskmarshal_domain::photos::{SimilarityTolerance, Thumbnail};
use diskmarshal_domain::shared::ByteSize;
use egui::{ColorImage, Context, TextureHandle, TextureOptions};

use crate::app::UiServices;
use crate::screens::group_review::GroupReview;
use crate::tasks::Search;

/// Most groups drawn at once, the ones that free the most space first.
pub(crate) const MAX_GROUPS_SHOWN: usize = 60;
/// Longer side, in pixels, of the pictures shown. Twice the size they are
/// drawn at, so they stay sharp on a high-density display.
const PREVIEW_SIDE: u32 = 256;

/// Everything the photos screen remembers between frames.
#[derive(Default)]
pub(crate) struct PhotosViewModel {
    search: Search<SimilarPhotosReport>,
    report: Option<SimilarPhotosReport>,
    pictures: BTreeMap<PathBuf, TextureHandle>,
    pub review: GroupReview,
    pub tolerance: SimilarityTolerance,
}

impl PhotosViewModel {
    /// Searches `root` in the background.
    pub(crate) fn start(&mut self, ctx: &Context, services: &UiServices, root: PathBuf) {
        let (services, options) = (services.clone(), self.options());
        self.search.start(ctx, move |progress, cancellation| {
            search(&services, &root, options, progress, cancellation)
        });
    }

    /// Searches right now, blocking until it ends. Used to open the app with
    /// the screen already filled in.
    pub(crate) fn search_now(&mut self, ctx: &Context, services: &UiServices, root: &Path) {
        let outcome = search(
            services,
            root,
            self.options(),
            &NoProgress,
            &CancellationToken::new(),
        );
        if let Some(report) = self.search.settle(outcome) {
            self.accept(ctx, report);
        }
    }

    fn options(&self) -> SimilarPhotosOptions {
        SimilarPhotosOptions {
            tolerance: self.tolerance,
            previewed_groups: MAX_GROUPS_SHOWN,
            preview_side: PREVIEW_SIDE,
        }
    }

    pub(crate) fn cancel(&mut self) {
        self.search.cancel();
    }

    /// Picks up the result of a finished search.
    pub(crate) fn poll(&mut self, ctx: &Context) {
        if let Some(report) = self.search.poll() {
            self.accept(ctx, report);
        }
    }

    fn accept(&mut self, ctx: &Context, mut report: SimilarPhotosReport) {
        // The pixels move to the graphics card; the report keeps none.
        self.pictures = report
            .groups
            .iter_mut()
            .flat_map(|found| {
                let previews = std::mem::take(&mut found.previews);
                found
                    .group
                    .photos
                    .iter()
                    .map(|photo| photo.path.clone())
                    .zip(previews)
            })
            .filter_map(|(path, preview)| {
                let texture = texture_of(ctx, &path, &preview?);
                Some((path, texture))
            })
            .collect();
        let groups = report
            .groups
            .iter()
            .map(|found| found.requests.clone())
            .collect();
        self.review = GroupReview::new(groups);
        self.report = Some(report);
    }

    pub(crate) fn progress(&self) -> Option<ScanProgress> {
        self.search.progress()
    }

    pub(crate) fn report(&self) -> Option<&SimilarPhotosReport> {
        self.report.as_ref()
    }

    pub(crate) fn failure(&self) -> Option<&str> {
        self.search.failure()
    }

    /// The picture of a photo, when one could be made.
    pub(crate) fn picture(&self, path: &Path) -> Option<&TextureHandle> {
        self.pictures.get(path)
    }

    /// What removing every extra photo would free; `None` before the first search.
    pub(crate) fn recoverable(&self) -> Option<ByteSize> {
        self.report.as_ref().map(|_| self.review.recoverable())
    }
}

fn texture_of(ctx: &Context, path: &Path, preview: &Thumbnail) -> TextureHandle {
    let size = [preview.width() as usize, preview.height() as usize];
    let image = ColorImage::from_rgba_premultiplied(size, preview.rgba());
    ctx.load_texture(path.display().to_string(), image, TextureOptions::LINEAR)
}

fn search(
    services: &UiServices,
    root: &Path,
    options: SimilarPhotosOptions,
    progress: &dyn ProgressSink,
    cancellation: &CancellationToken,
) -> Result<SimilarPhotosReport, ApplicationError> {
    FindSimilarPhotos::new(
        services.walker.as_ref(),
        services.thumbnailer.as_ref(),
        services.inspector.as_ref(),
    )
    .execute(root, options, progress, cancellation)
}

//! State of the duplicates screen: the search and the review of what it found.

use std::path::{Path, PathBuf};

use diskmarshal_application::ApplicationError;
use diskmarshal_application::progress::{
    CancellationToken, NoProgress, ProgressSink, ScanProgress,
};
use diskmarshal_application::use_cases::{DuplicatesReport, FindDuplicates};
use diskmarshal_domain::shared::ByteSize;
use egui::Context;

use crate::app::UiServices;
use crate::screens::group_review::GroupReview;
use crate::tasks::Search;

/// Most groups drawn at once, largest first. Drawing thousands would make
/// the screen slow without helping anyone decide.
pub(crate) const MAX_GROUPS_SHOWN: usize = 100;

/// The smallest file worth comparing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum MinimumSize {
    #[default]
    OneMegabyte,
    TenMegabytes,
    HundredMegabytes,
}

impl MinimumSize {
    pub(crate) const ALL: [Self; 3] = [
        Self::OneMegabyte,
        Self::TenMegabytes,
        Self::HundredMegabytes,
    ];

    pub(crate) const fn bytes(self) -> ByteSize {
        ByteSize::from_bytes(match self {
            Self::OneMegabyte => 1_000_000,
            Self::TenMegabytes => 10_000_000,
            Self::HundredMegabytes => 100_000_000,
        })
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::OneMegabyte => "1 MB",
            Self::TenMegabytes => "10 MB",
            Self::HundredMegabytes => "100 MB",
        }
    }
}

/// Everything the duplicates screen remembers between frames.
#[derive(Default)]
pub(crate) struct DuplicatesViewModel {
    search: Search<DuplicatesReport>,
    report: Option<DuplicatesReport>,
    pub review: GroupReview,
    pub minimum: MinimumSize,
}

impl DuplicatesViewModel {
    /// Searches `root` in the background.
    pub(crate) fn start(&mut self, ctx: &Context, services: &UiServices, root: PathBuf) {
        let (services, minimum) = (services.clone(), self.minimum.bytes());
        self.search.start(ctx, move |progress, cancellation| {
            search(&services, &root, minimum, progress, cancellation)
        });
    }

    /// Searches right now, blocking until it ends. Used to open the app with
    /// the screen already filled in.
    pub(crate) fn search_now(&mut self, services: &UiServices, root: &Path) {
        let minimum = self.minimum.bytes();
        let outcome = search(
            services,
            root,
            minimum,
            &NoProgress,
            &CancellationToken::new(),
        );
        if let Some(report) = self.search.settle(outcome) {
            self.accept(report);
        }
    }

    pub(crate) fn cancel(&mut self) {
        self.search.cancel();
    }

    /// Picks up the result of a finished search.
    pub(crate) fn poll(&mut self) {
        if let Some(report) = self.search.poll() {
            self.accept(report);
        }
    }

    fn accept(&mut self, report: DuplicatesReport) {
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

    pub(crate) fn report(&self) -> Option<&DuplicatesReport> {
        self.report.as_ref()
    }

    pub(crate) fn failure(&self) -> Option<&str> {
        self.search.failure()
    }

    /// What removing every extra copy would free; `None` before the first search.
    pub(crate) fn recoverable(&self) -> Option<ByteSize> {
        self.report.as_ref().map(|_| self.review.recoverable())
    }
}

fn search(
    services: &UiServices,
    root: &Path,
    minimum: ByteSize,
    progress: &dyn ProgressSink,
    cancellation: &CancellationToken,
) -> Result<DuplicatesReport, ApplicationError> {
    FindDuplicates::new(
        services.walker.as_ref(),
        services.hasher.as_ref(),
        services.inspector.as_ref(),
    )
    .execute(root, minimum, progress, cancellation)
}

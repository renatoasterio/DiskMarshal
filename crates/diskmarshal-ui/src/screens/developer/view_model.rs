//! State and logic of the developer screen: the search, the filter, what is
//! selected, and the removal.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use diskmarshal_application::ApplicationError;
use diskmarshal_application::progress::{
    CancellationToken, NoProgress, ProgressSink, ScanProgress,
};
use diskmarshal_application::use_cases::{DevArtifactsReport, FindDevArtifacts, FoundArtifact};
use diskmarshal_domain::cleanup::RiskLevel;
use diskmarshal_domain::dev_artifacts::Ecosystem;
use diskmarshal_domain::shared::ByteSize;
use egui::Context;

use crate::app::UiServices;
use crate::components::RemovalFlow;
use crate::i18n::pt_br;
use crate::tasks::Search;

/// A project untouched for this long has its safe folders selected by default.
const IDLE_SECONDS: i64 = 30 * 86_400;

/// Everything the developer screen remembers between frames.
pub(crate) struct DeveloperViewModel {
    search: Search<DevArtifactsReport>,
    report: Option<DevArtifactsReport>,
    selected: BTreeSet<PathBuf>,
    notice: Option<&'static str>,
    /// Shows only this ecosystem; `None` shows all.
    pub filter: Option<Ecosystem>,
    pub removal: RemovalFlow,
}

impl Default for DeveloperViewModel {
    fn default() -> Self {
        Self {
            search: Search::default(),
            report: None,
            selected: BTreeSet::new(),
            notice: None,
            filter: None,
            removal: RemovalFlow::Closed,
        }
    }
}

impl DeveloperViewModel {
    /// Searches `root` in the background.
    pub(crate) fn start(&mut self, ctx: &Context, services: &UiServices, root: PathBuf) {
        let services = services.clone();
        self.search.start(ctx, move |progress, cancellation| {
            search(&services, &root, progress, cancellation)
        });
    }

    /// Searches right now, blocking until it ends. Used to open the app with
    /// the screen already filled in.
    pub(crate) fn search_now(&mut self, services: &UiServices, root: &Path) {
        let outcome = search(services, root, &NoProgress, &CancellationToken::new());
        if let Some(report) = self.search.settle(outcome) {
            self.accept(report, services.wall_clock.unix_seconds());
        }
    }

    pub(crate) fn cancel(&mut self) {
        self.search.cancel();
    }

    /// Picks up the result of a finished search.
    pub(crate) fn poll(&mut self, services: &UiServices) {
        if let Some(report) = self.search.poll() {
            self.accept(report, services.wall_clock.unix_seconds());
        }
    }

    fn accept(&mut self, report: DevArtifactsReport, now_unix_seconds: i64) {
        self.selected = report
            .found
            .iter()
            .filter(|item| is_default_choice(item, now_unix_seconds))
            .map(|item| item.artifact.path.clone())
            .collect();
        self.filter = None;
        self.report = Some(report);
    }

    pub(crate) fn progress(&self) -> Option<ScanProgress> {
        self.search.progress()
    }

    pub(crate) fn report(&self) -> Option<&DevArtifactsReport> {
        self.report.as_ref()
    }

    pub(crate) fn failure(&self) -> Option<&str> {
        self.search.failure()
    }

    pub(crate) fn notice(&self) -> Option<&'static str> {
        self.notice
    }

    /// The ecosystems that found something, for the filter.
    pub(crate) fn ecosystems(&self) -> Vec<Ecosystem> {
        let present: BTreeSet<Ecosystem> = self
            .report
            .iter()
            .flat_map(|report| &report.found)
            .map(|item| item.artifact.ecosystem)
            .collect();
        present.into_iter().collect()
    }

    /// What the list shows under the current filter.
    pub(crate) fn visible(&self) -> Vec<&FoundArtifact> {
        self.report
            .iter()
            .flat_map(|report| &report.found)
            .filter(|item| {
                self.filter
                    .is_none_or(|filter| item.artifact.ecosystem == filter)
            })
            .collect()
    }

    pub(crate) fn is_selected(&self, path: &Path) -> bool {
        self.selected.contains(path)
    }

    pub(crate) fn toggle(&mut self, path: &Path) {
        self.notice = None;
        if !self.selected.remove(path) {
            self.selected.insert(path.to_path_buf());
        }
    }

    fn chosen(&self) -> Vec<&FoundArtifact> {
        self.report
            .iter()
            .flat_map(|report| &report.found)
            .filter(|item| self.is_selected(&item.artifact.path))
            .collect()
    }

    /// How many folders are selected and how much space they use.
    pub(crate) fn selection(&self) -> (usize, ByteSize) {
        let chosen = self.chosen();
        (
            chosen.len(),
            chosen.iter().map(|item| item.artifact.size).sum(),
        )
    }

    /// What the search found in total; `None` before the first search.
    pub(crate) fn recoverable(&self) -> Option<ByteSize> {
        let report = self.report.as_ref()?;
        Some(report.found.iter().map(|item| item.artifact.size).sum())
    }

    /// The largest folder found, which fills the size bar completely.
    pub(crate) fn largest(&self) -> ByteSize {
        self.report
            .iter()
            .flat_map(|report| &report.found)
            .map(|item| item.artifact.size)
            .max()
            .unwrap_or(ByteSize::ZERO)
    }

    /// Opens the confirmation, or explains why there is nothing to confirm.
    pub(crate) fn request_removal(&mut self) {
        if self.selection().0 == 0 {
            self.notice = Some(pt_br::SELECT_SOMETHING);
        } else {
            self.removal.open();
        }
    }

    /// Removes the selected folders, in the background.
    pub(crate) fn confirm(&mut self, ctx: &Context, services: &UiServices) {
        let Some(report) = &self.report else { return };
        let requests = self
            .chosen()
            .into_iter()
            .map(|item| item.request.clone())
            .collect();
        let roots = vec![report.root.clone()];
        self.removal.start(ctx, services, requests, roots);
    }
}

/// Safe folders of projects nobody has touched for a while are offered ready
/// to remove. Everything else waits for the user to pick it.
fn is_default_choice(item: &FoundArtifact, now_unix_seconds: i64) -> bool {
    item.artifact.risk == RiskLevel::Safe
        && item
            .artifact
            .project_modified_unix_seconds
            .is_some_and(|modified| now_unix_seconds.saturating_sub(modified) >= IDLE_SECONDS)
}

fn search(
    services: &UiServices,
    root: &Path,
    progress: &dyn ProgressSink,
    cancellation: &CancellationToken,
) -> Result<DevArtifactsReport, ApplicationError> {
    FindDevArtifacts::new(
        services.rules.as_ref(),
        services.walker.as_ref(),
        services.inspector.as_ref(),
        services.version_control.as_ref(),
    )
    .execute(root, progress, cancellation)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use diskmarshal_application::use_cases::{DevArtifactsReport, FoundArtifact};
    use diskmarshal_domain::cleanup::RiskLevel;
    use diskmarshal_domain::deletion::{DeletionMethod, DeletionRequest, ItemFingerprint};
    use diskmarshal_domain::dev_artifacts::{Ecosystem, ProjectArtifact};
    use diskmarshal_domain::shared::ByteSize;

    use super::{DeveloperViewModel, IDLE_SECONDS};
    use crate::components::RemovalFlow;

    const NOW: i64 = 2_000_000_000;

    fn item(
        folder: &str,
        ecosystem: Ecosystem,
        risk: RiskLevel,
        idle: i64,
        size: u64,
    ) -> FoundArtifact {
        let path = PathBuf::from(format!("/work/app/{folder}"));
        FoundArtifact {
            artifact: ProjectArtifact {
                project: "app".into(),
                ecosystem,
                folder: folder.into(),
                path: path.clone(),
                size: ByteSize::from_bytes(size),
                project_modified_unix_seconds: Some(NOW - idle),
                risk,
            },
            request: DeletionRequest {
                target: path,
                method: DeletionMethod::Permanent,
                fingerprint: ItemFingerprint {
                    device: 1,
                    inode: size,
                    size,
                    modified_unix_seconds: 0,
                },
                reclaimable: ByteSize::from_bytes(size),
            },
        }
    }

    fn model() -> DeveloperViewModel {
        let mut model = DeveloperViewModel::default();
        let found = vec![
            item(
                "node_modules",
                Ecosystem::Node,
                RiskLevel::Safe,
                IDLE_SECONDS + 5,
                900,
            ),
            item(
                "dist",
                Ecosystem::Node,
                RiskLevel::Review,
                IDLE_SECONDS + 5,
                300,
            ),
            item("target", Ecosystem::Rust, RiskLevel::Safe, 3_600, 700),
        ];
        model.accept(
            DevArtifactsReport {
                root: PathBuf::from("/work"),
                found,
            },
            NOW,
        );
        model
    }

    #[test]
    fn selects_by_default_only_safe_folders_of_idle_projects() {
        let model = model();
        assert!(model.is_selected(Path::new("/work/app/node_modules")));
        assert!(
            !model.is_selected(Path::new("/work/app/dist")),
            "review is never preselected"
        );
        assert!(
            !model.is_selected(Path::new("/work/app/target")),
            "the project was just touched"
        );
        assert_eq!(model.selection(), (1, ByteSize::from_bytes(900)));
    }

    #[test]
    fn the_filter_narrows_the_list_but_not_the_selection() {
        let mut model = model();
        assert_eq!(model.ecosystems(), [Ecosystem::Node, Ecosystem::Rust]);
        model.filter = Some(Ecosystem::Rust);
        assert_eq!(model.visible().len(), 1);
        assert_eq!(model.selection().0, 1);
        assert_eq!(model.largest(), ByteSize::from_bytes(900));
    }

    #[test]
    fn asking_to_remove_with_nothing_selected_explains_instead_of_confirming() {
        let mut model = model();
        model.toggle(Path::new("/work/app/node_modules"));
        model.request_removal();
        assert!(matches!(model.removal, RemovalFlow::Closed));
        assert!(model.notice().is_some());

        model.toggle(Path::new("/work/app/target"));
        model.request_removal();
        assert!(matches!(model.removal, RemovalFlow::Confirm));
    }
}

//! State and logic of the Docker screen: what Docker answered, the filter,
//! what is selected, and the removal.

use std::collections::BTreeSet;

use egui::Context;
use maccleaner_application::ports::EngineError;
use maccleaner_application::use_cases::{InspectDocker, PruneDocker};
use maccleaner_domain::cleanup::RiskLevel;
use maccleaner_domain::docker::{DockerInventory, DockerResource, ResourceKind};
use maccleaner_domain::shared::ByteSize;

use crate::app::UiServices;
use crate::components::{RemovalFlow, RemovalResult};
use crate::i18n::pt_br;
use crate::tasks::Search;

/// What identifies a resource in the selection: its kind and what Docker
/// removes it by.
type ResourceKey = (ResourceKind, String);

/// What asking Docker led to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DockerState {
    /// No Docker on this Mac.
    NotInstalled,
    /// Docker is installed but closed.
    NotRunning,
    /// Docker answered something this app could not use; the text says what.
    Failed(String),
    /// What Docker holds.
    Ready(DockerInventory),
}

/// What the confirmation dialog tells the user.
pub(crate) struct Selection {
    pub items: usize,
    /// The sizes Docker reports, added up. Images share layers, so the
    /// space really freed can be less.
    pub size: ByteSize,
    /// Whether a container or a volume is selected: their content exists
    /// nowhere else.
    pub has_data: bool,
}

/// Everything the Docker screen remembers between frames.
pub(crate) struct DockerViewModel {
    inspection: Search<DockerState>,
    state: Option<DockerState>,
    has_started: bool,
    selected: BTreeSet<ResourceKey>,
    notice: Option<&'static str>,
    /// Shows only this kind; `None` shows all.
    pub filter: Option<ResourceKind>,
    pub removal: RemovalFlow,
}

impl Default for DockerViewModel {
    fn default() -> Self {
        Self {
            inspection: Search::default(),
            state: None,
            has_started: false,
            selected: BTreeSet::new(),
            notice: None,
            filter: None,
            removal: RemovalFlow::Closed,
        }
    }
}

impl DockerViewModel {
    /// Asks Docker the first time the screen is shown.
    pub(crate) fn ensure_started(&mut self, ctx: &Context, services: &UiServices) {
        if !self.has_started {
            self.inspect(ctx, services);
        }
    }

    /// Asks Docker again what it holds, in the background.
    pub(crate) fn inspect(&mut self, ctx: &Context, services: &UiServices) {
        self.has_started = true;
        let services = services.clone();
        self.inspection
            .start(ctx, move |_, _| Ok(inspect(&services)));
    }

    /// Asks Docker right now, blocking until it answers. Used to open the
    /// app with the screen already filled in.
    pub(crate) fn inspect_now(&mut self, services: &UiServices) {
        self.has_started = true;
        self.accept(inspect(services));
    }

    /// Picks up the answer of Docker when it arrives.
    pub(crate) fn poll(&mut self) {
        if let Some(state) = self.inspection.poll() {
            self.accept(state);
        }
    }

    fn accept(&mut self, state: DockerState) {
        self.selected = match &state {
            DockerState::Ready(inventory) => inventory
                .unused
                .iter()
                .filter(|resource| resource.risk() == RiskLevel::Safe)
                .map(key)
                .collect(),
            _ => BTreeSet::new(),
        };
        self.filter = None;
        self.notice = None;
        self.state = Some(state);
    }

    /// Whether Docker is being asked right now.
    pub(crate) fn is_inspecting(&self) -> bool {
        self.inspection.progress().is_some()
    }

    /// What asking Docker led to; `None` before the first answer, or when
    /// the question was lost.
    pub(crate) fn state(&self) -> Option<&DockerState> {
        self.state.as_ref()
    }

    pub(crate) fn notice(&self) -> Option<&'static str> {
        self.notice
    }

    /// Why the last question to Docker was lost, when it was.
    pub(crate) fn failure(&self) -> Option<&str> {
        self.inspection.failure()
    }

    fn unused(&self) -> &[DockerResource] {
        match &self.state {
            Some(DockerState::Ready(inventory)) => &inventory.unused,
            _ => &[],
        }
    }

    /// The kinds that have something unused, for the filter.
    pub(crate) fn kinds(&self) -> Vec<ResourceKind> {
        ResourceKind::ALL
            .into_iter()
            .filter(|kind| self.unused().iter().any(|resource| resource.kind == *kind))
            .collect()
    }

    /// What the list shows under the current filter.
    pub(crate) fn visible(&self) -> Vec<&DockerResource> {
        self.unused()
            .iter()
            .filter(|resource| self.filter.is_none_or(|filter| resource.kind == filter))
            .collect()
    }

    pub(crate) fn is_selected(&self, resource: &DockerResource) -> bool {
        self.selected.contains(&key(resource))
    }

    pub(crate) fn toggle(&mut self, resource: &DockerResource) {
        self.notice = None;
        let key = key(resource);
        if !self.selected.remove(&key) {
            self.selected.insert(key);
        }
    }

    fn picked(&self) -> Vec<&DockerResource> {
        self.unused()
            .iter()
            .filter(|resource| self.is_selected(resource))
            .collect()
    }

    /// What is selected right now.
    pub(crate) fn selection(&self) -> Selection {
        let picked = self.picked();
        Selection {
            items: picked.len(),
            size: picked.iter().map(|resource| resource.size).sum(),
            has_data: picked.iter().any(|resource| {
                matches!(
                    resource.kind,
                    ResourceKind::Container | ResourceKind::Volume
                )
            }),
        }
    }

    /// What Docker itself says removing everything unused would free;
    /// `None` before Docker was asked. Without Docker there is nothing.
    pub(crate) fn recoverable(&self) -> Option<ByteSize> {
        match self.state.as_ref()? {
            DockerState::Ready(inventory) => {
                Some(inventory.usage.iter().map(|usage| usage.reclaimable).sum())
            }
            DockerState::NotInstalled => Some(ByteSize::ZERO),
            DockerState::NotRunning | DockerState::Failed(_) => None,
        }
    }

    /// The largest unused resource, which fills the size bar completely.
    pub(crate) fn largest(&self) -> ByteSize {
        self.unused()
            .iter()
            .map(|resource| resource.size)
            .max()
            .unwrap_or(ByteSize::ZERO)
    }

    /// Opens the confirmation, or explains why there is nothing to confirm.
    pub(crate) fn request_removal(&mut self) {
        if self.selection().items == 0 {
            self.notice = Some(pt_br::SELECT_SOMETHING);
        } else {
            self.removal.open();
        }
    }

    /// Asks Docker to remove what is selected, in the background.
    pub(crate) fn confirm(&mut self, ctx: &Context, services: &UiServices) {
        let resources: Vec<DockerResource> = self.picked().into_iter().cloned().collect();
        let services = services.clone();
        self.removal.start_work(ctx, move || {
            let report = PruneDocker::new(
                services.container_engine.as_ref(),
                services.history.as_ref(),
            )
            .execute(&resources);
            RemovalResult::new(report, Vec::new()).explaining_skipped(pt_br::DOCKER_SKIPPED)
        });
    }
}

fn key(resource: &DockerResource) -> ResourceKey {
    (resource.kind, resource.reference.clone())
}

fn inspect(services: &UiServices) -> DockerState {
    match InspectDocker::new(services.container_engine.as_ref()).execute() {
        Ok(inventory) => DockerState::Ready(inventory),
        Err(EngineError::NotInstalled) => DockerState::NotInstalled,
        Err(EngineError::NotRunning) => DockerState::NotRunning,
        Err(error) => DockerState::Failed(pt_br::engine_error(&error)),
    }
}

#[cfg(test)]
mod tests {
    use maccleaner_domain::docker::{DockerInventory, DockerResource, KindUsage, ResourceKind};
    use maccleaner_domain::shared::ByteSize;

    use super::{DockerState, DockerViewModel};
    use crate::components::RemovalFlow;

    fn resource(kind: ResourceKind, name: &str, size: u64) -> DockerResource {
        DockerResource {
            kind,
            reference: format!("ref-{name}-{size}"),
            name: name.into(),
            size: ByteSize::from_bytes(size),
            created_unix_seconds: None,
        }
    }

    fn usage(kind: ResourceKind, reclaimable: u64) -> KindUsage {
        KindUsage {
            kind,
            total_count: 3,
            active_count: 1,
            size: ByteSize::from_bytes(reclaimable * 2),
            reclaimable: ByteSize::from_bytes(reclaimable),
        }
    }

    fn model() -> DockerViewModel {
        let mut model = DockerViewModel::default();
        model.accept(DockerState::Ready(DockerInventory {
            engine_version: "24.0.2".into(),
            usage: vec![
                usage(ResourceKind::Image, 1_500),
                usage(ResourceKind::BuildCache, 800),
            ],
            unused: vec![
                resource(ResourceKind::BuildCache, "", 800),
                resource(ResourceKind::Image, "", 300),
                resource(ResourceKind::Image, "web:dev", 1_200),
                resource(ResourceKind::Volume, "pgdata", 400),
            ],
        }));
        model
    }

    #[test]
    fn selects_by_default_only_what_a_build_makes_again() {
        let model = model();
        let selection = model.selection();
        assert_eq!(selection.items, 2);
        assert_eq!(selection.size, ByteSize::from_bytes(1_100));
        assert!(!selection.has_data);
        assert_eq!(model.largest(), ByteSize::from_bytes(1_200));
    }

    #[test]
    fn picking_a_volume_warns_that_its_content_exists_nowhere_else() {
        let mut model = model();
        let volume = model.visible()[3].clone();
        model.toggle(&volume);
        assert!(model.selection().has_data);
        assert_eq!(model.selection().items, 3);
    }

    #[test]
    fn the_filter_narrows_the_list_but_not_the_selection() {
        let mut model = model();
        assert_eq!(
            model.kinds(),
            [
                ResourceKind::BuildCache,
                ResourceKind::Image,
                ResourceKind::Volume
            ]
        );
        model.filter = Some(ResourceKind::Image);
        assert_eq!(model.visible().len(), 2);
        assert_eq!(model.selection().items, 2);
    }

    #[test]
    fn the_dashboard_gets_what_docker_itself_calls_reclaimable() {
        assert_eq!(DockerViewModel::default().recoverable(), None);
        assert_eq!(model().recoverable(), Some(ByteSize::from_bytes(2_300)));

        let mut missing = DockerViewModel::default();
        missing.accept(DockerState::NotInstalled);
        assert_eq!(missing.recoverable(), Some(ByteSize::ZERO));
        let mut closed = DockerViewModel::default();
        closed.accept(DockerState::NotRunning);
        assert_eq!(closed.recoverable(), None);
        assert!(closed.visible().is_empty());
    }

    #[test]
    fn asking_to_clean_with_nothing_selected_explains_instead_of_confirming() {
        let mut model = model();
        for selected in model.visible().into_iter().cloned().collect::<Vec<_>>() {
            if model.is_selected(&selected) {
                model.toggle(&selected);
            }
        }
        model.request_removal();
        assert!(matches!(model.removal, RemovalFlow::Closed));
        assert!(model.notice().is_some());

        let cache = model.visible()[0].clone();
        model.toggle(&cache);
        model.request_removal();
        assert!(matches!(model.removal, RemovalFlow::Confirm));
    }
}

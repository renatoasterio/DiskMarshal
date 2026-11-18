//! State and logic of the apps screen: the installed apps, the one chosen,
//! what uninstalling it removes, what is selected, and the removal.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use diskmarshal_application::ApplicationError;
use diskmarshal_application::progress::{
    CancellationToken, NoProgress, ProgressSink, ScanProgress,
};
use diskmarshal_application::use_cases::{
    DeletionReport, ListInstalledApps, PlanUninstall, UninstallSystemItems,
};
use diskmarshal_domain::applications::{InstalledApp, UninstallItem, UninstallPlan};
use diskmarshal_domain::cleanup::{PrivilegeScope, RiskLevel};
use diskmarshal_domain::deletion::DeletionRequest;
use diskmarshal_domain::shared::ByteSize;
use egui::Context;

use crate::app::UiServices;
use crate::components::{RemovalFlow, RemovalResult, delete_items};
use crate::i18n::pt_br;
use crate::tasks::Search;

/// What the confirmation dialog tells the user.
pub(crate) struct Selection {
    pub items: usize,
    pub reclaimable: ByteSize,
    /// Whether some of the selected items go to the Trash.
    pub uses_trash: bool,
    /// Whether some of the selected items need an administrator.
    pub needs_administrator: bool,
    /// Whether some of the selected items are not sure matches.
    pub has_review: bool,
}

/// Everything the apps screen remembers between frames.
pub(crate) struct AppsViewModel {
    listing: Search<Vec<InstalledApp>>,
    apps: Option<Vec<InstalledApp>>,
    has_started: bool,
    /// What the user typed to narrow the list of apps.
    pub query: String,
    /// The bundle of the app whose uninstall is being previewed.
    chosen: Option<PathBuf>,
    planning: Search<UninstallPlan>,
    plan: Option<UninstallPlan>,
    selected: BTreeSet<PathBuf>,
    notice: Option<String>,
    /// Whether this installation can remove with administrator rights.
    removes_system: bool,
    pub removal: RemovalFlow,
}

impl Default for AppsViewModel {
    fn default() -> Self {
        Self {
            listing: Search::default(),
            apps: None,
            has_started: false,
            query: String::new(),
            chosen: None,
            planning: Search::default(),
            plan: None,
            selected: BTreeSet::new(),
            notice: None,
            removes_system: false,
            removal: RemovalFlow::Closed,
        }
    }
}

impl AppsViewModel {
    /// Lists the apps the first time the screen is shown.
    pub(crate) fn ensure_started(&mut self, ctx: &Context, services: &UiServices) {
        if !self.has_started {
            self.list(ctx, services);
        }
    }

    /// Lists the installed apps again, in the background.
    pub(crate) fn list(&mut self, ctx: &Context, services: &UiServices) {
        self.forget_choice();
        self.has_started = true;
        self.removes_system = services.privileged.is_available();
        let services = services.clone();
        self.listing.start(ctx, move |progress, cancellation| {
            list(&services, progress, cancellation)
        });
    }

    /// Lists the apps and examines the first one that is closed right now,
    /// blocking until it ends. Used to open the app with the screen already
    /// filled in.
    pub(crate) fn list_now(&mut self, services: &UiServices) {
        self.forget_choice();
        self.has_started = true;
        self.removes_system = services.privileged.is_available();
        let (progress, cancellation) = (NoProgress, CancellationToken::new());
        let outcome = list(services, &progress, &cancellation);
        let Some(apps) = self.listing.settle(outcome) else {
            return;
        };
        let first = apps.iter().find(|app| !app.is_running).cloned();
        self.apps = Some(apps);
        let (Some(app), Some(apps)) = (first, &self.apps) else {
            return;
        };
        self.chosen = Some(app.path.clone());
        let outcome = plan(services, &app, apps, &progress, &cancellation);
        if let Some(plan) = self.planning.settle(outcome) {
            self.accept_plan(plan);
        }
    }

    pub(crate) fn cancel(&mut self) {
        self.listing.cancel();
    }

    /// Picks up the results of finished background work.
    pub(crate) fn poll(&mut self) {
        if let Some(apps) = self.listing.poll() {
            self.apps = Some(apps);
        }
        if let Some(plan) = self.planning.poll() {
            self.accept_plan(plan);
        }
    }

    fn forget_choice(&mut self) {
        self.chosen = None;
        self.plan = None;
        self.selected.clear();
        self.notice = None;
        self.planning.cancel();
    }

    /// Totals of the listing under way, when there is one.
    pub(crate) fn progress(&self) -> Option<ScanProgress> {
        self.listing.progress()
    }

    pub(crate) fn apps(&self) -> Option<&[InstalledApp]> {
        self.apps.as_deref()
    }

    /// Why the last listing failed, when it did.
    pub(crate) fn failure(&self) -> Option<&str> {
        self.listing.failure()
    }

    /// Why the chosen app could not be examined, when it could not.
    pub(crate) fn plan_failure(&self) -> Option<&str> {
        self.planning.failure()
    }

    pub(crate) fn notice(&self) -> Option<&str> {
        self.notice.as_deref()
    }

    /// Whether this installation can remove with administrator rights.
    pub(crate) fn removes_system(&self) -> bool {
        self.removes_system
    }

    /// The apps whose name or identifier holds what the user typed.
    pub(crate) fn visible(&self) -> Vec<&InstalledApp> {
        let query = self.query.trim().to_lowercase();
        let matches = |text: &str| text.to_lowercase().contains(&query);
        self.apps
            .iter()
            .flatten()
            .filter(|app| {
                query.is_empty()
                    || matches(&app.name)
                    || matches(&app.bundle_id)
                    || app.declared_name.as_deref().is_some_and(matches)
            })
            .collect()
    }

    /// How many apps are installed and how much space their bundles use.
    pub(crate) fn totals(&self) -> (usize, ByteSize) {
        let apps = self.apps.as_deref().unwrap_or_default();
        (apps.len(), apps.iter().map(|app| app.size).sum())
    }

    /// The app whose uninstall is being previewed.
    pub(crate) fn chosen_app(&self) -> Option<&InstalledApp> {
        let chosen = self.chosen.as_deref()?;
        self.apps.iter().flatten().find(|app| app.path == chosen)
    }

    pub(crate) fn is_chosen(&self, bundle: &Path) -> bool {
        self.chosen.as_deref() == Some(bundle)
    }

    /// Starts examining the app at `bundle`, in the background.
    pub(crate) fn choose(&mut self, ctx: &Context, services: &UiServices, bundle: &Path) {
        if self.is_chosen(bundle) && self.plan_failure().is_none() {
            return;
        }
        self.forget_choice();
        let Some(apps) = self.apps.clone() else {
            return;
        };
        let Some(app) = apps.iter().find(|app| app.path == bundle).cloned() else {
            return;
        };
        self.chosen = Some(app.path.clone());
        let services = services.clone();
        self.planning.start(ctx, move |progress, cancellation| {
            plan(&services, &app, &apps, progress, cancellation)
        });
    }

    /// Whether the chosen app is being examined right now.
    pub(crate) fn is_planning(&self) -> bool {
        self.planning.progress().is_some()
    }

    fn accept_plan(&mut self, plan: UninstallPlan) {
        if !self.is_chosen(&plan.app.path) {
            return;
        }
        // Removing what an app left while the app itself stays would only
        // reset it. So nothing is picked for the user unless the bundle,
        // which comes first in the plan, can go too.
        let can_remove_bundle = plan
            .items
            .first()
            .is_some_and(|bundle| self.is_selectable(bundle, &plan.app));
        self.selected = plan
            .items
            .iter()
            .filter(|item| {
                can_remove_bundle && item.is_default_choice() && self.is_selectable(item, &plan.app)
            })
            .map(|item| item.request.target.clone())
            .collect();
        self.plan = Some(plan);
    }

    pub(crate) fn plan(&self) -> Option<&UninstallPlan> {
        self.plan.as_ref()
    }

    /// An item can be picked when the app is closed and this installation
    /// is able to remove it.
    pub(crate) fn is_selectable(&self, item: &UninstallItem, app: &InstalledApp) -> bool {
        let is_removable = match item.scope {
            PrivilegeScope::User => true,
            PrivilegeScope::Administrator => self.removes_system,
        };
        is_removable && !app.is_running
    }

    pub(crate) fn is_selected(&self, target: &Path) -> bool {
        self.selected.contains(target)
    }

    pub(crate) fn toggle(&mut self, target: &Path) {
        self.notice = None;
        if !self.selected.remove(target) {
            self.selected.insert(target.to_path_buf());
        }
    }

    fn picked(&self) -> Vec<&UninstallItem> {
        self.plan
            .iter()
            .flat_map(|plan| &plan.items)
            .filter(|item| self.is_selected(&item.request.target))
            .collect()
    }

    /// What is selected right now.
    pub(crate) fn selection(&self) -> Selection {
        let picked = self.picked();
        let has = |scope| picked.iter().any(|item| item.scope == scope);
        Selection {
            items: picked.len(),
            reclaimable: picked.iter().map(|item| item.request.reclaimable).sum(),
            uses_trash: has(PrivilegeScope::User),
            needs_administrator: has(PrivilegeScope::Administrator),
            has_review: picked.iter().any(|item| item.risk == RiskLevel::Review),
        }
    }

    /// Opens the confirmation, or explains why there is nothing to confirm.
    pub(crate) fn request_removal(&mut self) {
        let Some(app) = self.chosen_app() else {
            self.notice = Some(pt_br::APPS_CHOOSE_FIRST.to_owned());
            return;
        };
        if app.is_running {
            self.notice = Some(pt_br::apps_running(&app.name));
        } else if self.selection().items == 0 {
            self.notice = Some(pt_br::SELECT_SOMETHING.to_owned());
        } else {
            self.removal.open();
        }
    }

    /// Removes what is selected, in the background.
    pub(crate) fn confirm(&mut self, ctx: &Context, services: &UiServices) {
        let Some(plan) = &self.plan else { return };
        let of_scope = |scope| {
            self.picked()
                .into_iter()
                .filter(move |item| item.scope == scope)
        };
        let requests: Vec<DeletionRequest> = of_scope(PrivilegeScope::User)
            .map(|item| item.request.clone())
            .collect();
        let system_items: Vec<PathBuf> = of_scope(PrivilegeScope::Administrator)
            .map(|item| item.request.target.clone())
            .collect();
        let (app, roots, services) = (plan.app.clone(), plan.roots.clone(), services.clone());
        self.removal.start_work(ctx, move || {
            uninstall(&services, &app, &requests, &roots, &system_items)
        });
    }
}

fn list(
    services: &UiServices,
    progress: &dyn ProgressSink,
    cancellation: &CancellationToken,
) -> Result<Vec<InstalledApp>, ApplicationError> {
    ListInstalledApps::new(
        services.walker.as_ref(),
        services.inspector.as_ref(),
        services.bundles.as_ref(),
        services.running_apps.as_ref(),
    )
    .execute(&services.application_folders, progress, cancellation)
}

fn plan(
    services: &UiServices,
    app: &InstalledApp,
    installed: &[InstalledApp],
    progress: &dyn ProgressSink,
    cancellation: &CancellationToken,
) -> Result<UninstallPlan, ApplicationError> {
    PlanUninstall::new(
        services.rules.as_ref(),
        services.folder_lister.as_ref(),
        services.walker.as_ref(),
        services.inspector.as_ref(),
    )
    .execute(app, installed, progress, cancellation)
}

/// Removes the selected items of `app`: what needs an administrator through
/// the privileged cleaner, the rest through the one use case that deletes.
fn uninstall(
    services: &UiServices,
    app: &InstalledApp,
    requests: &[DeletionRequest],
    roots: &[PathBuf],
    system_items: &[PathBuf],
) -> RemovalResult {
    let mut report = DeletionReport::default();
    // An app opened since the preview would recreate what is removed.
    let is_running = services
        .running_apps
        .running_executables()
        .iter()
        .any(|executable| app.owns_executable(executable));
    if is_running {
        return RemovalResult::new(report, vec![pt_br::apps_opened_meanwhile(&app.name)]);
    }
    // What needs an administrator goes first: when the user gives up at the
    // authentication, nothing at all has been removed.
    if !system_items.is_empty() {
        let removal =
            UninstallSystemItems::new(services.privileged.as_ref(), services.history.as_ref());
        match removal.execute(&app.bundle_id, system_items, pt_br::APPS_ADMIN_PROMPT) {
            Ok(removed) => report.absorb(removed),
            Err(error) => {
                return RemovalResult::new(report, vec![pt_br::uninstall_privilege_error(&error)]);
            }
        }
    }
    report.absorb(delete_items(services, requests, roots));
    RemovalResult::new(report, Vec::new())
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use diskmarshal_domain::applications::{
        InstalledApp, LeftoverKind, UninstallItem, UninstallPlan,
    };
    use diskmarshal_domain::cleanup::{PrivilegeScope, RiskLevel};
    use diskmarshal_domain::deletion::{DeletionMethod, DeletionRequest, ItemFingerprint};
    use diskmarshal_domain::shared::ByteSize;

    use super::AppsViewModel;
    use crate::components::RemovalFlow;

    const BUNDLE: &str = "/Applications/Editor.app";
    const CACHES: &str = "/Users/tester/Library/Caches/com.vendor.editor";
    const SUPPORT: &str = "/Users/tester/Library/Application Support/Editor";
    const DAEMON: &str = "/Library/LaunchDaemons/com.vendor.editor.plist";

    fn app(name: &str, bundle_id: &str, is_running: bool) -> InstalledApp {
        InstalledApp {
            name: name.into(),
            declared_name: None,
            bundle_id: bundle_id.into(),
            version: Some("1.0".into()),
            path: PathBuf::from(format!("/Applications/{name}.app")),
            size: ByteSize::from_bytes(1_000),
            scope: PrivilegeScope::User,
            is_running,
        }
    }

    fn item(target: &str, scope: PrivilegeScope, risk: RiskLevel, size: u64) -> UninstallItem {
        UninstallItem {
            kind: LeftoverKind::Caches,
            scope,
            risk,
            request: DeletionRequest {
                target: PathBuf::from(target),
                method: DeletionMethod::Trash,
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

    /// A model with the editor chosen and examined.
    fn model(is_running: bool, removes_system: bool) -> AppsViewModel {
        let editor = app("Editor", "com.vendor.editor", is_running);
        let (user, admin) = (PrivilegeScope::User, PrivilegeScope::Administrator);
        let plan = UninstallPlan {
            app: editor.clone(),
            items: vec![
                item(BUNDLE, user, RiskLevel::Safe, 1_000),
                item(CACHES, user, RiskLevel::Safe, 400),
                item(SUPPORT, user, RiskLevel::Review, 900),
                item(DAEMON, admin, RiskLevel::Safe, 50),
            ],
            roots: Vec::new(),
            has_other_copy: false,
        };
        let mut model = AppsViewModel {
            apps: Some(vec![editor, app("Viewer", "org.other.viewer", false)]),
            chosen: Some(PathBuf::from(BUNDLE)),
            removes_system,
            ..AppsViewModel::default()
        };
        model.accept_plan(plan);
        model
    }

    #[test]
    fn selects_by_default_the_sure_matches_this_installation_can_remove() {
        let limited = model(false, false);
        assert!(limited.is_selected(Path::new(BUNDLE)));
        assert!(limited.is_selected(Path::new(CACHES)));
        assert!(
            !limited.is_selected(Path::new(SUPPORT)),
            "review is never preselected"
        );
        assert!(
            !limited.is_selected(Path::new(DAEMON)),
            "no privileged cleaner here"
        );
        let selection = limited.selection();
        assert_eq!(selection.items, 2);
        assert_eq!(selection.reclaimable, ByteSize::from_bytes(1_400));
        assert!(selection.uses_trash && !selection.needs_administrator && !selection.has_review);

        let mut complete = model(false, true);
        assert!(complete.is_selected(Path::new(DAEMON)));
        complete.toggle(Path::new(SUPPORT));
        let selection = complete.selection();
        assert_eq!(selection.items, 4);
        assert!(selection.needs_administrator && selection.has_review);
    }

    #[test]
    fn nothing_is_preselected_when_the_app_itself_cannot_be_removed_here() {
        let editor = InstalledApp {
            scope: PrivilegeScope::Administrator,
            ..app("Editor", "com.vendor.editor", false)
        };
        let plan = UninstallPlan {
            app: editor.clone(),
            items: vec![
                item(
                    BUNDLE,
                    PrivilegeScope::Administrator,
                    RiskLevel::Safe,
                    1_000,
                ),
                item(CACHES, PrivilegeScope::User, RiskLevel::Safe, 400),
            ],
            roots: Vec::new(),
            has_other_copy: false,
        };
        let examine = |removes_system| {
            let mut model = AppsViewModel {
                apps: Some(vec![editor.clone()]),
                chosen: Some(PathBuf::from(BUNDLE)),
                removes_system,
                ..AppsViewModel::default()
            };
            model.accept_plan(plan.clone());
            model
        };

        let mut limited = examine(false);
        assert_eq!(
            limited.selection().items,
            0,
            "the caches alone would only reset the app"
        );
        limited.toggle(Path::new(CACHES));
        assert_eq!(
            limited.selection().items,
            1,
            "the user may still ask for it"
        );

        assert_eq!(examine(true).selection().items, 2);
    }

    #[test]
    fn an_app_that_is_open_offers_nothing_and_says_why() {
        let mut open = model(true, true);
        assert_eq!(open.selection().items, 0);
        let plan = open.plan().unwrap().clone();
        assert!(
            plan.items
                .iter()
                .all(|item| !open.is_selectable(item, &plan.app))
        );

        open.request_removal();
        assert!(matches!(open.removal, RemovalFlow::Closed));
        assert!(open.notice().is_some_and(|text| text.contains("Editor")));
    }

    #[test]
    fn asking_to_uninstall_with_nothing_chosen_or_selected_explains_instead_of_confirming() {
        let mut empty = AppsViewModel::default();
        empty.request_removal();
        assert!(matches!(empty.removal, RemovalFlow::Closed));
        assert!(empty.notice().is_some());

        let mut model = model(false, false);
        model.toggle(Path::new(BUNDLE));
        model.toggle(Path::new(CACHES));
        model.request_removal();
        assert!(matches!(model.removal, RemovalFlow::Closed));
        assert!(model.notice().is_some());

        model.toggle(Path::new(CACHES));
        assert!(model.notice().is_none());
        model.request_removal();
        assert!(matches!(model.removal, RemovalFlow::Confirm));
    }

    #[test]
    fn the_search_narrows_the_list_by_name_or_identifier() {
        let mut model = model(false, false);
        assert_eq!(model.visible().len(), 2);
        assert_eq!(model.totals(), (2, ByteSize::from_bytes(2_000)));

        model.query = " edit ".into();
        let names: Vec<&str> = model
            .visible()
            .iter()
            .map(|app| app.name.as_str())
            .collect();
        assert_eq!(names, ["Editor"]);

        model.query = "ORG.OTHER".into();
        assert_eq!(model.visible().len(), 1);
        model.query = "nothing like this".into();
        assert!(model.visible().is_empty());
    }

    #[test]
    fn a_plan_that_arrives_for_another_app_is_ignored() {
        let mut model = model(false, false);
        model.chosen = Some(PathBuf::from("/Applications/Viewer.app"));
        model.plan = None;
        model.selected.clear();

        let stale = UninstallPlan {
            app: app("Editor", "com.vendor.editor", false),
            items: vec![item(BUNDLE, PrivilegeScope::User, RiskLevel::Safe, 1)],
            roots: Vec::new(),
            has_other_copy: false,
        };
        model.accept_plan(stale);

        assert!(model.plan().is_none());
        assert_eq!(model.selection().items, 0);
    }
}

//! State and logic of the cleanup screen: the analysis, what is selected, and
//! the removal.

use std::collections::BTreeSet;

use diskmarshal_application::ApplicationError;
use diskmarshal_application::progress::{CancellationToken, NoProgress, ScanProgress};
use diskmarshal_application::use_cases::PlanCleanup;
use diskmarshal_domain::cleanup::{CleanupFinding, CleanupPlan, PrivilegeScope, RiskLevel};
use diskmarshal_domain::deletion::DeletionMethod;
use diskmarshal_domain::shared::ByteSize;
use egui::Context;

use crate::app::UiServices;
use crate::components::RemovalFlow;
use crate::i18n::pt_br;
use crate::tasks::Search;

type Analysis = Result<CleanupPlan, ApplicationError>;

/// What the confirmation dialog tells the user.
pub(crate) struct Selection {
    pub rules: usize,
    pub items: usize,
    pub reclaimable: ByteSize,
    /// Whether some of the selected content goes to the Trash.
    pub uses_trash: bool,
    /// Whether some of the selected rules need an administrator.
    pub needs_administrator: bool,
}

/// Everything the cleanup screen remembers between frames.
pub(crate) struct CleanupViewModel {
    analysis: Search<CleanupPlan>,
    plan: Option<CleanupPlan>,
    selected: BTreeSet<String>,
    notice: Option<&'static str>,
    has_started: bool,
    /// Whether this installation can clean with administrator rights.
    cleans_system: bool,
    pub removal: RemovalFlow,
}

impl Default for CleanupViewModel {
    fn default() -> Self {
        Self {
            analysis: Search::default(),
            plan: None,
            selected: BTreeSet::new(),
            notice: None,
            has_started: false,
            cleans_system: false,
            removal: RemovalFlow::Closed,
        }
    }
}

impl CleanupViewModel {
    /// Starts the first analysis the first time the screen is shown.
    pub(crate) fn ensure_started(&mut self, ctx: &Context, services: &UiServices) {
        if !self.has_started {
            self.analyze(ctx, services);
        }
    }

    /// Analyzes every rule again, in the background.
    pub(crate) fn analyze(&mut self, ctx: &Context, services: &UiServices) {
        self.has_started = true;
        self.cleans_system = services.privileged.is_available();
        let services = services.clone();
        self.analysis.start(ctx, move |progress, cancellation| {
            plan(&services, progress, cancellation)
        });
    }

    /// Analyzes right now, blocking until it ends. Used to open the app with
    /// the screen already filled in.
    pub(crate) fn analyze_now(&mut self, services: &UiServices) {
        self.has_started = true;
        self.cleans_system = services.privileged.is_available();
        let outcome = plan(services, &NoProgress, &CancellationToken::new());
        if let Some(plan) = self.analysis.settle(outcome) {
            self.accept(plan);
        }
    }

    /// Picks up the results of finished background work.
    pub(crate) fn poll(&mut self) {
        if let Some(plan) = self.analysis.poll() {
            self.accept(plan);
        }
    }

    fn accept(&mut self, plan: CleanupPlan) {
        self.selected = plan
            .findings
            .iter()
            // Nothing that needs an administrator is ever picked for the user.
            .filter(|finding| {
                finding.rule.scope == PrivilegeScope::User
                    && self.is_selectable(finding)
                    && finding.rule.risk == RiskLevel::Safe
            })
            .map(|finding| finding.rule.id.clone())
            .collect();
        self.plan = Some(plan);
    }

    /// Totals of the analysis under way, when there is one.
    pub(crate) fn progress(&self) -> Option<ScanProgress> {
        self.analysis.progress()
    }

    pub(crate) fn plan(&self) -> Option<&CleanupPlan> {
        self.plan.as_ref()
    }

    pub(crate) fn failure(&self) -> Option<&str> {
        self.analysis.failure()
    }

    pub(crate) fn notice(&self) -> Option<&'static str> {
        self.notice
    }

    /// A rule can be picked when it found something and this installation
    /// is able to clean it.
    pub(crate) fn is_selectable(&self, finding: &CleanupFinding) -> bool {
        let is_cleanable = match finding.rule.scope {
            PrivilegeScope::User => true,
            PrivilegeScope::Administrator => self.cleans_system,
        };
        is_cleanable && !finding.requests.is_empty()
    }

    /// Whether this installation can clean with administrator rights.
    pub(crate) fn cleans_system(&self) -> bool {
        self.cleans_system
    }

    pub(crate) fn is_selected(&self, rule_id: &str) -> bool {
        self.selected.contains(rule_id)
    }

    pub(crate) fn toggle(&mut self, rule_id: &str) {
        self.notice = None;
        if !self.selected.remove(rule_id) {
            self.selected.insert(rule_id.to_owned());
        }
    }

    /// What is selected right now.
    pub(crate) fn selection(&self) -> Selection {
        let findings: Vec<&CleanupFinding> = self
            .plan
            .iter()
            .flat_map(|plan| &plan.findings)
            .filter(|finding| self.is_selected(&finding.rule.id))
            .collect();
        Selection {
            rules: findings.len(),
            items: findings.iter().map(|finding| finding.requests.len()).sum(),
            reclaimable: findings.iter().map(|finding| finding.reclaimable()).sum(),
            uses_trash: findings
                .iter()
                .any(|finding| finding.rule.method == DeletionMethod::Trash),
            needs_administrator: findings
                .iter()
                .any(|finding| finding.rule.scope == PrivilegeScope::Administrator),
        }
    }

    /// What the user could clean right now; `None` before the first analysis.
    pub(crate) fn recoverable(&self) -> Option<ByteSize> {
        let plan = self.plan.as_ref()?;
        Some(
            plan.findings
                .iter()
                .filter(|finding| self.is_selectable(finding))
                .map(CleanupFinding::reclaimable)
                .sum(),
        )
    }

    /// The largest finding, which fills the size bar completely.
    pub(crate) fn largest(&self) -> ByteSize {
        self.plan
            .iter()
            .flat_map(|plan| &plan.findings)
            .map(CleanupFinding::reclaimable)
            .max()
            .unwrap_or(ByteSize::ZERO)
    }

    /// Opens the confirmation, or explains why there is nothing to confirm.
    pub(crate) fn request_removal(&mut self) {
        if self.selection().rules == 0 {
            self.notice = Some(pt_br::SELECT_SOMETHING);
        } else {
            self.removal.open();
        }
    }

    /// Removes what is selected, in the background.
    pub(crate) fn confirm(&mut self, ctx: &Context, services: &UiServices) {
        let Some(plan) = &self.plan else { return };
        // The user's own rules are removed here, from the preview. System
        // rules go by id to the privileged helper, which analyzes them again.
        let selected_of = |scope: PrivilegeScope| -> Vec<&str> {
            plan.findings
                .iter()
                .filter(|finding| finding.rule.scope == scope && self.is_selected(&finding.rule.id))
                .map(|finding| finding.rule.id.as_str())
                .collect()
        };
        let requests = plan.requests_for(&selected_of(PrivilegeScope::User));
        let system_rules = selected_of(PrivilegeScope::Administrator)
            .into_iter()
            .map(str::to_owned)
            .collect();
        self.removal
            .start_with_system(ctx, services, requests, plan.roots.clone(), system_rules);
    }
}

fn plan(
    services: &UiServices,
    progress: &dyn diskmarshal_application::progress::ProgressSink,
    cancellation: &CancellationToken,
) -> Analysis {
    PlanCleanup::new(
        services.rules.as_ref(),
        services.walker.as_ref(),
        services.inspector.as_ref(),
    )
    .execute(services.wall_clock.unix_seconds(), progress, cancellation)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use diskmarshal_domain::cleanup::{
        CleanupFinding, CleanupPlan, CleanupRule, PrivilegeScope, RiskLevel,
    };
    use diskmarshal_domain::deletion::{DeletionMethod, DeletionRequest, ItemFingerprint};
    use diskmarshal_domain::shared::ByteSize;

    use super::CleanupViewModel;
    use crate::components::RemovalFlow;

    fn finding(id: &str, risk: RiskLevel, scope: PrivilegeScope, sizes: &[u64]) -> CleanupFinding {
        let rule = CleanupRule {
            id: id.into(),
            name: id.into(),
            description: String::new(),
            locations: vec![PathBuf::from("/x")],
            risk,
            scope,
            method: DeletionMethod::Permanent,
            minimum_age_days: None,
        };
        let requests = sizes
            .iter()
            .map(|size| DeletionRequest {
                target: PathBuf::from(format!("/x/{id}-{size}")),
                method: DeletionMethod::Permanent,
                fingerprint: ItemFingerprint {
                    device: 1,
                    inode: *size,
                    size: *size,
                    modified_unix_seconds: 0,
                },
                reclaimable: ByteSize::from_bytes(*size),
            })
            .collect();
        CleanupFinding { rule, requests }
    }

    fn model() -> CleanupViewModel {
        let mut model = CleanupViewModel::default();
        model.accept(CleanupPlan {
            findings: vec![
                finding("caches", RiskLevel::Safe, PrivilegeScope::User, &[400, 100]),
                finding("archives", RiskLevel::Review, PrivilegeScope::User, &[900]),
                finding("empty", RiskLevel::Safe, PrivilegeScope::User, &[]),
                finding(
                    "system",
                    RiskLevel::Safe,
                    PrivilegeScope::Administrator,
                    &[700],
                ),
            ],
            roots: vec![PathBuf::from("/x")],
        });
        model
    }

    #[test]
    fn selects_by_default_only_safe_user_rules_that_found_something() {
        let model = model();
        assert!(model.is_selected("caches"));
        assert!(!model.is_selected("archives"));
        assert!(!model.is_selected("empty"));
        assert!(!model.is_selected("system"));
        let selection = model.selection();
        assert_eq!((selection.rules, selection.items), (1, 2));
        assert_eq!(selection.reclaimable, ByteSize::from_bytes(500));
    }

    #[test]
    fn system_rules_can_be_picked_only_where_they_can_be_cleaned_and_never_by_default() {
        let locked = model();
        let system = &locked.plan().unwrap().findings[3];
        assert!(!locked.is_selectable(system), "no privileged cleaner here");
        assert!(!locked.selection().needs_administrator);

        let mut unlocked = CleanupViewModel {
            cleans_system: true,
            ..CleanupViewModel::default()
        };
        unlocked.accept(locked.plan().unwrap().clone());
        let system = unlocked.plan().unwrap().findings[3].clone();
        assert!(unlocked.is_selectable(&system));
        assert!(
            !unlocked.is_selected("system"),
            "the user has to ask for it"
        );

        unlocked.toggle("system");
        let selection = unlocked.selection();
        assert!(selection.needs_administrator);
        assert_eq!(selection.reclaimable, ByteSize::from_bytes(1_200));
    }

    #[test]
    fn toggling_changes_the_totals() {
        let mut model = model();
        model.toggle("archives");
        assert_eq!(model.selection().reclaimable, ByteSize::from_bytes(1_400));
        model.toggle("caches");
        assert_eq!(model.selection().reclaimable, ByteSize::from_bytes(900));
        assert_eq!(model.largest(), ByteSize::from_bytes(900));
    }

    #[test]
    fn asking_to_remove_with_nothing_selected_explains_instead_of_confirming() {
        let mut model = model();
        model.toggle("caches");

        model.request_removal();
        assert!(matches!(model.removal, RemovalFlow::Closed));
        assert!(model.notice().is_some());

        model.toggle("caches");
        model.request_removal();
        assert!(matches!(model.removal, RemovalFlow::Confirm));
        assert!(model.notice().is_none());
    }
}

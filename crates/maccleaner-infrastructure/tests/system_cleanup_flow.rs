//! What the privileged helper does, run here on temporary folders: clean
//! chosen rules of one privilege scope, with no preview.

use std::path::PathBuf;

use maccleaner_application::ApplicationError;
use maccleaner_application::ports::{HistoryRepository, RuleCatalog, TrashBin};
use maccleaner_application::use_cases::{
    CleanRules, DeleteItems, DeletionPorts, ExecutionMode, PlanCleanup,
};
use maccleaner_domain::applications::LeftoverLocation;
use maccleaner_domain::cleanup::{CleanupRule, PrivilegeScope, RiskLevel};
use maccleaner_domain::deletion::{DeletionMethod, DeletionOutcome, HistoryEntry, ProtectedPaths};
use maccleaner_domain::dev_artifacts::ArtifactRule;
use maccleaner_infrastructure::filesystem::{LstatFileInspector, ReaddirWalker, SafeFileRemover};
use maccleaner_test_support::TempTree;

struct FixedCatalog(Vec<CleanupRule>);

impl RuleCatalog for FixedCatalog {
    fn cleanup_rules(&self) -> Result<Vec<CleanupRule>, ApplicationError> {
        Ok(self.0.clone())
    }

    fn artifact_rules(&self) -> Result<Vec<ArtifactRule>, ApplicationError> {
        Ok(Vec::new())
    }

    fn leftover_locations(&self) -> Result<Vec<LeftoverLocation>, ApplicationError> {
        Ok(Vec::new())
    }
}

struct NoTrash;

impl TrashBin for NoTrash {
    fn move_to_trash(&self, path: &std::path::Path) -> Result<(), ApplicationError> {
        Err(ApplicationError::RootNotFound(path.to_path_buf()))
    }
}

struct NoHistory;

impl HistoryRepository for NoHistory {
    fn append(&self, _outcome: &DeletionOutcome) -> Result<(), ApplicationError> {
        Ok(())
    }

    fn recent(&self, _limit: usize) -> Vec<HistoryEntry> {
        Vec::new()
    }
}

fn rule(id: &str, scope: PrivilegeScope, location: PathBuf) -> CleanupRule {
    CleanupRule {
        id: id.to_owned(),
        name: id.to_owned(),
        description: String::new(),
        locations: vec![location],
        risk: RiskLevel::Safe,
        scope,
        method: DeletionMethod::Permanent,
        minimum_age_days: None,
    }
}

/// Runs the cleanup the way the helper does and returns its result.
fn clean(
    fixture: &TempTree,
    rule_ids: &[&str],
    mode: ExecutionMode,
) -> Result<Vec<DeletionOutcome>, ApplicationError> {
    let catalog = FixedCatalog(vec![
        rule(
            "system-caches",
            PrivilegeScope::Administrator,
            fixture.path("system/caches"),
        ),
        rule(
            "user-caches",
            PrivilegeScope::User,
            fixture.path("user/caches"),
        ),
    ]);
    let (walker, inspector) = (ReaddirWalker::default(), LstatFileInspector);
    let policy = ProtectedPaths::new(&fixture.path("home"));
    let ports = DeletionPorts {
        inspector: &inspector,
        remover: &SafeFileRemover,
        trash_bin: &NoTrash,
        history: &NoHistory,
    };
    let ids: Vec<String> = rule_ids.iter().map(|id| (*id).to_owned()).collect();
    CleanRules::new(
        &catalog,
        PlanCleanup::new(&catalog, &walker, &inspector),
        DeleteItems::new(&policy, ports),
    )
    .execute(&ids, PrivilegeScope::Administrator, 0, mode)
    .map(|report| report.outcomes)
}

fn fixture() -> std::io::Result<TempTree> {
    let fixture = TempTree::new()?;
    fixture.file("system/caches/com.vendor.tool/data.bin", 4_000)?;
    fixture.file("system/caches/loose.cache", 1_000)?;
    fixture.file("user/caches/com.app/data.bin", 2_000)?;
    Ok(fixture)
}

#[test]
fn removes_what_the_administrator_rule_covers_and_nothing_else() {
    let fixture = fixture().unwrap();

    let outcomes = clean(&fixture, &["system-caches"], ExecutionMode::Real).unwrap();

    assert_eq!(outcomes.len(), 2);
    assert!(outcomes.iter().all(DeletionOutcome::is_removed));
    assert!(
        fixture.path("system/caches").is_dir(),
        "the folder of the rule stays"
    );
    assert!(!fixture.path("system/caches/com.vendor.tool").exists());
    assert!(!fixture.path("system/caches/loose.cache").exists());
    assert!(
        fixture.path("user/caches/com.app/data.bin").exists(),
        "another rule's content"
    );
}

#[test]
fn a_simulation_reports_the_same_items_and_removes_nothing() {
    let fixture = fixture().unwrap();

    let outcomes = clean(&fixture, &["system-caches"], ExecutionMode::DryRun).unwrap();

    assert_eq!(outcomes.len(), 2);
    assert!(outcomes.iter().all(|outcome| !outcome.is_removed()));
    assert!(fixture.path("system/caches/loose.cache").exists());
}

#[test]
fn a_rule_of_another_scope_or_an_unknown_one_stops_everything() {
    let fixture = fixture().unwrap();

    for ids in [
        &["user-caches"][..],
        &["system-caches", "user-caches"],
        &["made-up"],
    ] {
        let result = clean(&fixture, ids, ExecutionMode::Real);
        assert!(
            matches!(result, Err(ApplicationError::RuleNotAllowed(_))),
            "{ids:?}"
        );
    }
    assert!(
        fixture.path("system/caches/loose.cache").exists(),
        "nothing ran"
    );
    assert!(fixture.path("user/caches/com.app/data.bin").exists());
}

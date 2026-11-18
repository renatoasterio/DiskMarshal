//! The whole cleanup, on real files in a temporary folder: analyze, preview,
//! remove, and check what is left.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use maccleaner_application::ApplicationError;
use maccleaner_application::ports::{RuleCatalog, TrashBin};
use maccleaner_application::progress::{CancellationToken, NoProgress};
use maccleaner_application::use_cases::{DeleteItems, DeletionPorts, ExecutionMode, PlanCleanup};
use maccleaner_domain::applications::LeftoverLocation;
use maccleaner_domain::cleanup::{CleanupRule, PrivilegeScope, RiskLevel};
use maccleaner_domain::deletion::{DeletionMethod, DeletionStatus, ProtectedPaths};
use maccleaner_domain::dev_artifacts::ArtifactRule;
use maccleaner_infrastructure::filesystem::{LstatFileInspector, ReaddirWalker, SafeFileRemover};
use maccleaner_infrastructure::persistence::JsonlHistoryRepository;
use maccleaner_test_support::TempTree;

/// A catalog with one rule over a folder of the fixture.
struct OneRule(CleanupRule);

impl RuleCatalog for OneRule {
    fn cleanup_rules(&self) -> Result<Vec<CleanupRule>, ApplicationError> {
        Ok(vec![self.0.clone()])
    }

    fn artifact_rules(&self) -> Result<Vec<ArtifactRule>, ApplicationError> {
        Ok(Vec::new())
    }

    fn leftover_locations(&self) -> Result<Vec<LeftoverLocation>, ApplicationError> {
        Ok(Vec::new())
    }
}

/// The Trash is not exercised here; a move to it must never be requested.
struct NoTrash;

impl TrashBin for NoTrash {
    fn move_to_trash(&self, path: &Path) -> Result<(), ApplicationError> {
        Err(ApplicationError::RootNotFound(path.to_path_buf()))
    }
}

fn rule(location: PathBuf, minimum_age_days: Option<u32>) -> CleanupRule {
    CleanupRule {
        id: "fixture-caches".into(),
        name: "Fixture caches".into(),
        description: String::new(),
        locations: vec![location],
        risk: RiskLevel::Safe,
        scope: PrivilegeScope::User,
        method: DeletionMethod::Permanent,
        minimum_age_days,
    }
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX)
        })
}

#[test]
fn removes_the_content_of_a_rule_folder_and_nothing_else() {
    let outside = TempTree::new().unwrap();
    let sentinel = outside.file("thesis.txt", 10).unwrap();
    let fixture = TempTree::new().unwrap();
    fixture.file("caches/app-one/data.bin", 4_000).unwrap();
    fixture.file("caches/app-two/index.db", 1_000).unwrap();
    fixture.file("caches/loose.tmp", 500).unwrap();
    fixture.symlink(outside.root(), "caches/shortcut").unwrap();
    let neighbor = fixture.file("documents/keep.txt", 10).unwrap();
    let caches = fixture.path("caches");

    let catalog = OneRule(rule(caches.clone(), None));
    let (walker, inspector) = (ReaddirWalker::default(), LstatFileInspector);
    let plan = PlanCleanup::new(&catalog, &walker, &inspector)
        .execute(now(), &NoProgress, &CancellationToken::new())
        .unwrap();

    let finding = &plan.findings[0];
    assert_eq!(finding.requests.len(), 4);
    assert!(finding.reclaimable().as_bytes() >= 5_500);

    let history_file = fixture.path("history.jsonl");
    let history = JsonlHistoryRepository::new(history_file.clone());
    let policy = ProtectedPaths::new(Path::new("/Users/nobody"));
    let ports = DeletionPorts {
        inspector: &inspector,
        remover: &SafeFileRemover,
        trash_bin: &NoTrash,
        history: &history,
    };
    let requests = plan.requests_for(&["fixture-caches"]);
    let delete = DeleteItems::new(&policy, ports);

    let preview = delete.execute(&requests, &plan.roots, ExecutionMode::DryRun);
    assert!(
        preview
            .outcomes
            .iter()
            .all(|o| o.status == DeletionStatus::WouldDelete)
    );
    assert_eq!(
        fs::read_dir(&caches).unwrap().count(),
        4,
        "a preview removes nothing"
    );

    let report = delete.execute(&requests, &plan.roots, ExecutionMode::Real);

    assert!(
        report
            .outcomes
            .iter()
            .all(|o| o.status == DeletionStatus::Deleted)
    );
    assert!(caches.is_dir(), "the rule folder itself stays");
    assert_eq!(fs::read_dir(&caches).unwrap().count(), 0);
    assert!(sentinel.exists(), "what a link points to is never touched");
    assert!(neighbor.exists());
    assert_eq!(fs::read_to_string(history_file).unwrap().lines().count(), 4);
}

#[test]
fn an_age_limit_keeps_what_was_touched_recently() {
    let fixture = TempTree::new().unwrap();
    let old = fixture.file("temp/old.tmp", 100).unwrap();
    fixture.file("temp/fresh.tmp", 100).unwrap();
    let ten_days_ago = SystemTime::now() - Duration::from_secs(10 * 86_400);
    fs::File::options()
        .write(true)
        .open(&old)
        .unwrap()
        .set_modified(ten_days_ago)
        .unwrap();

    let catalog = OneRule(rule(fixture.path("temp"), Some(7)));
    let plan = PlanCleanup::new(&catalog, &ReaddirWalker::default(), &LstatFileInspector)
        .execute(now(), &NoProgress, &CancellationToken::new())
        .unwrap();

    let targets: Vec<_> = plan.findings[0]
        .requests
        .iter()
        .map(|r| r.target.clone())
        .collect();
    assert_eq!(targets.len(), 1);
    assert!(targets[0].ends_with("old.tmp"));
}

#[test]
fn a_folder_that_does_not_exist_finds_nothing() {
    let fixture = TempTree::new().unwrap();
    let catalog = OneRule(rule(fixture.path("not-there"), None));

    let plan = PlanCleanup::new(&catalog, &ReaddirWalker::default(), &LstatFileInspector)
        .execute(now(), &NoProgress, &CancellationToken::new())
        .unwrap();

    assert!(plan.findings[0].requests.is_empty());
    assert!(plan.roots.is_empty());
}

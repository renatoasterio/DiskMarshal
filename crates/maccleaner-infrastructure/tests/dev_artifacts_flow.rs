//! Finding build folders in real projects on disk, with the real catalog and
//! a real git repository.

use std::path::{Path, PathBuf};
use std::process::Command;

use maccleaner_application::progress::{CancellationToken, NoProgress};
use maccleaner_application::use_cases::FindDevArtifacts;
use maccleaner_domain::cleanup::RiskLevel;
use maccleaner_infrastructure::catalog::TomlRuleCatalog;
use maccleaner_infrastructure::filesystem::{LstatFileInspector, ReaddirWalker};
use maccleaner_infrastructure::process::GitVersionControlProbe;
use maccleaner_test_support::TempTree;

fn git(directory: &Path, arguments: &[&str]) {
    let output = Command::new("/usr/bin/git")
        .arg("-C")
        .arg(directory)
        .args(["-c", "user.name=Test", "-c", "user.email=test@example.com"])
        .args(arguments)
        .output();
    assert!(
        output.is_ok_and(|output| output.status.success()),
        "git {arguments:?}"
    );
}

#[test]
fn finds_build_folders_and_leaves_versioned_ones_alone() {
    let fixture = TempTree::new().unwrap();
    fixture.file("web/package.json", 10).unwrap();
    fixture
        .file("web/node_modules/left-pad/package.json", 10)
        .unwrap();
    fixture
        .file("web/node_modules/left-pad/node_modules/dep/index.js", 2_000)
        .unwrap();
    fixture.file("web/dist/bundle.js", 700).unwrap();
    fixture.file("web/src/index.ts", 50).unwrap();
    fixture.file("api/Billing.csproj", 10).unwrap();
    fixture.file("api/obj/cache.bin", 900).unwrap();
    fixture.file("api/bin/deploy.sh", 30).unwrap();
    fixture.file("notes/bin/todo.txt", 5).unwrap();
    let api = fixture.path("api");
    git(&api, &["init", "--quiet"]);
    git(&api, &["add", "bin", "Billing.csproj"]);
    git(&api, &["commit", "--quiet", "-m", "scripts"]);

    let catalog = TomlRuleCatalog::new(PathBuf::from("/Users/nobody"), None);
    let report = FindDevArtifacts::new(
        &catalog,
        &ReaddirWalker::default(),
        &LstatFileInspector,
        &GitVersionControlProbe,
    )
    .execute(fixture.root(), &NoProgress, &CancellationToken::new())
    .unwrap();

    let found: Vec<(String, String, RiskLevel)> = report
        .found
        .iter()
        .map(|f| {
            (
                f.artifact.project.clone(),
                f.artifact.folder.clone(),
                f.artifact.risk,
            )
        })
        .collect();
    assert_eq!(
        found,
        [
            ("web".to_owned(), "node_modules".to_owned(), RiskLevel::Safe),
            ("api".to_owned(), "obj".to_owned(), RiskLevel::Safe),
            ("web".to_owned(), "dist".to_owned(), RiskLevel::Review),
        ],
        "largest first; the versioned api/bin and the unrelated notes/bin are not offered"
    );
    for item in &report.found {
        assert!(item.request.target.starts_with(&report.root));
        assert_eq!(item.request.target, item.artifact.path);
    }
}

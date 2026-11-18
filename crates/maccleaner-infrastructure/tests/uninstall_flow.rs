//! Uninstalling an app made of real files on disk: listing what is installed,
//! finding what belongs to one app, and removing it without touching the rest.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use maccleaner_application::ApplicationError;
use maccleaner_application::ports::{RuleCatalog, RunningAppProbe, TrashBin};
use maccleaner_application::progress::{CancellationToken, NoProgress};
use maccleaner_application::use_cases::{
    DeleteItems, DeletionPorts, DeletionReport, ExecutionMode, ListInstalledApps, PlanUninstall,
    RemoveAppItems,
};
use maccleaner_domain::applications::{
    InstalledApp, LeftoverKind, LeftoverLocation, UninstallItem, UninstallPlan,
};
use maccleaner_domain::cleanup::{CleanupRule, PrivilegeScope, RiskLevel};
use maccleaner_domain::deletion::{
    DeletionMethod, DeletionOutcome, DeletionRequest, ProtectedPaths,
};
use maccleaner_domain::dev_artifacts::ArtifactRule;
use maccleaner_infrastructure::bundles::PlistAppBundleReader;
use maccleaner_infrastructure::catalog::TomlRuleCatalog;
use maccleaner_infrastructure::filesystem::{
    LstatFileInspector, ReaddirFolderLister, ReaddirWalker, SafeFileRemover,
};
use maccleaner_infrastructure::persistence::JsonlHistoryRepository;
use maccleaner_test_support::TempTree;

/// The real catalog, narrowed to the folders inside the home of the fixture,
/// so that no test reads or changes a folder of this Mac.
struct HomeOnlyCatalog(Vec<LeftoverLocation>);

impl HomeOnlyCatalog {
    fn for_home(home: &Path) -> Result<Self, ApplicationError> {
        let locations = TomlRuleCatalog::new(home.to_path_buf(), None).leftover_locations()?;
        Ok(Self(
            locations
                .into_iter()
                .filter(|location| location.folder.starts_with(home))
                .collect(),
        ))
    }
}

impl RuleCatalog for HomeOnlyCatalog {
    fn cleanup_rules(&self) -> Result<Vec<CleanupRule>, ApplicationError> {
        Ok(Vec::new())
    }

    fn artifact_rules(&self) -> Result<Vec<ArtifactRule>, ApplicationError> {
        Ok(Vec::new())
    }

    fn leftover_locations(&self) -> Result<Vec<LeftoverLocation>, ApplicationError> {
        Ok(self.0.clone())
    }
}

/// Says the given programs are running.
struct Running(Vec<PathBuf>);

impl RunningAppProbe for Running {
    fn running_executables(&self) -> Vec<PathBuf> {
        self.0.clone()
    }
}

/// A Trash inside the fixture, so the real one stays as it is.
struct FolderTrash(PathBuf);

impl TrashBin for FolderTrash {
    fn move_to_trash(&self, path: &Path) -> Result<(), ApplicationError> {
        let failure = |source| ApplicationError::Io {
            path: path.to_path_buf(),
            source,
        };
        fs::create_dir_all(&self.0).map_err(failure)?;
        let name = path.file_name().unwrap_or_default();
        fs::rename(path, self.0.join(name)).map_err(failure)
    }
}

/// A Mac in miniature: an applications folder and a home folder.
struct Machine {
    fixture: TempTree,
    root: PathBuf,
}

impl Machine {
    fn new() -> io::Result<Self> {
        let fixture = TempTree::new()?;
        // The use cases work on real paths; on macOS the temporary folder is
        // reached through a link.
        let root = fs::canonicalize(fixture.root())?;
        Ok(Self { fixture, root })
    }

    fn applications(&self) -> PathBuf {
        self.root.join("Applications")
    }

    fn home(&self) -> PathBuf {
        self.root.join("home")
    }

    fn app(&self, relative: &str, bundle_id: &str, size: usize) -> io::Result<()> {
        let description = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>{bundle_id}</string>
<key>CFBundleShortVersionString</key><string>1.0</string>
</dict></plist>"#
        );
        let info = self
            .fixture
            .file(&format!("{relative}/Contents/Info.plist"), 0)?;
        fs::write(info, description)?;
        self.fixture
            .file(&format!("{relative}/Contents/MacOS/program"), size)?;
        Ok(())
    }

    fn installed(&self, running: &[PathBuf]) -> Result<Vec<InstalledApp>, ApplicationError> {
        ListInstalledApps::new(
            &ReaddirWalker::default(),
            &LstatFileInspector,
            &PlistAppBundleReader::for_home(&self.home()),
            &Running(running.to_vec()),
        )
        .execute(
            &[self.applications(), self.home().join("Applications")],
            &NoProgress,
            &CancellationToken::new(),
        )
    }

    fn plan(
        &self,
        name: &str,
        installed: &[InstalledApp],
    ) -> Result<UninstallPlan, ApplicationError> {
        let app = installed
            .iter()
            .find(|app| app.name == name)
            .ok_or_else(|| ApplicationError::RootNotFound(PathBuf::from(name)))?;
        PlanUninstall::new(
            &HomeOnlyCatalog::for_home(&self.home())?,
            &ReaddirFolderLister,
            &ReaddirWalker::default(),
            &LstatFileInspector,
        )
        .execute(app, installed, &NoProgress, &CancellationToken::new())
    }

    /// The editor with everything it left behind, among apps and files that
    /// are not its own.
    fn with_editor() -> io::Result<Self> {
        let machine = Self::new()?;
        machine.app("Applications/Editor.app", "com.vendor.Editor", 9_000)?;
        machine.app(
            "Applications/Editor Beta.app",
            "com.vendor.Editor.beta",
            4_000,
        )?;
        machine.app("Applications/Suite/Viewer.app", "com.vendor.viewer", 2_000)?;
        for (relative, size) in [
            ("Application Support/Editor/documents.db", 6_000),
            ("Caches/com.vendor.Editor/blob.bin", 5_000),
            (
                "Group Containers/AB12CD34EF.com.vendor.Editor/shared.db",
                700,
            ),
            ("Preferences/com.vendor.Editor.plist", 300),
            ("Preferences/com.vendor.Editor.helper.plist", 200),
            (
                "Saved Application State/com.vendor.Editor.savedState/w",
                100,
            ),
            // Not the editor's:
            ("Preferences/com.vendor.Editor.beta.plist", 300),
            ("Preferences/com.vendor.EditorPro.plist", 300),
            ("Preferences/com.vendor.viewer.plist", 300),
            ("Application Support/Editor Backup/notes.txt", 50),
            ("Caches/com.apple.finder/cache.bin", 50),
        ] {
            machine
                .fixture
                .file(&format!("home/Library/{relative}"), size)?;
        }
        Ok(machine)
    }
}

fn facts(item: &UninstallItem, home: &Path) -> (String, LeftoverKind, RiskLevel) {
    let path = item.request.target.strip_prefix(home).map_or_else(
        |_| item.request.target.display().to_string(),
        |relative| format!("~/{}", relative.display()),
    );
    (path, item.kind, item.risk)
}

#[test]
fn lists_the_installed_apps_largest_first_and_leaves_out_what_is_not_an_app() {
    let machine = Machine::with_editor().unwrap();
    machine
        .fixture
        .file("Applications/Folder.app/readme.txt", 50_000)
        .unwrap();
    machine
        .fixture
        .symlink(
            &machine.applications().join("Editor.app"),
            "Applications/Linked.app",
        )
        .unwrap();
    let running = machine
        .applications()
        .join("Suite/Viewer.app/Contents/MacOS/program");

    let installed = machine.installed(&[running]).unwrap();

    let seen: Vec<(&str, &str, bool)> = installed
        .iter()
        .map(|app| (app.name.as_str(), app.bundle_id.as_str(), app.is_running))
        .collect();
    assert_eq!(
        seen,
        [
            ("Editor", "com.vendor.Editor", false),
            ("Editor Beta", "com.vendor.Editor.beta", false),
            ("Viewer", "com.vendor.viewer", true),
        ]
    );
    let editor = &installed[0];
    assert!(editor.size.as_bytes() >= 9_000);
    assert_eq!(editor.version.as_deref(), Some("1.0"));
    assert_eq!(editor.scope, PrivilegeScope::User);
    assert_eq!(editor.path, machine.applications().join("Editor.app"));
}

#[test]
fn the_plan_holds_everything_that_is_the_apps_and_nothing_of_another_app() {
    let machine = Machine::with_editor().unwrap();
    let installed = machine.installed(&[]).unwrap();

    let plan = machine.plan("Editor", &installed).unwrap();

    let found: Vec<_> = plan
        .items
        .iter()
        .map(|item| facts(item, &machine.home()))
        .collect();
    let (safe, review) = (RiskLevel::Safe, RiskLevel::Review);
    let bundle = machine.applications().join("Editor.app");
    let expected = [
        (bundle.to_str().unwrap(), LeftoverKind::Application, safe),
        (
            "~/Library/Application Support/Editor",
            LeftoverKind::Support,
            review,
        ),
        (
            "~/Library/Group Containers/AB12CD34EF.com.vendor.Editor",
            LeftoverKind::Container,
            safe,
        ),
        (
            "~/Library/Caches/com.vendor.Editor",
            LeftoverKind::Caches,
            safe,
        ),
        (
            "~/Library/Preferences/com.vendor.Editor.helper.plist",
            LeftoverKind::Preferences,
            review,
        ),
        (
            "~/Library/Preferences/com.vendor.Editor.plist",
            LeftoverKind::Preferences,
            safe,
        ),
        (
            "~/Library/Saved Application State/com.vendor.Editor.savedState",
            LeftoverKind::SavedState,
            safe,
        ),
    ]
    .map(|(path, kind, risk)| (path.to_owned(), kind, risk));
    assert_eq!(found, expected);

    assert!(!plan.has_other_copy);
    assert!(
        plan.items
            .iter()
            .all(|item| item.request.method == DeletionMethod::Trash),
        "what the user owns goes to the Trash"
    );
    let caches = &plan.items[3].request;
    assert!(
        caches.reclaimable.as_bytes() >= 5_000,
        "a folder counts with its content"
    );
    assert!(plan.reclaimable().as_bytes() >= 9_000 + 6_000 + 5_000);
}

#[test]
fn uninstalling_removes_the_selected_items_and_leaves_everything_else() {
    let machine = Machine::with_editor().unwrap();
    let installed = machine.installed(&[]).unwrap();
    let plan = machine.plan("Editor", &installed).unwrap();
    let selected: Vec<DeletionRequest> = plan
        .items
        .iter()
        .filter(|item| item.is_default_choice())
        .map(|item| item.request.clone())
        .collect();

    let home = machine.home();
    let trash = FolderTrash(machine.root.join("trash"));
    let history = JsonlHistoryRepository::new(machine.root.join("history.jsonl"));
    let policy = ProtectedPaths::new(&home);
    let ports = DeletionPorts {
        inspector: &LstatFileInspector,
        remover: &SafeFileRemover,
        trash_bin: &trash,
        history: &history,
    };
    let report =
        DeleteItems::new(&policy, ports).execute(&selected, &plan.roots, ExecutionMode::Real);

    assert_eq!(report.outcomes.len(), 5);
    assert!(
        report.outcomes.iter().all(DeletionOutcome::is_removed),
        "{:?}",
        report.outcomes
    );
    assert!(report.reclaimed.is_zero(), "nothing is removed for good");
    assert!(report.trashed.as_bytes() >= 9_000 + 5_000);

    let library = home.join("Library");
    for gone in [
        machine.applications().join("Editor.app"),
        library.join("Caches/com.vendor.Editor"),
        library.join("Group Containers/AB12CD34EF.com.vendor.Editor"),
        library.join("Preferences/com.vendor.Editor.plist"),
        library.join("Saved Application State/com.vendor.Editor.savedState"),
    ] {
        assert!(!gone.exists(), "{}", gone.display());
    }
    for kept in [
        machine.applications().join("Editor Beta.app"),
        machine.applications().join("Suite/Viewer.app"),
        library.join("Application Support/Editor/documents.db"),
        library.join("Application Support/Editor Backup/notes.txt"),
        library.join("Preferences/com.vendor.Editor.helper.plist"),
        library.join("Preferences/com.vendor.Editor.beta.plist"),
        library.join("Preferences/com.vendor.EditorPro.plist"),
        library.join("Preferences/com.vendor.viewer.plist"),
        library.join("Caches/com.apple.finder/cache.bin"),
        library.join("Caches"),
        library.join("Preferences"),
    ] {
        assert!(kept.exists(), "{}", kept.display());
    }
    assert!(
        machine
            .root
            .join("trash/Editor.app/Contents/Info.plist")
            .exists()
    );
}

#[test]
fn with_another_copy_installed_only_the_bundle_is_offered() {
    let machine = Machine::with_editor().unwrap();
    machine
        .app("home/Applications/Editor.app", "com.vendor.editor", 1_000)
        .unwrap();
    let installed = machine.installed(&[]).unwrap();
    assert_eq!(installed.len(), 4);

    let plan = machine.plan("Editor", &installed).unwrap();

    assert!(plan.has_other_copy);
    assert_eq!(plan.items.len(), 1);
    assert_eq!(plan.items[0].kind, LeftoverKind::Application);
}

#[test]
fn an_app_that_is_gone_cannot_be_planned() {
    let machine = Machine::with_editor().unwrap();
    let installed = machine.installed(&[]).unwrap();
    fs::remove_dir_all(machine.applications().join("Editor.app")).unwrap();
    let editor = installed.iter().find(|app| app.name == "Editor").unwrap();

    let result = PlanUninstall::new(
        &HomeOnlyCatalog::for_home(&machine.home()).unwrap(),
        &ReaddirFolderLister,
        &ReaddirWalker::default(),
        &LstatFileInspector,
    )
    .execute(editor, &installed, &NoProgress, &CancellationToken::new());

    assert!(matches!(result, Err(ApplicationError::RootNotFound(_))));
}

/// Two folders of the fixture: one only an administrator may change, and
/// the library of the user, which an administrator removal never touches.
struct TwoScopes(Vec<LeftoverLocation>);

impl TwoScopes {
    fn for_machine(machine: &Machine) -> Self {
        let location = |folder: PathBuf, kind, scope| LeftoverLocation {
            folder,
            kind,
            scope,
            is_matched_by_name: false,
        };
        Self(vec![
            location(
                machine.root.join("SystemLibrary/LaunchDaemons"),
                LeftoverKind::LaunchItem,
                PrivilegeScope::Administrator,
            ),
            location(
                machine.home().join("Library/Caches"),
                LeftoverKind::Caches,
                PrivilegeScope::User,
            ),
        ])
    }
}

impl RuleCatalog for TwoScopes {
    fn cleanup_rules(&self) -> Result<Vec<CleanupRule>, ApplicationError> {
        Ok(Vec::new())
    }

    fn artifact_rules(&self) -> Result<Vec<ArtifactRule>, ApplicationError> {
        Ok(Vec::new())
    }

    fn leftover_locations(&self) -> Result<Vec<LeftoverLocation>, ApplicationError> {
        Ok(self.0.clone())
    }
}

/// What the privileged helper does for an uninstall, here without any
/// privilege and inside the fixture.
fn remove_as_administrator(
    machine: &Machine,
    bundle_id: &str,
    items: &[PathBuf],
    running: &[PathBuf],
) -> Result<DeletionReport, ApplicationError> {
    let (walker, inspector) = (ReaddirWalker::default(), LstatFileInspector);
    let catalog = TwoScopes::for_machine(machine);
    let policy = ProtectedPaths::new(&machine.home());
    let history = JsonlHistoryRepository::new(machine.root.join("history.jsonl"));
    let ports = DeletionPorts {
        inspector: &inspector,
        remover: &SafeFileRemover,
        trash_bin: &FolderTrash(machine.root.join("trash")),
        history: &history,
    };
    RemoveAppItems::new(
        ListInstalledApps::new(
            &walker,
            &inspector,
            &PlistAppBundleReader::for_home(&machine.home()),
            &Running(running.to_vec()),
        ),
        PlanUninstall::new(&catalog, &ReaddirFolderLister, &walker, &inspector),
        DeleteItems::new(&policy, ports),
    )
    .execute(
        bundle_id,
        items,
        &[machine.applications()],
        ExecutionMode::Real,
    )
}

#[test]
fn an_administrator_removal_takes_only_what_its_own_analysis_finds_for_the_app() {
    let machine = Machine::with_editor().unwrap();
    let daemon = machine
        .fixture
        .file("SystemLibrary/LaunchDaemons/com.vendor.Editor.plist", 400)
        .unwrap();
    let daemon = fs::canonicalize(daemon).unwrap();
    let other_daemon = machine
        .fixture
        .file("SystemLibrary/LaunchDaemons/com.vendor.viewer.plist", 400)
        .unwrap();
    let bundle = machine.applications().join("Editor.app");
    let caches = machine.home().join("Library/Caches/com.vendor.Editor");

    // Anything that is not the app's own, at administrator scope, stops the
    // whole removal before a single item is touched.
    for stranger in [
        fs::canonicalize(&other_daemon).unwrap(),
        caches.clone(),
        machine.applications().join("Editor Beta.app"),
        machine.home().join("Library/Preferences"),
    ] {
        let result = remove_as_administrator(
            &machine,
            "com.vendor.Editor",
            &[bundle.clone(), stranger.clone()],
            &[],
        );
        assert!(
            matches!(result, Err(ApplicationError::ItemNotAllowed(_))),
            "{}",
            stranger.display()
        );
        assert!(bundle.exists() && stranger.exists());
    }

    let report = remove_as_administrator(
        &machine,
        "com.vendor.editor",
        &[bundle.clone(), daemon.clone()],
        &[],
    )
    .unwrap();

    assert!(report.outcomes.iter().all(DeletionOutcome::is_removed));
    assert!(
        report.trashed.is_zero() && report.reclaimed.as_bytes() >= 9_000,
        "removed for good: an administrator has no Trash of the user"
    );
    assert!(!bundle.exists() && !daemon.exists());
    assert!(other_daemon.exists() && caches.exists());
    assert!(!machine.root.join("trash").exists());
}

#[test]
fn an_administrator_removal_refuses_an_app_it_cannot_tell_apart_or_that_is_open() {
    let machine = Machine::with_editor().unwrap();
    let bundle = machine.applications().join("Editor.app");
    let remove = |bundle_id: &str, running: &[PathBuf]| {
        remove_as_administrator(&machine, bundle_id, std::slice::from_ref(&bundle), running)
    };

    assert!(matches!(
        remove("com.vendor.missing", &[]),
        Err(ApplicationError::AppNotFound(_))
    ));
    let program = bundle.join("Contents/MacOS/program");
    assert!(matches!(
        remove("com.vendor.Editor", &[program]),
        Err(ApplicationError::AppRunning(_))
    ));
    machine
        .app("Applications/Suite/Editor.app", "com.vendor.Editor", 100)
        .unwrap();
    assert!(
        matches!(
            remove("com.vendor.Editor", &[]),
            Err(ApplicationError::AppNotFound(_))
        ),
        "two copies: it is not known which one is meant"
    );
    assert!(bundle.exists());
}

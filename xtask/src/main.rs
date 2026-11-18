//! Project automation, run as `cargo xtask <task>`.
//!
//! - `check` is the quality gate: the same steps run locally and in CI, and a
//!   change is not done while any of them fails.
//! - `bundle` builds `dist/MacCleaner.app`, a regular macOS application.
//! - `install` builds the app and copies it to `/Applications`, or to the
//!   folder given as an argument (for example `~/Applications`).

#![forbid(unsafe_code)]

use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// The quality gate, in the order it runs: name and `cargo` arguments.
const CHECK_STEPS: [(&str, &[&str]); 4] = [
    ("format", &["fmt", "--all", "--", "--check"]),
    (
        "lint",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    ),
    ("test", &["test", "--workspace"]),
    ("audit", &["deny", "check"]),
];

const APP_BUNDLE: &str = "MacCleaner.app";
const APP_EXECUTABLE: &str = "maccleaner";
/// The program that cleans system areas as an administrator. It sits beside
/// the app's own executable, where the app looks for it.
const HELPER_EXECUTABLE: &str = "maccleaner-privileged-helper";
const INSTALL_DIRECTORY: &str = "/Applications";
/// Environment variable naming the code-signing identity to use. Without it
/// the app is signed ad hoc, which is enough to run it on this Mac.
const SIGNING_IDENTITY_VARIABLE: &str = "MACCLEANER_SIGNING_IDENTITY";
const AD_HOC_IDENTITY: &str = "-";
const USAGE: &str = "usage: cargo xtask <check | bundle | install [folder]>

  check             format, lint, test and audit the workspace
  bundle            build dist/MacCleaner.app
  install [folder]  build the app and copy it to /Applications, or to `folder`";

type TaskResult = Result<(), String>;

fn main() -> ExitCode {
    let result = match env::args().nth(1).as_deref() {
        Some("check") => check(),
        Some("bundle") => bundle().map(|app| eprintln!("xtask: built {}", app.display())),
        Some("install") => install(env::args().nth(2).as_deref().unwrap_or(INSTALL_DIRECTORY)),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("xtask: {message}");
            ExitCode::FAILURE
        }
    }
}

fn check() -> TaskResult {
    for (name, arguments) in CHECK_STEPS {
        eprintln!("==> {name}: cargo {}", arguments.join(" "));
        cargo(arguments).map_err(|_| format!("step `{name}` failed"))?;
    }
    eprintln!("xtask: all checks passed");
    Ok(())
}

/// Builds the release binary and assembles the application bundle around it.
fn bundle() -> Result<PathBuf, String> {
    let root = workspace_root();
    cargo(&[
        "build",
        "--release",
        "--package",
        APP_EXECUTABLE,
        "--package",
        HELPER_EXECUTABLE,
    ])?;

    let app = root.join("dist").join(APP_BUNDLE);
    if app.exists() {
        fs::remove_dir_all(&app).map_err(|error| describe("remove the previous bundle", &error))?;
    }
    let contents = app.join("Contents");
    let executables = contents.join("MacOS");
    let resources = contents.join("Resources");
    for directory in [&executables, &resources] {
        fs::create_dir_all(directory).map_err(|error| describe("create the bundle", &error))?;
    }

    let assets = root.join("apps").join("desktop").join("bundle");
    let built = target_directory(&root).join("release");
    for executable in [APP_EXECUTABLE, HELPER_EXECUTABLE] {
        fs::copy(built.join(executable), executables.join(executable))
            .map_err(|error| describe("copy the executables", &error))?;
    }
    fs::copy(assets.join("AppIcon.icns"), resources.join("AppIcon.icns"))
        .map_err(|error| describe("copy the icon", &error))?;
    let information = fs::read_to_string(assets.join("Info.plist"))
        .map_err(|error| describe("read Info.plist", &error))?
        .replace("{{VERSION}}", env!("CARGO_PKG_VERSION"));
    fs::write(contents.join("Info.plist"), information)
        .map_err(|error| describe("write Info.plist", &error))?;

    // Code inside the bundle is signed first: the signature of the bundle
    // seals everything it contains.
    sign(&executables.join(HELPER_EXECUTABLE))?;
    sign(&app)?;
    Ok(app)
}

/// Signs the bundle, or a program inside it, so macOS agrees to run it and
/// can remember its permissions.
fn sign(app: &Path) -> TaskResult {
    let identity =
        env::var(SIGNING_IDENTITY_VARIABLE).unwrap_or_else(|_| AD_HOC_IDENTITY.to_owned());
    run(
        Command::new("/usr/bin/codesign")
            .args(["--force", "--options", "runtime", "--sign", &identity])
            .arg(app),
        "sign the app",
    )
}

fn install(directory: &str) -> TaskResult {
    let app = bundle()?;
    let destination = Path::new(directory).join(APP_BUNDLE);
    if destination.exists() {
        fs::remove_dir_all(&destination)
            .map_err(|error| describe("replace the installed app", &error))?;
    }
    // `ditto` copies a bundle with its signature and attributes intact.
    run(
        Command::new("/usr/bin/ditto").arg(&app).arg(&destination),
        "copy the app to the destination folder",
    )?;
    eprintln!("xtask: installed {}", destination.display());
    eprintln!("xtask: open it from Launchpad, Spotlight or the Applications folder");
    Ok(())
}

fn cargo(arguments: &[&str]) -> TaskResult {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    run(
        Command::new(cargo)
            .args(arguments)
            .current_dir(workspace_root()),
        "run cargo",
    )
}

fn run(command: &mut Command, purpose: &str) -> TaskResult {
    match command.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(format!("could not {purpose}: {status}")),
        Err(error) => Err(describe(purpose, &error)),
    }
}

fn describe(purpose: &str, error: &dyn std::fmt::Display) -> String {
    format!("could not {purpose}: {error}")
}

/// The workspace folder: the parent of this crate's own folder.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

/// Where cargo puts build output, honoring `CARGO_TARGET_DIR`.
fn target_directory(root: &Path) -> PathBuf {
    env::var_os("CARGO_TARGET_DIR")
        .filter(|value| !value.is_empty())
        .map_or_else(|| root.join("target"), |value| root.join::<&OsStr>(&value))
}

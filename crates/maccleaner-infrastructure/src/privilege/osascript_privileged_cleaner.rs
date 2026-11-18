//! Privileged cleaner that asks macOS to authenticate the user and to run
//! the helper once, through `osascript`.

use std::env;
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use maccleaner_application::ports::{PrivilegeError, PrivilegedCleaner};
use maccleaner_domain::deletion::DeletionOutcome;

use crate::privilege::helper_protocol::{
    HELPER_NAME, UNINSTALL_FLAG, decode_outcomes, is_bundle_id, is_item_path, is_rule_id,
};

/// Absolute paths, so a different program earlier in `PATH` can never run.
const OSASCRIPT: &str = "/usr/bin/osascript";
const CODESIGN: &str = "/usr/bin/codesign";

/// The whole script, one statement per line. It never changes: the dialog
/// text, the helper and what the helper is asked arrive as arguments, and
/// each word of the command is quoted by AppleScript itself before the shell
/// sees it.
const SCRIPT: [&str; 7] = [
    "on run argv",
    "set command to \"\"",
    "repeat with part in (rest of argv)",
    "set command to command & (quoted form of (part as text)) & \" \"",
    "end repeat",
    "do shell script command with prompt (item 1 of argv) with administrator privileges",
    "end run",
];
/// What `osascript` reports when the user closes the authentication dialog.
const USER_CANCELLED: &str = "(-128)";
/// Folders of an app bundle between the bundle and its executables.
const BUNDLE_EXECUTABLES: [&str; 2] = ["Contents", "MacOS"];
/// Permission bits that would let someone other than the owner change a file.
const WRITABLE_BY_OTHERS: u32 = 0o022;
const EXECUTABLE_BY_OWNER: u32 = 0o100;

/// Runs the helper as an administrator, once per cleanup.
///
/// macOS shows its own dialog and collects the password or the fingerprint;
/// this process never sees either. There is no service left running.
#[derive(Debug, Clone)]
pub struct OsascriptPrivilegedCleaner {
    helper: Option<PathBuf>,
}

impl OsascriptPrivilegedCleaner {
    /// Looks for the helper beside the running executable. Without it the
    /// cleaner reports itself unavailable.
    #[must_use]
    pub fn beside_current_executable() -> Self {
        let helper = env::current_exe()
            .ok()
            .and_then(|executable| executable.canonicalize().ok())
            .and_then(|executable| Some(executable.parent()?.join(HELPER_NAME)));
        Self::new(helper)
    }

    /// Uses the helper at `helper`, when it is there and can be trusted.
    #[must_use]
    pub fn new(helper: Option<PathBuf>) -> Self {
        Self {
            helper: helper.filter(|path| is_unmodifiable_program(path)),
        }
    }
}

impl OsascriptPrivilegedCleaner {
    /// Asks macOS to authenticate the user and to run the helper once with
    /// `arguments`, and returns what the helper reports.
    fn run_helper(
        &self,
        arguments: &[OsString],
        prompt: &str,
    ) -> Result<Vec<DeletionOutcome>, PrivilegeError> {
        let unavailable = |reason: &str| PrivilegeError::Unavailable(reason.to_owned());
        let helper = self
            .helper
            .as_ref()
            .ok_or_else(|| unavailable("helper not found"))?;
        // Checked again right before use: time passed since the app started.
        if !is_unmodifiable_program(helper) {
            return Err(unavailable("helper changed"));
        }
        if !has_valid_signature(helper) {
            return Err(unavailable("the app's signature is not valid"));
        }

        let mut command = Command::new(OSASCRIPT);
        for line in SCRIPT {
            command.args(["-e", line]);
        }
        let output = command
            .arg(prompt)
            .arg(helper)
            .args(arguments)
            .env_clear()
            .stdin(Stdio::null())
            .output()
            .map_err(|error| PrivilegeError::Failed(error.to_string()))?;

        if !output.status.success() {
            let message = String::from_utf8_lossy(&output.stderr);
            return Err(if message.contains(USER_CANCELLED) {
                PrivilegeError::Cancelled
            } else {
                PrivilegeError::Failed(message.trim().to_owned())
            });
        }
        decode_outcomes(&String::from_utf8_lossy(&output.stdout)).map_err(PrivilegeError::Failed)
    }
}

impl PrivilegedCleaner for OsascriptPrivilegedCleaner {
    fn is_available(&self) -> bool {
        self.helper.is_some()
    }

    fn clean(
        &self,
        rule_ids: &[String],
        prompt: &str,
    ) -> Result<Vec<DeletionOutcome>, PrivilegeError> {
        // Checked before the user is asked to authenticate for nothing.
        if rule_ids.is_empty() || !rule_ids.iter().all(|id| is_rule_id(id)) {
            return Err(PrivilegeError::Failed("invalid rule id".to_owned()));
        }
        let arguments: Vec<OsString> = rule_ids.iter().map(OsString::from).collect();
        self.run_helper(&arguments, prompt)
    }

    fn uninstall(
        &self,
        bundle_id: &str,
        items: &[PathBuf],
        prompt: &str,
    ) -> Result<Vec<DeletionOutcome>, PrivilegeError> {
        let are_items_valid = !items.is_empty()
            && items
                .iter()
                .all(|item| item.to_str().is_some_and(is_item_path));
        if !(is_bundle_id(bundle_id) && are_items_valid) {
            return Err(PrivilegeError::Failed(
                "invalid uninstall request".to_owned(),
            ));
        }
        let arguments: Vec<OsString> = [OsString::from(UNINSTALL_FLAG), OsString::from(bundle_id)]
            .into_iter()
            .chain(items.iter().map(OsString::from))
            .collect();
        self.run_helper(&arguments, prompt)
    }
}

/// Whether `path` is a real program file that only its owner can change.
/// A link is refused: what it points to could be swapped.
fn is_unmodifiable_program(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| {
        metadata.file_type().is_file()
            && metadata.mode() & WRITABLE_BY_OTHERS == 0
            && metadata.mode() & EXECUTABLE_BY_OWNER != 0
    })
}

/// When the helper is inside an app bundle, asks macOS whether the bundle is
/// still exactly what was signed. Outside a bundle, as in a development
/// build, there is no signature to check.
fn has_valid_signature(helper: &Path) -> bool {
    let Some(bundle) = bundle_of(helper) else {
        return true;
    };
    Command::new(CODESIGN)
        .args(["--verify", "--strict"])
        .arg(bundle)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// The app bundle whose `Contents/MacOS` folder holds `program`, if any.
fn bundle_of(program: &Path) -> Option<&Path> {
    let mut folder = program.parent()?;
    for expected in BUNDLE_EXECUTABLES.iter().rev() {
        if folder.file_name()?.to_str()? != *expected {
            return None;
        }
        folder = folder.parent()?;
    }
    folder
        .extension()
        .is_some_and(|extension| extension == "app")
        .then_some(folder)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};

    use maccleaner_application::ports::{PrivilegeError, PrivilegedCleaner};
    use maccleaner_test_support::TempTree;

    use super::{OsascriptPrivilegedCleaner, bundle_of};

    fn program(fixture: &TempTree, relative: &str, mode: u32) -> std::path::PathBuf {
        let path = fixture.file(relative, 16).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
        path
    }

    #[test]
    fn is_unavailable_without_a_helper() {
        let fixture = TempTree::new().unwrap();
        let cleaner = OsascriptPrivilegedCleaner::new(Some(fixture.path("missing")));
        assert!(!cleaner.is_available());
        assert!(matches!(
            cleaner.clean(&["system-caches".to_owned()], "prompt"),
            Err(PrivilegeError::Unavailable(_))
        ));
        assert!(!OsascriptPrivilegedCleaner::new(None).is_available());
    }

    #[test]
    fn refuses_a_helper_that_others_can_change_or_that_is_a_link() {
        let fixture = TempTree::new().unwrap();
        let good = program(&fixture, "good", 0o755);
        let writable = program(&fixture, "writable", 0o777);
        let plain = program(&fixture, "plain", 0o644);
        let link = fixture.symlink(&good, "link").unwrap();

        assert!(OsascriptPrivilegedCleaner::new(Some(good)).is_available());
        assert!(!OsascriptPrivilegedCleaner::new(Some(writable)).is_available());
        assert!(
            !OsascriptPrivilegedCleaner::new(Some(plain)).is_available(),
            "not a program"
        );
        assert!(!OsascriptPrivilegedCleaner::new(Some(link)).is_available());
    }

    #[test]
    fn refuses_rule_ids_before_asking_for_authentication() {
        let fixture = TempTree::new().unwrap();
        let cleaner = OsascriptPrivilegedCleaner::new(Some(program(&fixture, "helper", 0o755)));

        for ids in [
            vec![],
            vec!["/Library/Caches".to_owned()],
            vec!["ok".to_owned(), "; rm".to_owned()],
        ] {
            assert!(matches!(
                cleaner.clean(&ids, "prompt"),
                Err(PrivilegeError::Failed(_))
            ));
        }
    }

    #[test]
    fn refuses_an_uninstall_before_asking_for_authentication() {
        let fixture = TempTree::new().unwrap();
        let cleaner = OsascriptPrivilegedCleaner::new(Some(program(&fixture, "helper", 0o755)));
        let app = PathBuf::from("/Applications/Editor.app");

        for (bundle_id, items) in [
            ("com.vendor.editor", vec![]),
            ("--force", vec![app.clone()]),
            (
                "com.vendor.editor",
                vec![PathBuf::from("relative/Editor.app")],
            ),
            (
                "com.vendor.editor",
                vec![app, PathBuf::from("/Library/../etc")],
            ),
        ] {
            assert!(
                matches!(
                    cleaner.uninstall(bundle_id, &items, "prompt"),
                    Err(PrivilegeError::Failed(_))
                ),
                "{bundle_id} {items:?}"
            );
        }
        assert!(matches!(
            OsascriptPrivilegedCleaner::new(None).uninstall(
                "com.vendor.editor",
                &[PathBuf::from("/Applications/Editor.app")],
                "prompt"
            ),
            Err(PrivilegeError::Unavailable(_))
        ));
    }

    #[test]
    fn finds_the_bundle_only_from_its_executables_folder() {
        let inside = Path::new("/Applications/MacCleaner.app/Contents/MacOS/helper");
        assert_eq!(
            bundle_of(inside),
            Some(Path::new("/Applications/MacCleaner.app"))
        );
        assert_eq!(bundle_of(Path::new("/work/target/debug/helper")), None);
        assert_eq!(
            bundle_of(Path::new("/x/Folder/Contents/MacOS/helper")),
            None
        );
    }
}

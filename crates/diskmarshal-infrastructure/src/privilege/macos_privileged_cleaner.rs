//! Privileged cleaner that asks macOS to authenticate the user and to run
//! the helper once.
//!
//! There are two ways to get there. Where the user turned on Touch ID for
//! `sudo`, the helper is started through `sudo` and macOS takes the
//! fingerprint. Everywhere else, and whenever the fingerprint does not go
//! through, it is started through AppleScript, whose dialog takes the
//! password.

use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

use diskmarshal_application::ports::{AuthenticationKind, PrivilegeError, PrivilegedCleaner};
use diskmarshal_domain::deletion::DeletionOutcome;

use crate::privilege::helper_program::{has_valid_signature, is_unmodifiable_program};
use crate::privilege::helper_protocol::{
    HELPER_NAME, UNINSTALL_FLAG, decode_outcomes, is_bundle_id, is_item_path, is_rule_id,
};
use crate::privilege::sudo_launch::SudoRun;
use crate::privilege::{osascript_launch, sudo_launch};

/// Runs the helper as an administrator, once per cleanup.
///
/// macOS shows its own dialog and collects the fingerprint or the password;
/// this process never sees either. There is no service left running.
#[derive(Debug, Clone)]
pub struct MacosPrivilegedCleaner {
    helper: Option<PathBuf>,
}

impl MacosPrivilegedCleaner {
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

        let with_fingerprint =
            sudo_launch::accepts_touch_id().then(|| sudo_launch::run(helper, arguments));
        let report = match with_fingerprint {
            Some(SudoRun::Finished(result)) => result?,
            // The fingerprint did not go through, or this Mac does not take
            // it for `sudo`: the password dialog is the way that always exists.
            Some(SudoRun::NotAuthenticated) | None => {
                osascript_launch::run(helper, arguments, prompt)?
            }
        };
        decode_outcomes(&report).map_err(PrivilegeError::Failed)
    }
}

impl PrivilegedCleaner for MacosPrivilegedCleaner {
    fn is_available(&self) -> bool {
        self.helper.is_some()
    }

    fn authentication(&self) -> AuthenticationKind {
        if sudo_launch::accepts_touch_id() {
            AuthenticationKind::TouchId
        } else {
            AuthenticationKind::Password
        }
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    use diskmarshal_application::ports::{PrivilegeError, PrivilegedCleaner};
    use diskmarshal_test_support::TempTree;

    use super::MacosPrivilegedCleaner;

    fn program(fixture: &TempTree, relative: &str, mode: u32) -> std::path::PathBuf {
        let path = fixture.file(relative, 16).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
        path
    }

    #[test]
    fn is_unavailable_without_a_helper() {
        let fixture = TempTree::new().unwrap();
        let cleaner = MacosPrivilegedCleaner::new(Some(fixture.path("missing")));
        assert!(!cleaner.is_available());
        assert!(matches!(
            cleaner.clean(&["system-caches".to_owned()], "prompt"),
            Err(PrivilegeError::Unavailable(_))
        ));
        assert!(!MacosPrivilegedCleaner::new(None).is_available());
    }

    #[test]
    fn refuses_a_helper_that_others_can_change_or_that_is_a_link() {
        let fixture = TempTree::new().unwrap();
        let good = program(&fixture, "good", 0o755);
        let writable = program(&fixture, "writable", 0o777);
        let plain = program(&fixture, "plain", 0o644);
        let link = fixture.symlink(&good, "link").unwrap();

        assert!(MacosPrivilegedCleaner::new(Some(good)).is_available());
        assert!(!MacosPrivilegedCleaner::new(Some(writable)).is_available());
        assert!(
            !MacosPrivilegedCleaner::new(Some(plain)).is_available(),
            "not a program"
        );
        assert!(!MacosPrivilegedCleaner::new(Some(link)).is_available());
    }

    #[test]
    fn refuses_rule_ids_before_asking_for_authentication() {
        let fixture = TempTree::new().unwrap();
        let cleaner = MacosPrivilegedCleaner::new(Some(program(&fixture, "helper", 0o755)));

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
        let cleaner = MacosPrivilegedCleaner::new(Some(program(&fixture, "helper", 0o755)));
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
            MacosPrivilegedCleaner::new(None).uninstall(
                "com.vendor.editor",
                &[PathBuf::from("/Applications/Editor.app")],
                "prompt"
            ),
            Err(PrivilegeError::Unavailable(_))
        ));
    }
}

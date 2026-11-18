//! Asking macOS to authenticate the user and to run the helper once.
//!
//! There are two ways to get there. Where the user turned on Touch ID for
//! `sudo`, the helper is started through `sudo` and macOS takes the
//! fingerprint. Everywhere else, and whenever the fingerprint does not go
//! through, it is started through AppleScript, whose dialog takes the
//! password.

use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

use diskmarshal_application::ports::{AuthenticationKind, PrivilegeError};

use crate::privilege::helper_program::{has_valid_signature, is_unmodifiable_program};
use crate::privilege::helper_protocol::HELPER_NAME;
use crate::privilege::sudo_launch::SudoRun;
use crate::privilege::{osascript_launch, sudo_launch};

/// Runs the helper as an administrator, once per request.
///
/// macOS shows its own dialog and collects the fingerprint or the password;
/// this process never sees either. There is no service left running.
#[derive(Debug, Clone)]
pub(crate) struct HelperLauncher {
    helper: Option<PathBuf>,
}

impl HelperLauncher {
    /// Looks for the helper beside the running executable. Without it the
    /// launcher reports itself unavailable.
    pub(crate) fn beside_current_executable() -> Self {
        let helper = env::current_exe()
            .ok()
            .and_then(|executable| executable.canonicalize().ok())
            .and_then(|executable| Some(executable.parent()?.join(HELPER_NAME)));
        Self::new(helper)
    }

    /// Uses the helper at `helper`, when it is there and can be trusted.
    pub(crate) fn new(helper: Option<PathBuf>) -> Self {
        Self {
            helper: helper.filter(|path| is_unmodifiable_program(path)),
        }
    }

    /// Whether there is a helper to run.
    pub(crate) fn is_available(&self) -> bool {
        self.helper.is_some()
    }

    /// What macOS will ask the user for.
    pub(crate) fn authentication(&self) -> AuthenticationKind {
        if sudo_launch::accepts_touch_id() {
            AuthenticationKind::TouchId
        } else {
            AuthenticationKind::Password
        }
    }

    /// Asks macOS to authenticate the user, showing `prompt` in its password
    /// dialog, and to run the helper once with `arguments`. Returns what the
    /// helper printed.
    pub(crate) fn run(
        &self,
        arguments: &[OsString],
        prompt: &str,
    ) -> Result<String, PrivilegeError> {
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
        match with_fingerprint {
            Some(SudoRun::Finished(result)) => result,
            // The fingerprint did not go through, or this Mac does not take
            // it for `sudo`: the password dialog is the way that always exists.
            Some(SudoRun::NotAuthenticated) | None => {
                osascript_launch::run(helper, arguments, prompt)
            }
        }
    }
}

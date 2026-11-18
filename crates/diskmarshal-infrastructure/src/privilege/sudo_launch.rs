//! Starts the helper as an administrator through `sudo`, which is where
//! macOS can take the fingerprint instead of the password.
//!
//! Touch ID for `sudo` is a setting of the Mac, off by default: one line in
//! `/etc/pam.d/sudo_local`. The app never changes it. When the user has
//! turned it on, this module is tried first; when the fingerprint is
//! refused, cancelled or not available, the caller falls back to the
//! password dialog.

use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use diskmarshal_application::ports::PrivilegeError;

/// Absolute path, so a different program earlier in `PATH` can never run.
const SUDO: &str = "/usr/bin/sudo";
/// Where macOS configures how `sudo` authenticates. The first one is the
/// file meant for local changes, which the second includes.
const PAM_FILES: [&str; 2] = ["/etc/pam.d/sudo_local", "/etc/pam.d/sudo"];
/// The authentication module of Touch ID.
const TOUCH_ID_MODULE: &str = "pam_tid.so";
/// First word of the lines of a PAM file that decide who the user is.
const AUTHENTICATION_LINE: &str = "auth";
/// How `sudo` starts every message of its own on the error output.
const SUDO_MESSAGE: &str = "sudo:";

/// What happened when the helper was started through `sudo`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SudoRun {
    /// The helper ran. This is what it printed, or why it failed.
    Finished(Result<String, PrivilegeError>),
    /// `sudo` did not authenticate the user: the fingerprint was refused or
    /// cancelled, or Touch ID cannot be used right now.
    NotAuthenticated,
}

/// Whether `sudo` on this Mac is set up to take the fingerprint.
pub(crate) fn accepts_touch_id() -> bool {
    PAM_FILES
        .iter()
        .any(|file| fs::read_to_string(file).is_ok_and(|text| enables_touch_id(&text)))
}

/// Whether a PAM configuration has an active line that authenticates with
/// Touch ID. A commented line starts with `#`, so it is not an `auth` line.
fn enables_touch_id(configuration: &str) -> bool {
    configuration.lines().any(|line| {
        let mut words = line.split_whitespace();
        words.next() == Some(AUTHENTICATION_LINE) && words.any(|word| word == TOUCH_ID_MODULE)
    })
}

/// Runs `helper` once with `arguments` as an administrator, after `sudo`
/// authenticates the user.
pub(crate) fn run(helper: &Path, arguments: &[OsString]) -> SudoRun {
    let output = Command::new(SUDO)
        // -S: a password would be read from the standard input, which is
        //     closed here, so `sudo` can never sit waiting for one to be
        //     typed at a terminal: without the fingerprint it just fails.
        // -k: an authentication made earlier is never reused.
        // -p: no prompt text; nobody would see it.
        .args(["-S", "-k", "-p", "", "--"])
        .arg(helper)
        .args(arguments)
        .env_clear()
        .stdin(Stdio::null())
        .output();
    match output {
        Ok(output) => interpret(output.status.success(), &output.stdout, &output.stderr),
        Err(error) => SudoRun::Finished(Err(PrivilegeError::Failed(error.to_string()))),
    }
}

/// Tells a run that never got past `sudo` from one where the helper ran.
fn interpret(succeeded: bool, printed: &[u8], errors: &[u8]) -> SudoRun {
    if succeeded {
        return SudoRun::Finished(Ok(String::from_utf8_lossy(printed).into_owned()));
    }
    let errors = String::from_utf8_lossy(errors);
    if errors
        .lines()
        .any(|line| line.trim_start().starts_with(SUDO_MESSAGE))
    {
        return SudoRun::NotAuthenticated;
    }
    let reason = errors
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .unwrap_or_default();
    SudoRun::Finished(Err(PrivilegeError::Failed(reason.trim().to_owned())))
}

#[cfg(test)]
mod tests {
    use diskmarshal_application::ports::PrivilegeError;

    use super::{SudoRun, enables_touch_id, interpret};

    /// The file macOS ships as a template, with its line still commented.
    const TEMPLATE: &str = "# sudo_local: local config file which survives system update and is included for sudo\n\
# uncomment following line to enable Touch ID for sudo\n\
#auth       sufficient     pam_tid.so\n";

    #[test]
    fn touch_id_is_off_while_its_line_is_commented() {
        assert!(!enables_touch_id(TEMPLATE));
        assert!(!enables_touch_id(""));
        assert!(!enables_touch_id("auth       include        sudo_local\n"));
        assert!(!enables_touch_id("  # auth sufficient pam_tid.so\n"));
    }

    #[test]
    fn touch_id_is_on_with_an_active_line_however_it_is_spaced() {
        assert!(enables_touch_id(&TEMPLATE.replace("#auth", "auth")));
        assert!(enables_touch_id("auth\tsufficient\tpam_tid.so\n"));
        assert!(enables_touch_id(
            "auth sufficient pam_smartcard.so\n  auth sufficient pam_tid.so\n"
        ));
        assert!(
            !enables_touch_id("session required pam_tid.so\n"),
            "not an authentication line"
        );
    }

    #[test]
    fn a_run_that_succeeds_returns_what_the_helper_printed() {
        assert_eq!(
            interpret(true, b"[]\n", b""),
            SudoRun::Finished(Ok("[]\n".to_owned()))
        );
    }

    #[test]
    fn a_refused_fingerprint_is_not_a_failure_of_the_helper() {
        let errors = b"sudo: no password was provided\nsudo: a password is required\n";
        assert_eq!(interpret(false, b"", errors), SudoRun::NotAuthenticated);
        let errors = b"sudo: a terminal is required to read the password\n";
        assert_eq!(interpret(false, b"", errors), SudoRun::NotAuthenticated);
    }

    #[test]
    fn a_helper_that_fails_reports_its_own_reason() {
        let errors = b"\ndiskmarshal-privileged-helper: rule not allowed here: app-caches\n\n";
        assert_eq!(
            interpret(false, b"", errors),
            SudoRun::Finished(Err(PrivilegeError::Failed(
                "diskmarshal-privileged-helper: rule not allowed here: app-caches".to_owned()
            )))
        );
    }
}

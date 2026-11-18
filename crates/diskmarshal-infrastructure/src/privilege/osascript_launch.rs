//! Starts the helper as an administrator through AppleScript, whose dialog
//! takes the administrator password. It is the way that always exists.

use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, Stdio};

use diskmarshal_application::ports::PrivilegeError;

/// Absolute path, so a different program earlier in `PATH` can never run.
const OSASCRIPT: &str = "/usr/bin/osascript";

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

/// Asks macOS to authenticate the user with its password dialog, showing
/// `prompt`, and to run `helper` once with `arguments`. Returns what the
/// helper printed.
pub(crate) fn run(
    helper: &Path,
    arguments: &[OsString],
    prompt: &str,
) -> Result<String, PrivilegeError> {
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

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }
    let message = String::from_utf8_lossy(&output.stderr);
    Err(if message.contains(USER_CANCELLED) {
        PrivilegeError::Cancelled
    } else {
        PrivilegeError::Failed(message.trim().to_owned())
    })
}

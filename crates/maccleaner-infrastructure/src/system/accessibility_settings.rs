//! Accessibility choices the user made in System Settings.

use std::process::{Command, Stdio};

/// Absolute path, so a different program earlier in `PATH` can never run.
const DEFAULTS: &str = "/usr/bin/defaults";
/// Where macOS keeps the "Reduce motion" switch.
const DOMAIN: &str = "com.apple.universalaccess";
const REDUCE_MOTION_KEY: &str = "reduceMotion";

/// Whether the user turned on "Reduce motion" in System Settings.
///
/// macOS only stores the switch once it has been changed, so a missing
/// value, like any failure to read it, counts as off.
#[must_use]
pub fn prefers_reduced_motion() -> bool {
    Command::new(DEFAULTS)
        .args(["read", DOMAIN, REDUCE_MOTION_KEY])
        .env_clear()
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .is_ok_and(|output| output.status.success() && is_on(&output.stdout))
}

/// `defaults` prints a switch that is on as `1`.
fn is_on(printed: &[u8]) -> bool {
    printed.trim_ascii() == b"1"
}

#[cfg(test)]
mod tests {
    use super::is_on;

    #[test]
    fn reads_the_switch_the_way_defaults_prints_it() {
        assert!(is_on(b"1\n"));
        assert!(!is_on(b"0\n"));
        assert!(!is_on(b""));
        assert!(!is_on(b"10\n"));
    }
}

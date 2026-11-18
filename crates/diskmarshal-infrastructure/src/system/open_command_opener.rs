//! Opener built on the `open` command that ships with macOS.

use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

use diskmarshal_application::ApplicationError;
use diskmarshal_application::ports::ExternalOpener;

/// Absolute path, so a different `open` earlier in `PATH` can never run instead.
const OPEN: &str = "/usr/bin/open";
const FULL_DISK_ACCESS_SETTINGS: &str =
    "x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles";

/// Asks macOS to open things, with `/usr/bin/open` and no shell in between.
#[derive(Debug, Clone, Copy, Default)]
pub struct OpenCommandOpener;

impl ExternalOpener for OpenCommandOpener {
    fn reveal_in_finder(&self, path: &Path) -> Result<(), ApplicationError> {
        run(Command::new(OPEN).arg("-R").arg(path), path)
    }

    fn open_full_disk_access_settings(&self) -> Result<(), ApplicationError> {
        run(
            Command::new(OPEN).arg(FULL_DISK_ACCESS_SETTINGS),
            Path::new(OPEN),
        )
    }
}

fn run(command: &mut Command, subject: &Path) -> Result<(), ApplicationError> {
    let status = command
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let failure = match status {
        Ok(status) if status.success() => return Ok(()),
        Ok(status) => io::Error::other(format!("open ended with {status}")),
        Err(error) => error,
    };
    Err(ApplicationError::Io {
        path: subject.to_path_buf(),
        source: failure,
    })
}

#[cfg(test)]
mod tests {
    use diskmarshal_application::ports::ExternalOpener;
    use diskmarshal_test_support::TempTree;

    use super::OpenCommandOpener;

    #[test]
    fn revealing_an_item_that_does_not_exist_fails() {
        let fixture = TempTree::new().unwrap();
        assert!(
            OpenCommandOpener
                .reveal_in_finder(&fixture.path("gone"))
                .is_err()
        );
    }
}

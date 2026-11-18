//! Version control probe that asks `git`.

use std::io::Read as _;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use maccleaner_application::ports::VersionControlProbe;

/// Absolute path, so a different `git` earlier in `PATH` can never run instead.
const GIT: &str = "/usr/bin/git";
/// Longest a single question to git may take.
const TIMEOUT: Duration = Duration::from_secs(5);
const POLL_INTERVAL: Duration = Duration::from_millis(5);
/// Exit code of `git ls-files --error-unmatch` when nothing tracked matches.
const NOTHING_TRACKED: i32 = 1;
/// What git says when the folder is not inside a repository at all.
const NOT_A_REPOSITORY: &str = "not a git repository";

/// Asks git whether a folder has tracked files.
///
/// When git cannot answer (it is missing, it times out, the repository is
/// broken), the probe answers `true`: a folder that might be project content
/// is never offered for removal.
#[derive(Debug, Clone, Copy, Default)]
pub struct GitVersionControlProbe;

impl VersionControlProbe for GitVersionControlProbe {
    fn has_tracked_files(&self, project: &Path, folder: &str) -> bool {
        let child = Command::new(GIT)
            .arg("-C")
            .arg(project)
            .args(["ls-files", "--error-unmatch", "--"])
            .arg(folder)
            .env_clear()
            // A fixed language keeps the message this probe looks for stable.
            .env("LC_ALL", "C")
            .env("GIT_OPTIONAL_LOCKS", "0")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn();
        let Ok(mut child) = child else { return true };
        let Some(code) = wait(&mut child) else {
            return true;
        };
        match code {
            0 => true,
            NOTHING_TRACKED => false,
            _ => !error_output(&mut child).contains(NOT_A_REPOSITORY),
        }
    }
}

/// Waits for the program up to the time limit. Returns its exit code, or
/// `None` when it had to be stopped or ended without one.
fn wait(child: &mut Child) -> Option<i32> {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.code(),
            Ok(None) if Instant::now() < deadline => thread::sleep(POLL_INTERVAL),
            Ok(None) | Err(_) => {
                // Stopping it is all that is left to do; the answer is already "unknown".
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }
}

fn error_output(child: &mut Child) -> String {
    let mut text = String::new();
    if let Some(mut stream) = child.stderr.take() {
        // An unreadable message leaves the text empty, which reads as "unknown".
        let _ = stream.read_to_string(&mut text);
    }
    text
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use maccleaner_application::ports::VersionControlProbe;
    use maccleaner_test_support::TempTree;

    use super::{GIT, GitVersionControlProbe};

    fn git(directory: &Path, arguments: &[&str]) {
        let status = Command::new(GIT)
            .arg("-C")
            .arg(directory)
            .args(["-c", "user.name=Test", "-c", "user.email=test@example.com"])
            .args(arguments)
            .output()
            .unwrap()
            .status;
        assert!(status.success(), "git {arguments:?}");
    }

    #[test]
    fn a_folder_outside_any_repository_has_nothing_tracked() {
        let fixture = TempTree::new().unwrap();
        fixture.file("project/node_modules/lib.js", 10).unwrap();

        let project = fixture.path("project");
        assert!(!GitVersionControlProbe.has_tracked_files(&project, "node_modules"));
    }

    #[test]
    fn tells_tracked_folders_from_untracked_ones() {
        let fixture = TempTree::new().unwrap();
        fixture.file("project/bin/deploy.sh", 10).unwrap();
        fixture.file("project/node_modules/lib.js", 10).unwrap();
        let project = fixture.path("project");
        git(&project, &["init", "--quiet"]);
        git(&project, &["add", "bin"]);
        git(&project, &["commit", "--quiet", "-m", "add scripts"]);

        assert!(GitVersionControlProbe.has_tracked_files(&project, "bin"));
        assert!(!GitVersionControlProbe.has_tracked_files(&project, "node_modules"));
    }

    #[test]
    fn a_project_folder_that_does_not_exist_cannot_be_ruled_out() {
        let fixture = TempTree::new().unwrap();
        assert!(GitVersionControlProbe.has_tracked_files(&fixture.path("gone"), "bin"));
    }
}

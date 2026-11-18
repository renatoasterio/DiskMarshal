//! Version control probe that asks `git`.

use std::path::Path;
use std::process::Command;
use std::time::Duration;

use diskmarshal_application::ports::VersionControlProbe;

use crate::process::bounded_command::{Stdout, run_bounded};

/// Absolute path, so a different `git` earlier in `PATH` can never run instead.
const GIT: &str = "/usr/bin/git";
/// Longest a single question to git may take.
const TIMEOUT: Duration = Duration::from_secs(5);
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
        let mut command = Command::new(GIT);
        command
            .arg("-C")
            .arg(project)
            .args(["ls-files", "--error-unmatch", "--"])
            .arg(folder)
            .env_clear()
            // A fixed language keeps the message this probe looks for stable.
            .env("LC_ALL", "C")
            .env("GIT_OPTIONAL_LOCKS", "0");
        let Ok(output) = run_bounded(&mut command, Stdout::Discard, TIMEOUT) else {
            return true;
        };
        match output.code {
            Some(0) | None => true,
            Some(NOTHING_TRACKED) => false,
            Some(_) => !String::from_utf8_lossy(&output.stderr).contains(NOT_A_REPOSITORY),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use diskmarshal_application::ports::VersionControlProbe;
    use diskmarshal_test_support::TempTree;

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

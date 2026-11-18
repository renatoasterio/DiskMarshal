//! Detects Full Disk Access by trying to list folders macOS guards with it.
//!
//! macOS offers no call that answers the question. Listing a guarded folder
//! never shows a dialog: it works with the permission and fails without.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use maccleaner_application::ports::{AccessStatus, PermissionProbe};

/// Folders inside the home folder that only open with Full Disk Access.
const GUARDED_FOLDERS: [&str; 4] = [
    "Library/Safari",
    "Library/Mail",
    "Library/Messages",
    "Library/Cookies",
];

/// Tests Full Disk Access against the guarded folders of one user.
#[derive(Debug, Clone)]
pub struct FullDiskAccessProbe {
    guarded: Vec<PathBuf>,
}

impl FullDiskAccessProbe {
    /// Creates a probe for the user whose home folder is `home`.
    #[must_use]
    pub fn for_home(home: &Path) -> Self {
        Self::over(
            GUARDED_FOLDERS
                .iter()
                .map(|folder| home.join(folder))
                .collect(),
        )
    }

    /// Creates a probe that tests with the given folders.
    #[must_use]
    pub fn over(guarded: Vec<PathBuf>) -> Self {
        Self { guarded }
    }
}

impl PermissionProbe for FullDiskAccessProbe {
    /// One folder that lists proves the permission; otherwise one refusal
    /// proves its absence. Folders that do not exist prove nothing.
    fn full_disk_access(&self) -> AccessStatus {
        let mut status = AccessStatus::Unknown;
        for folder in &self.guarded {
            match fs::read_dir(folder) {
                Ok(_) => return AccessStatus::Granted,
                Err(error) if error.kind() == io::ErrorKind::PermissionDenied => {
                    status = AccessStatus::Denied;
                }
                Err(_) => {}
            }
        }
        status
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use maccleaner_application::ports::{AccessStatus, PermissionProbe};
    use maccleaner_test_support::TempTree;

    use super::FullDiskAccessProbe;

    #[test]
    fn a_folder_that_lists_proves_the_permission() {
        let fixture = TempTree::new().unwrap();
        let open = fixture.directory("open").unwrap();
        let probe = FullDiskAccessProbe::over(vec![fixture.path("missing"), open]);
        assert_eq!(probe.full_disk_access(), AccessStatus::Granted);
    }

    #[test]
    fn a_refusal_proves_its_absence() {
        let fixture = TempTree::new().unwrap();
        let locked = fixture.directory("locked").unwrap();
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();

        let probe = FullDiskAccessProbe::over(vec![fixture.path("missing"), locked.clone()]);
        let status = probe.full_disk_access();

        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(status, AccessStatus::Denied);
    }

    #[test]
    fn nothing_to_test_with_is_unknown() {
        let fixture = TempTree::new().unwrap();
        let probe = FullDiskAccessProbe::over(vec![fixture.path("missing")]);
        assert_eq!(probe.full_disk_access(), AccessStatus::Unknown);
    }
}

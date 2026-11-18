//! One app installed on this Mac.

use std::path::{Path, PathBuf};

use crate::cleanup::PrivilegeScope;
use crate::shared::ByteSize;

/// An app bundle found in one of the applications folders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledApp {
    /// Name shown to the user: the bundle's file name without `.app`.
    pub name: String,
    /// The name the bundle declares for itself, when it differs from `name`.
    pub declared_name: Option<String>,
    /// Bundle identifier, such as `com.vendor.editor`.
    pub bundle_id: String,
    /// Version shown to the user, when the bundle declares one.
    pub version: Option<String>,
    /// Absolute path of the bundle.
    pub path: PathBuf,
    /// Space the bundle occupies on disk.
    pub size: ByteSize,
    /// The rights removing the bundle needs.
    pub scope: PrivilegeScope,
    /// Whether one of the app's programs is running right now.
    pub is_running: bool,
}

impl InstalledApp {
    /// Returns `true` when `executable` is a program inside this bundle.
    #[must_use]
    pub fn owns_executable(&self, executable: &Path) -> bool {
        executable != self.path && executable.starts_with(&self.path)
    }

    /// Returns `true` when `installed` holds another bundle with the same
    /// identifier. The two copies share their data, so removing the data
    /// of one would break the other.
    #[must_use]
    pub fn has_other_copy_in(&self, installed: &[Self]) -> bool {
        installed.iter().any(|other| {
            other.path != self.path && other.bundle_id.eq_ignore_ascii_case(&self.bundle_id)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::InstalledApp;
    use crate::cleanup::PrivilegeScope;
    use crate::shared::ByteSize;

    fn app(path: &str, bundle_id: &str) -> InstalledApp {
        InstalledApp {
            name: "Editor".into(),
            declared_name: None,
            bundle_id: bundle_id.into(),
            version: None,
            path: PathBuf::from(path),
            size: ByteSize::ZERO,
            scope: PrivilegeScope::User,
            is_running: false,
        }
    }

    #[test]
    fn owns_only_the_programs_inside_its_bundle() {
        let editor = app("/Applications/Editor.app", "com.vendor.editor");
        assert!(
            editor.owns_executable(Path::new("/Applications/Editor.app/Contents/MacOS/Editor"))
        );
        assert!(
            !editor.owns_executable(Path::new("/Applications/Editor.app2/Contents/MacOS/Editor"))
        );
        assert!(!editor.owns_executable(Path::new("/Applications/Editor.app")));
        assert!(!editor.owns_executable(Path::new("/usr/bin/true")));
    }

    #[test]
    fn notices_another_copy_with_the_same_identifier() {
        let editor = app("/Applications/Editor.app", "com.vendor.editor");
        let copy = app("/Users/tester/Applications/Editor.app", "COM.vendor.Editor");
        let other = app("/Applications/Viewer.app", "com.vendor.viewer");

        assert!(!editor.has_other_copy_in(&[editor.clone(), other.clone()]));
        assert!(editor.has_other_copy_in(&[editor.clone(), other, copy]));
    }
}

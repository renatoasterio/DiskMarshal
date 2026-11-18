//! The rule that decides whether a path may be deleted at all.

use std::path::{Component, Path, PathBuf};

use crate::scan::ScanPolicy;

/// Why a path may not be deleted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtectionReason {
    /// The path is relative, so it does not name one definite item.
    NotAbsolute,
    /// The path contains `..`, which could point somewhere unexpected.
    NotNormalized,
    /// The path is a location the system or the user depends on.
    ProtectedLocation,
    /// The path is inside a tree owned by macOS.
    InsideProtectedTree,
    /// The path is, or is inside, a `.git` folder.
    InsideGitFolder,
    /// The path is inside a package such as an app or a Photos library.
    InsidePackage,
    /// The path is one of the scanned folders itself.
    IsAllowedRoot,
    /// The path is outside every folder the user scanned.
    OutsideAllowedRoots,
}

/// Decides whether a path may be deleted.
///
/// Every deletion goes through [`check`](Self::check). Paths are compared
/// without regard to letter case, because the default macOS file system treats
/// `/System` and `/system` as the same folder.
#[derive(Debug, Clone)]
pub struct ProtectedPaths {
    locations: Vec<Vec<String>>,
    trees: Vec<Vec<String>>,
}

impl ProtectedPaths {
    /// Locations that must never be removed themselves. Their content may be.
    const SYSTEM_LOCATIONS: [&'static str; 14] = [
        "/",
        "/Applications",
        "/Library",
        "/Library/Caches",
        "/Library/Logs",
        "/Users",
        "/Volumes",
        "/cores",
        "/etc",
        "/opt",
        "/private",
        "/private/tmp",
        "/private/var",
        "/tmp",
    ];

    /// Trees owned by macOS: nothing inside them may be removed.
    const SYSTEM_TREES: [&'static str; 4] = ["/System", "/bin", "/sbin", "/usr"];

    /// Locations inside the home folder that must never be removed themselves.
    /// The empty entry is the home folder.
    const HOME_LOCATIONS: [&'static str; 20] = [
        "",
        ".Trash",
        "Applications",
        "Desktop",
        "Documents",
        "Downloads",
        "Library",
        "Library/Application Support",
        "Library/Caches",
        "Library/Containers",
        "Library/Developer",
        "Library/Group Containers",
        "Library/Logs",
        "Library/Mobile Documents",
        "Library/Preferences",
        "Movies",
        "Music",
        "Pictures",
        "Public",
        "Sites",
    ];

    const GIT_FOLDER: &'static str = ".git";

    /// Builds the policy for the user whose home folder is `home`.
    #[must_use]
    pub fn new(home: &Path) -> Self {
        let system = Self::SYSTEM_LOCATIONS
            .iter()
            .map(|location| normalize(Path::new(location)));
        let personal = Self::HOME_LOCATIONS
            .iter()
            .map(|relative| normalize(&home.join(relative)));
        Self {
            locations: system.chain(personal).collect(),
            trees: Self::SYSTEM_TREES
                .iter()
                .map(|tree| normalize(Path::new(tree)))
                .collect(),
        }
    }

    /// Checks whether `target` may be deleted.
    ///
    /// `target` must already be resolved on disk (no symbolic link in its
    /// parent folders), and so must `allowed_roots`.
    ///
    /// # Errors
    ///
    /// Returns the first reason that forbids the deletion.
    pub fn check(&self, target: &Path, allowed_roots: &[PathBuf]) -> Result<(), ProtectionReason> {
        if !target.is_absolute() {
            return Err(ProtectionReason::NotAbsolute);
        }
        if target
            .components()
            .any(|component| matches!(component, Component::ParentDir))
        {
            return Err(ProtectionReason::NotNormalized);
        }

        let parts = normalize(target);
        if self.locations.contains(&parts) {
            return Err(ProtectionReason::ProtectedLocation);
        }
        if self.trees.iter().any(|tree| parts.starts_with(tree)) {
            return Err(ProtectionReason::InsideProtectedTree);
        }
        if parts.iter().any(|part| part == Self::GIT_FOLDER) {
            return Err(ProtectionReason::InsideGitFolder);
        }
        let ancestors = parts
            .split_last()
            .map_or(&[][..], |(_, ancestors)| ancestors);
        if ancestors.iter().any(|part| ScanPolicy::is_package(part)) {
            return Err(ProtectionReason::InsidePackage);
        }

        let roots: Vec<Vec<String>> = allowed_roots.iter().map(|root| normalize(root)).collect();
        if roots.contains(&parts) {
            return Err(ProtectionReason::IsAllowedRoot);
        }
        if roots.iter().any(|root| parts.starts_with(root)) {
            Ok(())
        } else {
            Err(ProtectionReason::OutsideAllowedRoots)
        }
    }
}

/// Splits a path into lowercase names, so paths compare component by
/// component and without regard to letter case.
fn normalize(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(name) => Some(name.to_string_lossy().to_lowercase()),
            Component::RootDir
            | Component::CurDir
            | Component::ParentDir
            | Component::Prefix(_) => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{ProtectedPaths, ProtectionReason};

    const HOME: &str = "/Users/tester";

    fn check(target: &str, allowed_roots: &[&str]) -> Result<(), ProtectionReason> {
        let roots: Vec<PathBuf> = allowed_roots.iter().map(PathBuf::from).collect();
        ProtectedPaths::new(Path::new(HOME)).check(Path::new(target), &roots)
    }

    #[test]
    fn allows_an_ordinary_item_inside_a_scanned_folder() {
        assert_eq!(
            check(
                "/Users/tester/Library/Caches/com.vendor.editor/data",
                &["/Users/tester"]
            ),
            Ok(())
        );
        assert_eq!(
            check(
                "/Users/tester/code/web/node_modules",
                &["/Users/tester/code"]
            ),
            Ok(())
        );
    }

    #[test]
    fn rejects_relative_and_unnormalized_paths() {
        assert_eq!(
            check("Library/Caches/x", &["/Users/tester"]),
            Err(ProtectionReason::NotAbsolute)
        );
        assert_eq!(
            check("/Users/tester/code/../Documents", &["/Users/tester"]),
            Err(ProtectionReason::NotNormalized)
        );
    }

    #[test]
    fn rejects_every_protected_location() {
        for location in [
            "/",
            "/Applications",
            "/Library",
            "/Library/Caches",
            "/Users",
            "/private/var",
            "/Users/tester",
            "/Users/tester/Library",
            "/Users/tester/Library/Caches",
            "/Users/tester/Documents",
            "/Users/tester/Desktop",
            "/Users/tester/Pictures",
            "/Users/tester/.Trash",
        ] {
            assert_eq!(
                check(location, &["/"]),
                Err(ProtectionReason::ProtectedLocation),
                "{location}"
            );
        }
    }

    #[test]
    fn rejects_anything_inside_trees_owned_by_macos() {
        for target in [
            "/System",
            "/System/Library/Caches/x",
            "/usr/local/bin/tool",
            "/bin/sh",
        ] {
            assert_eq!(
                check(target, &["/"]),
                Err(ProtectionReason::InsideProtectedTree),
                "{target}"
            );
        }
    }

    #[test]
    fn compares_paths_without_regard_to_letter_case() {
        assert_eq!(
            check("/system/library/x", &["/"]),
            Err(ProtectionReason::InsideProtectedTree)
        );
        assert_eq!(
            check("/users/TESTER/documents", &["/"]),
            Err(ProtectionReason::ProtectedLocation)
        );
    }

    #[test]
    fn rejects_git_folders_and_their_content() {
        assert_eq!(
            check("/Users/tester/code/web/.git", &["/Users/tester"]),
            Err(ProtectionReason::InsideGitFolder)
        );
        assert_eq!(
            check("/Users/tester/code/web/.git/objects/ab", &["/Users/tester"]),
            Err(ProtectionReason::InsideGitFolder)
        );
    }

    #[test]
    fn rejects_the_inside_of_a_package_but_not_the_package_itself() {
        assert_eq!(
            check(
                "/Users/tester/Pictures/Photos Library.photoslibrary/originals/a.heic",
                &["/Users/tester"]
            ),
            Err(ProtectionReason::InsidePackage)
        );
        assert_eq!(
            check("/Users/tester/Downloads/Old.app", &["/Users/tester"]),
            Ok(())
        );
    }

    #[test]
    fn rejects_a_scanned_folder_itself_and_anything_outside_them() {
        assert_eq!(
            check("/Users/tester/code", &["/Users/tester/code"]),
            Err(ProtectionReason::IsAllowedRoot)
        );
        assert_eq!(
            check("/Users/other/code/x", &["/Users/tester/code"]),
            Err(ProtectionReason::OutsideAllowedRoots)
        );
        assert_eq!(
            check("/Users/tester/codebase/x", &["/Users/tester/code"]),
            Err(ProtectionReason::OutsideAllowedRoots)
        );
    }
}

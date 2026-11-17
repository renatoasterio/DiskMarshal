//! What a scan is allowed to enter.

use std::path::{Path, PathBuf};

/// Decides which folders a scan skips and which ones it treats as a single item.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScanPolicy {
    pruned_paths: Vec<PathBuf>,
}

impl ScanPolicy {
    /// Folders skipped when the whole startup disk is scanned.
    ///
    /// `/System/Volumes` shows the data volume a second time, so entering it
    /// would count every user file twice. `/Volumes` holds other disks, and
    /// `/dev` holds devices, not files.
    const WHOLE_DISK_PRUNED: [&'static str; 3] = ["/System/Volumes", "/Volumes", "/dev"];

    /// Extensions of folders macOS presents as a single item.
    const PACKAGE_EXTENSIONS: [&'static str; 14] = [
        "app",
        "appex",
        "bundle",
        "fcpbundle",
        "framework",
        "imovielibrary",
        "kext",
        "logicx",
        "musiclibrary",
        "photoslibrary",
        "plugin",
        "sparsebundle",
        "tvlibrary",
        "xcarchive",
    ];

    /// Builds the policy for a scan that starts at `root`.
    #[must_use]
    pub fn for_root(root: &Path) -> Self {
        let pruned_paths = if root == Path::new("/") {
            Self::WHOLE_DISK_PRUNED.iter().map(PathBuf::from).collect()
        } else {
            Vec::new()
        };
        Self { pruned_paths }
    }

    /// Builds a policy that skips exactly the given folders.
    #[must_use]
    pub fn pruning(paths: impl IntoIterator<Item = PathBuf>) -> Self {
        Self {
            pruned_paths: paths.into_iter().collect(),
        }
    }

    /// Returns `true` when the scan must not enter `path`.
    #[must_use]
    pub fn is_pruned(&self, path: &Path) -> bool {
        self.pruned_paths.iter().any(|pruned| path == pruned)
    }

    /// Returns `true` when a folder with this name is a package: it is measured
    /// as a whole and its content never appears in the tree.
    #[must_use]
    pub fn is_package(name: &str) -> bool {
        name.rsplit_once('.').is_some_and(|(stem, extension)| {
            !stem.is_empty()
                && Self::PACKAGE_EXTENSIONS
                    .iter()
                    .any(|known| extension.eq_ignore_ascii_case(known))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ScanPolicy;
    use std::path::Path;

    #[test]
    fn whole_disk_scan_prunes_the_duplicated_data_volume() {
        let policy = ScanPolicy::for_root(Path::new("/"));
        assert!(policy.is_pruned(Path::new("/System/Volumes")));
        assert!(policy.is_pruned(Path::new("/Volumes")));
        assert!(!policy.is_pruned(Path::new("/Users")));
    }

    #[test]
    fn folder_scan_prunes_nothing() {
        let policy = ScanPolicy::for_root(Path::new("/Users/someone"));
        assert!(!policy.is_pruned(Path::new("/System/Volumes")));
    }

    #[test]
    fn recognizes_packages_by_extension_ignoring_case() {
        assert!(ScanPolicy::is_package("Photos Library.photoslibrary"));
        assert!(ScanPolicy::is_package("Xcode.APP"));
        assert!(!ScanPolicy::is_package("notes.txt"));
        assert!(!ScanPolicy::is_package("app"));
        assert!(!ScanPolicy::is_package(".app"));
    }
}

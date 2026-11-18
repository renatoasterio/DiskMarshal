//! Adapters that read app bundles.

mod plist_app_bundle_reader;

use std::path::{Path, PathBuf};

pub use plist_app_bundle_reader::PlistAppBundleReader;

/// The folder where macOS and installers put apps for every user.
const SHARED_APPLICATIONS: &str = "/Applications";
/// The folder, inside the home folder, for the apps of one user.
const PERSONAL_APPLICATIONS: &str = "Applications";

/// The folders that hold the apps the user whose home folder is `home` can
/// open, in the order they are searched.
#[must_use]
pub fn application_folders(home: &Path) -> Vec<PathBuf> {
    vec![
        PathBuf::from(SHARED_APPLICATIONS),
        home.join(PERSONAL_APPLICATIONS),
    ]
}

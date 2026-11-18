//! Port for reading what an app bundle says about itself.

use std::path::Path;

use maccleaner_domain::cleanup::PrivilegeScope;

/// What an app bundle declares, and who may remove it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppBundleFacts {
    /// Bundle identifier, such as `com.vendor.editor`.
    pub bundle_id: String,
    /// The name the bundle declares for itself, when it declares one.
    pub declared_name: Option<String>,
    /// Version shown to the user, when the bundle declares one.
    pub version: Option<String>,
    /// The rights removing the bundle needs.
    pub scope: PrivilegeScope,
}

/// Reads the description every app bundle carries.
pub trait AppBundleReader: Send + Sync {
    /// Reads the bundle at `bundle`. Returns `None` when it is not an app
    /// bundle, cannot be read, or declares no identifier.
    fn read(&self, bundle: &Path) -> Option<AppBundleFacts>;
}

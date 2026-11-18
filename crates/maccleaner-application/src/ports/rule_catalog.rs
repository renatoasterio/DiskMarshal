//! Port for the catalog of things MacCleaner knows how to clean.

use maccleaner_domain::applications::LeftoverLocation;
use maccleaner_domain::cleanup::CleanupRule;
use maccleaner_domain::dev_artifacts::ArtifactRule;

use crate::ApplicationError;

/// Supplies the rule catalogs shipped with the app. Folders of cleanup rules
/// come already made absolute for the current user.
pub trait RuleCatalog: Send + Sync {
    /// Returns every cleanup rule, in the order they are shown.
    ///
    /// # Errors
    ///
    /// Fails when the catalog is malformed.
    fn cleanup_rules(&self) -> Result<Vec<CleanupRule>, ApplicationError>;

    /// Returns the rules that recognize code projects and their build folders.
    ///
    /// # Errors
    ///
    /// Fails when the catalog is malformed.
    fn artifact_rules(&self) -> Result<Vec<ArtifactRule>, ApplicationError>;

    /// Returns the folders where apps leave items named after themselves,
    /// in the order they are searched.
    ///
    /// # Errors
    ///
    /// Fails when the catalog is malformed.
    fn leftover_locations(&self) -> Result<Vec<LeftoverLocation>, ApplicationError>;
}

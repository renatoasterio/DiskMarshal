//! How one ecosystem's projects and build folders are recognized.

use crate::cleanup::RiskLevel;
use crate::dev_artifacts::Ecosystem;

/// A folder a project of some ecosystem can recreate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactFolder {
    /// Name of the folder, such as `node_modules`.
    pub name: String,
    /// `Safe` for names nobody uses for anything else. `Review` for common
    /// names such as `build`, which a project might use for real content.
    pub risk: RiskLevel,
}

/// Says how to recognize a project and which of its folders can be removed.
///
/// A folder is only ever offered when one of the markers sits next to it.
/// That is what keeps a `bin` or a `dist` that has nothing to do with a build
/// from being offered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactRule {
    /// The ecosystem this rule recognizes.
    pub ecosystem: Ecosystem,
    /// File names that mark a project folder. A marker starting with `*.`
    /// matches any file with that extension.
    pub markers: Vec<String>,
    /// The folders that can be removed from such a project.
    pub folders: Vec<ArtifactFolder>,
}

impl ArtifactRule {
    /// Returns `true` when `file_name` marks a project of this ecosystem.
    #[must_use]
    pub fn is_marker(&self, file_name: &str) -> bool {
        let lowercase = file_name.to_lowercase();
        self.markers
            .iter()
            .any(|marker| match marker.strip_prefix('*') {
                Some(extension) => lowercase.ends_with(&extension.to_lowercase()),
                None => lowercase == marker.to_lowercase(),
            })
    }

    /// Returns the removable folder with this name, if the rule has one.
    #[must_use]
    pub fn folder(&self, name: &str) -> Option<&ArtifactFolder> {
        self.folders.iter().find(|folder| folder.name == name)
    }
}

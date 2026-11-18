//! Build and dependency folders that code projects leave behind.

mod artifact_finder;
mod artifact_rule;
mod ecosystem;
mod project_artifact;

pub use artifact_finder::find_artifacts;
pub use artifact_rule::{ArtifactFolder, ArtifactRule};
pub use ecosystem::Ecosystem;
pub use project_artifact::ProjectArtifact;

//! A removable folder found inside a code project.

use std::path::PathBuf;

use crate::cleanup::RiskLevel;
use crate::dev_artifacts::Ecosystem;
use crate::shared::ByteSize;

/// One build or dependency folder of one project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectArtifact {
    /// Name of the project folder.
    pub project: String,
    /// The ecosystem that recognized the project.
    pub ecosystem: Ecosystem,
    /// Name of the removable folder, such as `node_modules`.
    pub folder: String,
    /// Absolute path of the removable folder.
    pub path: PathBuf,
    /// Disk space the folder uses.
    pub size: ByteSize,
    /// When the project was last touched, in seconds since the Unix epoch:
    /// the modification time of its marker file.
    pub project_modified_unix_seconds: Option<i64>,
    /// How careful the user should be.
    pub risk: RiskLevel,
}

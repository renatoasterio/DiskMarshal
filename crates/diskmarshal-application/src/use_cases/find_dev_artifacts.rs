//! Use case: find the build and dependency folders of code projects.

use std::path::{Path, PathBuf};

use diskmarshal_domain::deletion::{DeletionMethod, DeletionRequest};
use diskmarshal_domain::dev_artifacts::{ProjectArtifact, find_artifacts};
use diskmarshal_domain::scan::ScanPolicy;

use crate::ApplicationError;
use crate::ports::{FileInspector, FileSystemWalker, RuleCatalog, VersionControlProbe};
use crate::progress::{CancellationToken, ProgressSink};

/// Folders directly inside the scanned folder that never hold code projects
/// and are slow to read. They are skipped when the user scans the home folder.
const SKIPPED_FOLDERS: [&str; 6] = [
    "Library",
    "Applications",
    "Pictures",
    "Movies",
    "Music",
    ".Trash",
];

/// A removable folder together with the removal that would delete it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundArtifact {
    /// What was found.
    pub artifact: ProjectArtifact,
    /// How to remove it.
    pub request: DeletionRequest,
}

/// What a search for build folders found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevArtifactsReport {
    /// The folder that was searched, as resolved on disk.
    pub root: PathBuf,
    /// Every removable folder, largest first.
    pub found: Vec<FoundArtifact>,
}

/// Searches a folder for projects and their recreatable folders.
pub struct FindDevArtifacts<'a> {
    catalog: &'a dyn RuleCatalog,
    walker: &'a dyn FileSystemWalker,
    inspector: &'a dyn FileInspector,
    version_control: &'a dyn VersionControlProbe,
}

impl<'a> FindDevArtifacts<'a> {
    /// Creates the use case over the catalog and the adapters it reads through.
    #[must_use]
    pub fn new(
        catalog: &'a dyn RuleCatalog,
        walker: &'a dyn FileSystemWalker,
        inspector: &'a dyn FileInspector,
        version_control: &'a dyn VersionControlProbe,
    ) -> Self {
        Self {
            catalog,
            walker,
            inspector,
            version_control,
        }
    }

    /// Searches `root`. A folder is offered only when a project file sits next
    /// to it and none of its files is under version control.
    ///
    /// # Errors
    ///
    /// Fails when `root` cannot be scanned, the catalog is malformed, or the
    /// search is cancelled.
    pub fn execute(
        &self,
        root: &Path,
        progress: &dyn ProgressSink,
        cancellation: &CancellationToken,
    ) -> Result<DevArtifactsReport, ApplicationError> {
        let rules = self.catalog.artifact_rules()?;
        let root = self
            .inspector
            .resolve(root)
            .ok_or_else(|| ApplicationError::RootNotFound(root.to_path_buf()))?;
        let policy = ScanPolicy::pruning(SKIPPED_FOLDERS.iter().map(|name| root.join(name)));
        let mut tree = self
            .walker
            .walk(&root, &policy, progress, cancellation)?
            .tree;
        if cancellation.is_cancelled() {
            return Err(ApplicationError::Cancelled);
        }
        tree.aggregate();

        let found = find_artifacts(&tree, &root, &rules)
            .into_iter()
            .filter(|artifact| !self.is_versioned(artifact))
            .filter_map(|artifact| {
                let fingerprint = self.inspector.fingerprint(&artifact.path)?;
                let request = DeletionRequest {
                    target: artifact.path.clone(),
                    method: DeletionMethod::Permanent,
                    fingerprint,
                    reclaimable: artifact.size,
                };
                Some(FoundArtifact { artifact, request })
            })
            .collect();
        Ok(DevArtifactsReport { root, found })
    }

    fn is_versioned(&self, artifact: &ProjectArtifact) -> bool {
        artifact.path.parent().is_none_or(|project| {
            self.version_control
                .has_tracked_files(project, &artifact.folder)
        })
    }
}

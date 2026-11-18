//! Use case: list the apps installed on this Mac. Removes nothing.

use std::path::{Path, PathBuf};

use diskmarshal_domain::applications::InstalledApp;
use diskmarshal_domain::scan::{EntryKind, FileTree, NodeId, ScanPolicy};
use diskmarshal_domain::shared::ByteSize;

use crate::ApplicationError;
use crate::ports::{AppBundleReader, FileInspector, FileSystemWalker, RunningAppProbe};
use crate::progress::{CancellationToken, ProgressSink};

/// What the name of an app bundle ends with.
const APP_EXTENSION: &str = ".app";

/// Finds the app bundles in the applications folders and measures them.
pub struct ListInstalledApps<'a> {
    walker: &'a dyn FileSystemWalker,
    inspector: &'a dyn FileInspector,
    bundles: &'a dyn AppBundleReader,
    running: &'a dyn RunningAppProbe,
}

impl<'a> ListInstalledApps<'a> {
    /// Creates the use case over the adapters it reads through.
    #[must_use]
    pub fn new(
        walker: &'a dyn FileSystemWalker,
        inspector: &'a dyn FileInspector,
        bundles: &'a dyn AppBundleReader,
        running: &'a dyn RunningAppProbe,
    ) -> Self {
        Self {
            walker,
            inspector,
            bundles,
            running,
        }
    }

    /// Lists the apps found in `folders`, largest first.
    ///
    /// An app counts when its bundle sits directly in one of the folders or
    /// one folder below, as suites install theirs. A folder that does not
    /// exist simply holds no app. A bundle reached through a symbolic link,
    /// as the apps macOS itself owns are, is left out: it cannot be removed.
    ///
    /// # Errors
    ///
    /// Fails when the listing is cancelled.
    pub fn execute(
        &self,
        folders: &[PathBuf],
        progress: &dyn ProgressSink,
        cancellation: &CancellationToken,
    ) -> Result<Vec<InstalledApp>, ApplicationError> {
        let executables = self.running.running_executables();
        let mut apps: Vec<InstalledApp> = Vec::new();
        for folder in folders {
            if cancellation.is_cancelled() {
                return Err(ApplicationError::Cancelled);
            }
            let Some(root) = self.inspector.resolve(folder) else {
                continue;
            };
            let Ok(outcome) =
                self.walker
                    .walk(&root, &ScanPolicy::default(), progress, cancellation)
            else {
                continue;
            };
            let mut tree = outcome.tree;
            tree.aggregate();
            for (path, size) in bundles_in(&tree, &root) {
                if apps.iter().any(|app| app.path == path) {
                    continue;
                }
                if let Some(app) = self.describe(path, size, &executables) {
                    apps.push(app);
                }
            }
        }
        if cancellation.is_cancelled() {
            return Err(ApplicationError::Cancelled);
        }
        apps.sort_by(|a, b| b.size.cmp(&a.size).then_with(|| a.name.cmp(&b.name)));
        Ok(apps)
    }

    fn describe(
        &self,
        path: PathBuf,
        size: ByteSize,
        executables: &[PathBuf],
    ) -> Option<InstalledApp> {
        let facts = self.bundles.read(&path)?;
        let name = app_name(path.file_name()?.to_str()?)?.to_owned();
        let declared_name = facts.declared_name.filter(|declared| *declared != name);
        let mut app = InstalledApp {
            name,
            declared_name,
            bundle_id: facts.bundle_id,
            version: facts.version,
            path,
            size,
            scope: facts.scope,
            is_running: false,
        };
        app.is_running = executables
            .iter()
            .any(|executable| app.owns_executable(executable));
        Some(app)
    }
}

/// The app bundles directly inside the scanned folder and one folder below,
/// each with the space it occupies.
fn bundles_in(tree: &FileTree, root: &Path) -> Vec<(PathBuf, ByteSize)> {
    let bundle = |id: NodeId, parent: &Path| {
        let node = tree.node(id)?;
        let is_app = node.kind() == EntryKind::Package && app_name(node.name()).is_some();
        is_app.then(|| (parent.join(node.name()), node.allocated_size()))
    };
    let mut found = Vec::new();
    for id in tree.children(FileTree::ROOT) {
        let Some(node) = tree.node(id) else { continue };
        if node.kind() == EntryKind::Directory {
            let folder = root.join(node.name());
            found.extend(tree.children(id).filter_map(|child| bundle(child, &folder)));
        } else {
            found.extend(bundle(id, root));
        }
    }
    found
}

/// The name of an app, given the file name of its bundle.
fn app_name(file_name: &str) -> Option<&str> {
    let split = file_name.len().checked_sub(APP_EXTENSION.len())?;
    let (name, extension) = file_name.split_at_checked(split)?;
    (!name.is_empty() && extension.eq_ignore_ascii_case(APP_EXTENSION)).then_some(name)
}

#[cfg(test)]
mod tests {
    use super::app_name;

    #[test]
    fn takes_the_name_of_an_app_from_its_bundle() {
        assert_eq!(app_name("Editor.app"), Some("Editor"));
        assert_eq!(app_name("Editor Pro.APP"), Some("Editor Pro"));
        assert_eq!(app_name("Ação.app"), Some("Ação"));
        assert_eq!(app_name(".app"), None);
        assert_eq!(app_name("Editor.application"), None);
        assert_eq!(app_name("app"), None);
    }
}

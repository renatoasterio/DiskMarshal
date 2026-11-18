//! Use case: find out what uninstalling one app would remove. Removes nothing.

use std::path::Path;

use maccleaner_domain::applications::{
    InstalledApp, LeftoverKind, LeftoverLocation, LeftoverMatcher, UninstallItem, UninstallPlan,
};
use maccleaner_domain::cleanup::{PrivilegeScope, RiskLevel};
use maccleaner_domain::deletion::{DeletionMethod, DeletionRequest};
use maccleaner_domain::scan::ScanPolicy;
use maccleaner_domain::shared::ByteSize;

use crate::ApplicationError;
use crate::ports::{FileInspector, FileSystemWalker, FolderItem, FolderLister, RuleCatalog};
use crate::progress::{CancellationToken, ProgressSink};

/// Builds the preview of an uninstall: the bundle and what the app left in
/// the library folders.
pub struct PlanUninstall<'a> {
    catalog: &'a dyn RuleCatalog,
    lister: &'a dyn FolderLister,
    walker: &'a dyn FileSystemWalker,
    inspector: &'a dyn FileInspector,
}

impl<'a> PlanUninstall<'a> {
    /// Creates the use case over the catalog and the adapters it reads through.
    #[must_use]
    pub fn new(
        catalog: &'a dyn RuleCatalog,
        lister: &'a dyn FolderLister,
        walker: &'a dyn FileSystemWalker,
        inspector: &'a dyn FileInspector,
    ) -> Self {
        Self {
            catalog,
            lister,
            walker,
            inspector,
        }
    }

    /// Finds everything that belongs to `app`. `installed` are all the apps
    /// on this Mac, so that nothing of another app is taken along.
    ///
    /// What the logged-in user can remove goes to the Trash. What needs an
    /// administrator is listed to be removed for good, because there is no
    /// Trash to send it to. When another copy of the app is installed, only
    /// the bundle is offered: the data belongs to both copies.
    ///
    /// # Errors
    ///
    /// Fails when the catalog is malformed, the bundle is gone, or the
    /// search is cancelled.
    pub fn execute(
        &self,
        app: &InstalledApp,
        installed: &[InstalledApp],
        progress: &dyn ProgressSink,
        cancellation: &CancellationToken,
    ) -> Result<UninstallPlan, ApplicationError> {
        let locations = self.catalog.leftover_locations()?;
        let missing = || ApplicationError::RootNotFound(app.path.clone());
        let bundle = self.inspector.resolve(&app.path).ok_or_else(missing)?;
        let fingerprint = self.inspector.fingerprint(&bundle).ok_or_else(missing)?;

        let mut plan = UninstallPlan {
            app: app.clone(),
            items: Vec::new(),
            roots: bundle.parent().map(Path::to_path_buf).into_iter().collect(),
            has_other_copy: app.has_other_copy_in(installed),
        };
        plan.items.push(UninstallItem {
            kind: LeftoverKind::Application,
            scope: app.scope,
            risk: RiskLevel::Safe,
            request: DeletionRequest {
                target: bundle,
                method: method_for(app.scope),
                fingerprint,
                reclaimable: app.size,
            },
        });
        if plan.has_other_copy {
            return Ok(plan);
        }

        let matcher = LeftoverMatcher::new(app, installed);
        for location in &locations {
            if cancellation.is_cancelled() {
                return Err(ApplicationError::Cancelled);
            }
            let Some(folder) = self.inspector.resolve(&location.folder) else {
                continue;
            };
            for item in self.lister.items(&folder) {
                let Some(risk) = matcher.classify(&item.name, location.is_matched_by_name) else {
                    continue;
                };
                let found = self.found_item(location, &folder, &item, risk, progress, cancellation);
                plan.items.extend(found);
            }
            plan.roots.push(folder);
        }
        if cancellation.is_cancelled() {
            return Err(ApplicationError::Cancelled);
        }
        Ok(plan)
    }

    fn found_item(
        &self,
        location: &LeftoverLocation,
        folder: &Path,
        item: &FolderItem,
        risk: RiskLevel,
        progress: &dyn ProgressSink,
        cancellation: &CancellationToken,
    ) -> Option<UninstallItem> {
        let target = folder.join(&item.name);
        let fingerprint = self.inspector.fingerprint(&target)?;
        let reclaimable = if item.is_folder {
            self.measure(&target, progress, cancellation)
                .unwrap_or(item.allocated_size)
        } else {
            item.allocated_size
        };
        Some(UninstallItem {
            kind: location.kind,
            scope: location.scope,
            risk,
            request: DeletionRequest {
                target,
                method: method_for(location.scope),
                fingerprint,
                reclaimable,
            },
        })
    }

    /// Space a folder and everything inside it occupy.
    fn measure(
        &self,
        folder: &Path,
        progress: &dyn ProgressSink,
        cancellation: &CancellationToken,
    ) -> Option<ByteSize> {
        let mut tree = self
            .walker
            .walk(folder, &ScanPolicy::default(), progress, cancellation)
            .ok()?
            .tree;
        tree.aggregate();
        Some(tree.root()?.allocated_size())
    }
}

/// The user's own items go to the Trash. With administrator rights there is
/// no Trash of the user to send anything to.
fn method_for(scope: PrivilegeScope) -> DeletionMethod {
    match scope {
        PrivilegeScope::User => DeletionMethod::Trash,
        PrivilegeScope::Administrator => DeletionMethod::Permanent,
    }
}

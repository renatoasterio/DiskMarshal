//! Use case: remove items of one app with the rights of an administrator.
//!
//! This is what the privileged helper runs for an uninstall. Nobody reviews
//! a preview there, so the helper trusts nothing it is told: it finds the app
//! by its identifier, works out again what belongs to it, and removes only
//! the requested items that its own analysis also found.

use std::path::PathBuf;

use maccleaner_domain::applications::{LeftoverKind, UninstallItem};
use maccleaner_domain::cleanup::PrivilegeScope;
use maccleaner_domain::deletion::{DeletionMethod, DeletionRequest};

use crate::ApplicationError;
use crate::progress::{CancellationToken, NoProgress};
use crate::use_cases::{
    DeleteItems, DeletionReport, ExecutionMode, ListInstalledApps, PlanUninstall,
};

/// Removes, for good, the bundle of an app and what it left in the folders
/// only an administrator may change.
pub struct RemoveAppItems<'a> {
    list: ListInstalledApps<'a>,
    plan: PlanUninstall<'a>,
    delete: DeleteItems<'a>,
}

impl<'a> RemoveAppItems<'a> {
    /// Creates the use case over the three use cases it chains.
    #[must_use]
    pub fn new(
        list: ListInstalledApps<'a>,
        plan: PlanUninstall<'a>,
        delete: DeleteItems<'a>,
    ) -> Self {
        Self { list, plan, delete }
    }

    /// Removes `items` of the app whose bundle identifier is `bundle_id`,
    /// looked for in `folders`.
    ///
    /// Every item must be the bundle itself or something the app left in an
    /// administrator folder of the catalog, as found right now. One item
    /// that is not stops the removal before anything is touched. So does an
    /// app that is not installed, is installed more than once, or is open.
    ///
    /// # Errors
    ///
    /// Fails when the catalog is malformed, the app cannot be told apart or
    /// is running, or an item is not allowed.
    pub fn execute(
        &self,
        bundle_id: &str,
        items: &[PathBuf],
        folders: &[PathBuf],
        mode: ExecutionMode,
    ) -> Result<DeletionReport, ApplicationError> {
        let (progress, cancellation) = (NoProgress, CancellationToken::new());
        let installed = self.list.execute(folders, &progress, &cancellation)?;
        let mut named = installed
            .iter()
            .filter(|app| app.bundle_id.eq_ignore_ascii_case(bundle_id));
        let (Some(app), None) = (named.next(), named.next()) else {
            return Err(ApplicationError::AppNotFound(bundle_id.to_owned()));
        };
        if app.is_running {
            return Err(ApplicationError::AppRunning(app.name.clone()));
        }

        let plan = self
            .plan
            .execute(app, &installed, &progress, &cancellation)?;
        let requests = items
            .iter()
            .map(|item| {
                plan.items
                    .iter()
                    .find(|found| found.request.target == *item && is_administrator_item(found))
                    .map(|found| DeletionRequest {
                        // An administrator has no Trash of the user to fill.
                        method: DeletionMethod::Permanent,
                        ..found.request.clone()
                    })
                    .ok_or_else(|| ApplicationError::ItemNotAllowed(item.clone()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(self.delete.execute(&requests, &plan.roots, mode))
    }
}

/// What an uninstall may remove with administrator rights: the bundle, and
/// what was found in the folders the catalog marks for an administrator.
/// The library of whoever runs the helper is never part of it.
fn is_administrator_item(item: &UninstallItem) -> bool {
    item.kind == LeftoverKind::Application || item.scope == PrivilegeScope::Administrator
}

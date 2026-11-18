//! Use case: find out what each cleanup rule would remove. Removes nothing.

use maccleaner_domain::cleanup::{CleanupFinding, CleanupPlan, CleanupRule};
use maccleaner_domain::deletion::DeletionRequest;
use maccleaner_domain::scan::{FileTree, ScanPolicy};

use crate::ApplicationError;
use crate::ports::{FileInspector, FileSystemWalker, RuleCatalog};
use crate::progress::{CancellationToken, ProgressSink};

/// Builds the preview of a cleanup.
pub struct PlanCleanup<'a> {
    catalog: &'a dyn RuleCatalog,
    walker: &'a dyn FileSystemWalker,
    inspector: &'a dyn FileInspector,
}

impl<'a> PlanCleanup<'a> {
    /// Creates the use case over the catalog and the adapters that read the disk.
    #[must_use]
    pub fn new(
        catalog: &'a dyn RuleCatalog,
        walker: &'a dyn FileSystemWalker,
        inspector: &'a dyn FileInspector,
    ) -> Self {
        Self {
            catalog,
            walker,
            inspector,
        }
    }

    /// Measures what every rule would remove, judging ages at `now_unix_seconds`.
    ///
    /// # Errors
    ///
    /// Fails when the catalog is malformed or the analysis is cancelled.
    pub fn execute(
        &self,
        now_unix_seconds: i64,
        progress: &dyn ProgressSink,
        cancellation: &CancellationToken,
    ) -> Result<CleanupPlan, ApplicationError> {
        let rules = self.catalog.cleanup_rules()?;
        self.execute_rules(rules, now_unix_seconds, progress, cancellation)
    }

    /// Measures what the given rules would remove.
    ///
    /// A rule's folder that does not exist or cannot be read simply finds
    /// nothing. Each removal targets one item directly inside a rule's
    /// folder, never the folder itself.
    ///
    /// # Errors
    ///
    /// Fails when the analysis is cancelled.
    pub fn execute_rules(
        &self,
        rules: Vec<CleanupRule>,
        now_unix_seconds: i64,
        progress: &dyn ProgressSink,
        cancellation: &CancellationToken,
    ) -> Result<CleanupPlan, ApplicationError> {
        let mut plan = CleanupPlan::default();
        for rule in rules {
            let mut requests = Vec::new();
            for location in &rule.locations {
                if cancellation.is_cancelled() {
                    return Err(ApplicationError::Cancelled);
                }
                let Some(root) = self.inspector.resolve(location) else {
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
                requests.extend(self.requests_in(&rule, &root, &tree, now_unix_seconds));
                plan.roots.push(root);
            }
            plan.findings.push(CleanupFinding { rule, requests });
        }
        if cancellation.is_cancelled() {
            return Err(ApplicationError::Cancelled);
        }
        Ok(plan)
    }

    /// One removal per item directly inside `root` that the rule accepts.
    fn requests_in(
        &self,
        rule: &CleanupRule,
        root: &std::path::Path,
        tree: &FileTree,
        now_unix_seconds: i64,
    ) -> Vec<DeletionRequest> {
        tree.children(FileTree::ROOT)
            .filter_map(|id| tree.node(id))
            .filter(|node| rule.is_old_enough(node.modified_unix_seconds(), now_unix_seconds))
            .filter_map(|node| {
                let target = root.join(node.name());
                let fingerprint = self.inspector.fingerprint(&target)?;
                Some(DeletionRequest {
                    target,
                    method: rule.method,
                    fingerprint,
                    reclaimable: node.allocated_size(),
                })
            })
            .collect()
    }
}

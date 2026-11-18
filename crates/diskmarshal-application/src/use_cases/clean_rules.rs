//! Use case: analyze and clean chosen rules in one go, with no preview.
//!
//! This is what the privileged helper runs. Nobody reviews a preview there,
//! so the rules are named by id and must belong to the scope the caller is
//! allowed to clean.

use diskmarshal_domain::cleanup::PrivilegeScope;

use crate::ApplicationError;
use crate::ports::RuleCatalog;
use crate::progress::{CancellationToken, NoProgress};
use crate::use_cases::{DeleteItems, DeletionReport, ExecutionMode, PlanCleanup};

/// Cleans the rules named by id, all of one privilege scope.
pub struct CleanRules<'a> {
    catalog: &'a dyn RuleCatalog,
    plan: PlanCleanup<'a>,
    delete: DeleteItems<'a>,
}

impl<'a> CleanRules<'a> {
    /// Creates the use case over the catalog and the two use cases it chains.
    #[must_use]
    pub fn new(
        catalog: &'a dyn RuleCatalog,
        plan: PlanCleanup<'a>,
        delete: DeleteItems<'a>,
    ) -> Self {
        Self {
            catalog,
            plan,
            delete,
        }
    }

    /// Cleans what the rules named in `rule_ids` select right now.
    ///
    /// Every id must name a rule of the catalog whose scope is `scope`; one
    /// that does not stops the cleanup before anything is touched.
    ///
    /// # Errors
    ///
    /// Fails when the catalog is malformed or a rule is not allowed.
    pub fn execute(
        &self,
        rule_ids: &[String],
        scope: PrivilegeScope,
        now_unix_seconds: i64,
        mode: ExecutionMode,
    ) -> Result<DeletionReport, ApplicationError> {
        let catalog = self.catalog.cleanup_rules()?;
        let rules = rule_ids
            .iter()
            .map(|id| {
                catalog
                    .iter()
                    .find(|rule| &rule.id == id && rule.scope == scope)
                    .cloned()
                    .ok_or_else(|| ApplicationError::RuleNotAllowed(id.clone()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let plan = self.plan.execute_rules(
            rules,
            now_unix_seconds,
            &NoProgress,
            &CancellationToken::new(),
        )?;
        let requests: Vec<_> = plan
            .findings
            .iter()
            .flat_map(|finding| finding.requests.iter().cloned())
            .collect();
        Ok(self.delete.execute(&requests, &plan.roots, mode))
    }
}

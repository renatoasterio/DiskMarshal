//! What an analysis of the cleanup rules found on this Mac.

use std::path::PathBuf;

use crate::cleanup::CleanupRule;
use crate::deletion::DeletionRequest;
use crate::shared::ByteSize;

/// What one rule found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupFinding {
    /// The rule that was analyzed.
    pub rule: CleanupRule,
    /// One removal per item found directly inside the rule's folders.
    pub requests: Vec<DeletionRequest>,
}

impl CleanupFinding {
    /// Space the rule would free.
    #[must_use]
    pub fn reclaimable(&self) -> ByteSize {
        self.requests
            .iter()
            .map(|request| request.reclaimable)
            .sum()
    }
}

/// The preview of a cleanup: every rule with what it would remove.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CleanupPlan {
    /// One finding per rule, in the order of the catalog.
    pub findings: Vec<CleanupFinding>,
    /// The folders that were analyzed. Nothing outside them may be removed.
    pub roots: Vec<PathBuf>,
}

impl CleanupPlan {
    /// The removals of the rules whose identifiers are in `rule_ids`.
    #[must_use]
    pub fn requests_for(&self, rule_ids: &[&str]) -> Vec<DeletionRequest> {
        self.findings
            .iter()
            .filter(|finding| rule_ids.contains(&finding.rule.id.as_str()))
            .flat_map(|finding| finding.requests.iter().cloned())
            .collect()
    }
}

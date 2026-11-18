//! Cleanup model: the rules that say what can be cleaned, and what a scan of
//! those rules found.

mod cleanup_plan;
mod cleanup_rule;
mod privilege_scope;
mod risk_level;

pub use cleanup_plan::{CleanupFinding, CleanupPlan};
pub use cleanup_rule::CleanupRule;
pub use privilege_scope::PrivilegeScope;
pub use risk_level::RiskLevel;

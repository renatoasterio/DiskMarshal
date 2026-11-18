//! One thing MacCleaner knows how to clean.

use std::path::PathBuf;

use crate::cleanup::{PrivilegeScope, RiskLevel};
use crate::deletion::DeletionMethod;

/// A named set of folders whose content can be removed.
///
/// A rule never removes its folders themselves, only what is inside them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupRule {
    /// Stable identifier, unique in the catalog.
    pub id: String,
    /// Name shown to the user.
    pub name: String,
    /// One line saying where the data is or why it is safe to remove.
    pub description: String,
    /// Absolute folders whose content this rule removes.
    pub locations: Vec<PathBuf>,
    /// How careful the user should be.
    pub risk: RiskLevel,
    /// The rights the removal needs.
    pub scope: PrivilegeScope,
    /// Whether the content goes to the Trash or is removed for good.
    pub method: DeletionMethod,
    /// Only content untouched for at least this many days is removed.
    pub minimum_age_days: Option<u32>,
}

impl CleanupRule {
    const SECONDS_PER_DAY: i64 = 86_400;

    /// Returns `true` when an item last modified at `modified_unix_seconds` is
    /// old enough for this rule, judged at `now_unix_seconds`.
    #[must_use]
    pub fn is_old_enough(&self, modified_unix_seconds: Option<i64>, now_unix_seconds: i64) -> bool {
        let Some(days) = self.minimum_age_days else {
            return true;
        };
        // An item with no known date is treated as new: when in doubt, keep it.
        let Some(modified) = modified_unix_seconds else {
            return false;
        };
        now_unix_seconds.saturating_sub(modified) >= i64::from(days) * Self::SECONDS_PER_DAY
    }
}

#[cfg(test)]
mod tests {
    use super::CleanupRule;
    use crate::cleanup::{PrivilegeScope, RiskLevel};
    use crate::deletion::DeletionMethod;

    fn rule(minimum_age_days: Option<u32>) -> CleanupRule {
        CleanupRule {
            id: "temp".into(),
            name: "Temp".into(),
            description: String::new(),
            locations: Vec::new(),
            risk: RiskLevel::Safe,
            scope: PrivilegeScope::User,
            method: DeletionMethod::Permanent,
            minimum_age_days,
        }
    }

    const NOW: i64 = 1_000_000_000;
    const DAY: i64 = 86_400;

    #[test]
    fn a_rule_without_an_age_limit_accepts_anything() {
        assert!(rule(None).is_old_enough(Some(NOW), NOW));
        assert!(rule(None).is_old_enough(None, NOW));
    }

    #[test]
    fn an_age_limit_keeps_recent_items() {
        let week = rule(Some(7));
        assert!(week.is_old_enough(Some(NOW - 7 * DAY), NOW));
        assert!(!week.is_old_enough(Some(NOW - 7 * DAY + 1), NOW));
        assert!(!week.is_old_enough(Some(NOW + DAY), NOW));
    }

    #[test]
    fn an_item_with_no_date_is_kept_when_there_is_an_age_limit() {
        assert!(!rule(Some(7)).is_old_enough(None, NOW));
    }
}

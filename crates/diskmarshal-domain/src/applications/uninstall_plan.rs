//! What uninstalling one app would remove.

use std::path::PathBuf;

use crate::applications::{InstalledApp, LeftoverKind};
use crate::cleanup::{PrivilegeScope, RiskLevel};
use crate::deletion::DeletionRequest;
use crate::shared::ByteSize;

/// One item that belongs to the app being uninstalled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UninstallItem {
    /// What the item is.
    pub kind: LeftoverKind,
    /// The rights removing it needs.
    pub scope: PrivilegeScope,
    /// How sure it is that the item belongs to the app.
    pub risk: RiskLevel,
    /// The removal that would take the item away.
    pub request: DeletionRequest,
}

impl UninstallItem {
    /// Whether the item is offered already selected: only sure matches are.
    #[must_use]
    pub fn is_default_choice(&self) -> bool {
        self.risk == RiskLevel::Safe
    }
}

/// The preview of an uninstall: the app and everything found that is its.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UninstallPlan {
    /// The app this plan uninstalls.
    pub app: InstalledApp,
    /// The bundle first, then what the app left behind, in catalog order.
    pub items: Vec<UninstallItem>,
    /// The folders that were searched. Nothing outside them may be removed.
    pub roots: Vec<PathBuf>,
    /// Whether another copy of the app is installed. The two share their
    /// data, so the plan then holds the bundle alone.
    pub has_other_copy: bool,
}

impl UninstallPlan {
    /// Space everything in the plan occupies.
    #[must_use]
    pub fn reclaimable(&self) -> ByteSize {
        self.items.iter().map(|item| item.request.reclaimable).sum()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{UninstallItem, UninstallPlan};
    use crate::applications::{InstalledApp, LeftoverKind};
    use crate::cleanup::{PrivilegeScope, RiskLevel};
    use crate::deletion::{DeletionMethod, DeletionRequest, ItemFingerprint};
    use crate::shared::ByteSize;

    fn item(risk: RiskLevel, size: u64) -> UninstallItem {
        UninstallItem {
            kind: LeftoverKind::Caches,
            scope: PrivilegeScope::User,
            risk,
            request: DeletionRequest {
                target: PathBuf::from(format!("/Users/tester/Library/Caches/{size}")),
                method: DeletionMethod::Trash,
                fingerprint: ItemFingerprint {
                    device: 1,
                    inode: size,
                    size,
                    modified_unix_seconds: 0,
                },
                reclaimable: ByteSize::from_bytes(size),
            },
        }
    }

    #[test]
    fn only_sure_matches_are_offered_already_selected() {
        assert!(item(RiskLevel::Safe, 1).is_default_choice());
        assert!(!item(RiskLevel::Review, 1).is_default_choice());
    }

    #[test]
    fn adds_up_everything_it_would_remove() {
        let plan = UninstallPlan {
            app: InstalledApp {
                name: "Editor".into(),
                declared_name: None,
                bundle_id: "com.vendor.editor".into(),
                version: None,
                path: PathBuf::from("/Applications/Editor.app"),
                size: ByteSize::from_bytes(700),
                scope: PrivilegeScope::User,
                is_running: false,
            },
            items: vec![item(RiskLevel::Safe, 700), item(RiskLevel::Review, 300)],
            roots: Vec::new(),
            has_other_copy: false,
        };
        assert_eq!(plan.reclaimable(), ByteSize::from_bytes(1_000));
    }
}

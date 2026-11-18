//! One folder where apps leave items named after themselves.

use std::path::PathBuf;

use crate::applications::LeftoverKind;
use crate::cleanup::PrivilegeScope;

/// A folder whose direct items are compared with the app being uninstalled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeftoverLocation {
    /// Absolute path of the folder.
    pub folder: PathBuf,
    /// What the items found here are.
    pub kind: LeftoverKind,
    /// The rights removing an item from this folder needs.
    pub scope: PrivilegeScope,
    /// Whether an item named exactly like the app counts here, besides the
    /// items named after the app's identifier.
    pub is_matched_by_name: bool,
}

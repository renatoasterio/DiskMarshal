//! Port for cleaning the areas only an administrator may change.

use std::path::PathBuf;

use diskmarshal_domain::deletion::DeletionOutcome;

/// Why a cleanup with administrator rights did not happen.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PrivilegeError {
    /// The user closed the authentication dialog of macOS.
    #[error("authentication cancelled")]
    Cancelled,
    /// This installation cannot clean with administrator rights.
    #[error("administrator cleanup is not available: {0}")]
    Unavailable(String),
    /// The cleanup started and could not finish.
    #[error("administrator cleanup failed: {0}")]
    Failed(String),
}

/// Cleans with administrator rights, after macOS authenticates the user.
///
/// The app itself never runs as an administrator and never sees the
/// password: macOS asks for it, or for Touch ID, in its own dialog.
pub trait PrivilegedCleaner: Send + Sync {
    /// Whether this installation can clean with administrator rights.
    fn is_available(&self) -> bool;

    /// Asks macOS to authenticate the user, showing `prompt` in its dialog,
    /// and then cleans what the administrator rules named in `rule_ids`
    /// select. Returns what happened to each item.
    ///
    /// # Errors
    ///
    /// Fails when the user does not authenticate, when the cleanup is not
    /// available here, or when it cannot finish.
    fn clean(
        &self,
        rule_ids: &[String],
        prompt: &str,
    ) -> Result<Vec<DeletionOutcome>, PrivilegeError>;

    /// Asks macOS to authenticate the user, showing `prompt` in its dialog,
    /// and then removes `items` of the app whose bundle identifier is
    /// `bundle_id`. Only items the uninstall of that app may remove with
    /// administrator rights are accepted: the bundle and what the app left
    /// in the system folders. They are removed for good, because there is
    /// no Trash to send them to. Returns what happened to each item.
    ///
    /// # Errors
    ///
    /// Fails when the user does not authenticate, when the removal is not
    /// available here, or when it is refused or cannot finish.
    fn uninstall(
        &self,
        bundle_id: &str,
        items: &[PathBuf],
        prompt: &str,
    ) -> Result<Vec<DeletionOutcome>, PrivilegeError>;
}

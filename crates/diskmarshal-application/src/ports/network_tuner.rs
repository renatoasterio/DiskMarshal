//! Port for changing network settings that only an administrator may change.

use diskmarshal_domain::network::NetworkChange;

use crate::ports::{AuthenticationKind, PrivilegeError};

/// Changes network settings after macOS authenticates the user.
///
/// The app itself never runs as an administrator and never sees the
/// password: macOS asks for it, or for Touch ID, in its own dialog.
pub trait NetworkTuner: Send + Sync {
    /// Whether this installation can change network settings.
    fn is_available(&self) -> bool;

    /// What macOS will ask the user for. The password, unless the adapter
    /// knows this Mac is set up to take the fingerprint instead.
    fn authentication(&self) -> AuthenticationKind {
        AuthenticationKind::Password
    }

    /// Asks macOS to authenticate the user once, showing `prompt` in its
    /// dialog, and then applies `changes` in order.
    ///
    /// # Errors
    ///
    /// Fails when the user does not authenticate, when changing settings is
    /// not available here, or when a change is refused. Changes before the
    /// refused one stay applied.
    fn apply(&self, changes: &[NetworkChange], prompt: &str) -> Result<(), PrivilegeError>;
}

//! Port for changing the network settings of this Mac.

use diskmarshal_domain::network::NetworkChange;

/// Why a network setting was not changed; the text is what macOS said.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("network setting not changed: {0}")]
pub struct NetworkSettingsError(pub String);

/// Changes network settings directly, with the rights of whoever runs it.
///
/// Most changes need an administrator, so this is what the privileged
/// helper plugs in. The app itself goes through
/// [`NetworkTuner`](crate::ports::NetworkTuner), which authenticates the
/// user first.
pub trait NetworkSettings: Send + Sync {
    /// The names of the network services of this Mac, such as `Wi-Fi`.
    ///
    /// # Errors
    ///
    /// Fails when macOS cannot list them.
    fn services(&self) -> Result<Vec<String>, NetworkSettingsError>;

    /// Applies one change.
    ///
    /// # Errors
    ///
    /// Fails when macOS refuses the change.
    fn apply(&self, change: &NetworkChange) -> Result<(), NetworkSettingsError>;
}

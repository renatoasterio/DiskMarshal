//! Port for reading how this Mac is set up to reach the internet.

use diskmarshal_domain::network::ConnectionProfile;

/// Why the settings of the connection could not be read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InspectionError {
    /// This Mac has no connection to inspect: there is no way out to a
    /// network.
    #[error("this Mac is not connected to a network")]
    Offline,
    /// The settings could not be read.
    #[error("the network settings cannot be read: {0}")]
    Unavailable(String),
}

/// Reads the network settings and the state of the wireless link.
///
/// Implementations read what macOS already knows and change nothing. The
/// one question they may put to the network is whether it answers the
/// search for a proxy, and only while macOS itself is making that search.
/// A part that cannot be read is left empty rather than failing the whole
/// reading.
pub trait ConnectionInspector: Send + Sync {
    /// Reads the interface in use, its addresses, name servers, proxy
    /// settings, VPN profiles and, on a wireless interface, the link.
    ///
    /// # Errors
    ///
    /// Fails when this Mac has no way out to a network, or when not even
    /// that can be found out.
    fn profile(&self) -> Result<ConnectionProfile, InspectionError>;
}

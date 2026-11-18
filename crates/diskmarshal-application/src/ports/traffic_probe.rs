//! Port for reading what macOS counts on every open connection.

use diskmarshal_domain::network::TrafficReading;

/// Why the traffic could not be read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TrafficProbeError {
    /// This Mac has no way to read the counters.
    #[error("network traffic cannot be read here: {0}")]
    Unavailable(String),
    /// The counters did not arrive within the time limit.
    #[error("reading the network traffic took too long")]
    TimedOut,
    /// The counters arrived in a form this app does not understand.
    #[error("the network traffic came in an unknown form: {0}")]
    Unreadable(String),
}

/// Reads the counters macOS keeps for the connections of every program.
///
/// Implementations only read counters: they never look inside a packet,
/// never open a connection of their own and change nothing.
pub trait TrafficProbe: Send + Sync {
    /// Takes one reading of every open connection that leaves this Mac.
    ///
    /// # Errors
    ///
    /// Fails when the counters cannot be read, do not arrive in time, or
    /// arrive in a form this app does not understand.
    fn read(&self) -> Result<TrafficReading, TrafficProbeError>;
}

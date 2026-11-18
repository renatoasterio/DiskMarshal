//! Port for timing how long the other side takes to answer.

use std::net::IpAddr;
use std::time::Duration;

use diskmarshal_domain::network::PingResult;

/// Sends a few small packets and times the answers.
///
/// Unlike reading the settings, this uses the network: a handful of pings
/// and name lookups, a few kilobytes in all. Implementations run only when
/// asked.
pub trait ReachabilityProbe: Send + Sync {
    /// Pings `target` a few times. When the pings cannot be sent at all the
    /// result reports none sent.
    fn ping(&self, target: IpAddr) -> PingResult;

    /// Asks the name server at `server` for a well-known name and returns
    /// how long it took to answer, or `None` when it did not.
    fn time_dns(&self, server: IpAddr) -> Option<Duration>;
}

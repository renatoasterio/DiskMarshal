//! What macOS had counted on every connection at one moment.

use std::time::Duration;

use crate::network::Connection;

/// The connections of one running process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessTraffic {
    /// Identifier of the process.
    pub pid: u32,
    /// The program the process belongs to, as people know it.
    pub program: String,
    /// Its open connections, each with what it moved since it was opened.
    pub connections: Vec<Connection>,
}

/// One reading of the counters macOS keeps for every open connection.
///
/// A counter only grows while its connection lives, so a speed is the
/// difference between two readings. [`TrafficMeter`](crate::network::TrafficMeter)
/// does that.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrafficReading {
    /// When the reading was taken, on a clock that never goes back. Only
    /// the difference between two readings means something.
    pub taken_at: Duration,
    /// Every process with a connection.
    pub processes: Vec<ProcessTraffic>,
}

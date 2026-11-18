//! The traffic of this Mac at one moment, as it is shown.

use crate::network::{BitRate, Connection};
use crate::shared::ByteSize;

/// What one program is doing on the network.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramTraffic {
    /// The program, as people know it.
    pub name: String,
    /// How fast it is receiving right now.
    pub download: BitRate,
    /// How fast it is sending right now.
    pub upload: BitRate,
    /// What its open connections received since they were opened.
    pub received: ByteSize,
    /// What its open connections sent since they were opened.
    pub sent: ByteSize,
    /// Its connections that moved data, the ones that moved the most first.
    pub connections: Vec<Connection>,
}

impl ProgramTraffic {
    /// Everything the program is moving right now, both ways.
    #[must_use]
    pub fn rate(&self) -> BitRate {
        self.download + self.upload
    }

    /// Everything its open connections moved, both ways.
    #[must_use]
    pub fn moved(&self) -> ByteSize {
        self.received + self.sent
    }
}

/// Who is using the network and how fast, between two readings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrafficSnapshot {
    /// How fast this Mac is receiving right now.
    pub download: BitRate,
    /// How fast this Mac is sending right now.
    pub upload: BitRate,
    /// Received since the metering began.
    pub received: ByteSize,
    /// Sent since the metering began.
    pub sent: ByteSize,
    /// The fastest download seen since the metering began.
    pub peak_download: BitRate,
    /// The programs with traffic, the busiest right now first.
    pub programs: Vec<ProgramTraffic>,
}

impl TrafficSnapshot {
    /// How many connections moved data.
    #[must_use]
    pub fn connection_count(&self) -> usize {
        self.programs
            .iter()
            .map(|program| program.connections.len())
            .sum()
    }

    /// Share of what the open connections sent that had to be sent again,
    /// from 0.0 to 1.0. A high share means packets are being lost on the way.
    #[must_use]
    pub fn retransmitted_ratio(&self) -> f64 {
        let connections = || {
            self.programs
                .iter()
                .flat_map(|program| &program.connections)
        };
        let sent: ByteSize = connections().map(|connection| connection.sent).sum();
        if sent.is_zero() {
            return 0.0;
        }
        let again: ByteSize = connections()
            .map(|connection| connection.retransmitted)
            .sum();
        (again.as_bytes() as f64 / sent.as_bytes() as f64).clamp(0.0, 1.0)
    }
}

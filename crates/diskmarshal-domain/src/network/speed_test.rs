//! What a speed test found.

use std::time::Duration;

use crate::network::BitRate;
use crate::shared::ByteSize;

/// Below this many round trips per minute a loaded connection feels stuck.
const LOW_BELOW_RPM: u32 = 200;
/// From this many round trips per minute a loaded connection stays smooth.
const HIGH_FROM_RPM: u32 = 800;

/// How well a connection keeps answering while it is busy, in the three
/// steps the interface tells apart. The steps are this app's own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResponsivenessLevel {
    /// Calls and games stall whenever something else uses the connection.
    Low,
    /// Short pauses show up when several things share the connection.
    Medium,
    /// Stays smooth whatever else is using the connection.
    High,
}

/// How many round trips per minute the connection completed while the test
/// kept it fully loaded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Responsiveness(u32);

impl Responsiveness {
    /// Creates the figure from round trips per minute.
    #[must_use]
    pub const fn from_rpm(rpm: u32) -> Self {
        Self(rpm)
    }

    /// Round trips per minute.
    #[must_use]
    pub const fn rpm(self) -> u32 {
        self.0
    }

    /// Which of the three steps the figure falls in.
    #[must_use]
    pub const fn level(self) -> ResponsivenessLevel {
        if self.0 >= HIGH_FROM_RPM {
            ResponsivenessLevel::High
        } else if self.0 >= LOW_BELOW_RPM {
            ResponsivenessLevel::Medium
        } else {
            ResponsivenessLevel::Low
        }
    }
}

/// The result of one speed test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpeedTest {
    /// How fast data arrived.
    pub download: BitRate,
    /// How fast data left.
    pub upload: BitRate,
    /// Time the server took to answer while nothing else was moving, when
    /// the test reported it.
    pub idle_latency: Option<Duration>,
    /// How well the connection kept answering under load, when measured.
    pub responsiveness: Option<Responsiveness>,
    /// Everything the test moved, both ways. A test spends real data.
    pub transferred: ByteSize,
    /// How long the test took.
    pub duration: Duration,
}

#[cfg(test)]
mod tests {
    use super::{Responsiveness, ResponsivenessLevel};

    #[test]
    fn classifies_responsiveness_at_the_thresholds() {
        let level = |rpm| Responsiveness::from_rpm(rpm).level();
        assert_eq!(level(0), ResponsivenessLevel::Low);
        assert_eq!(level(199), ResponsivenessLevel::Low);
        assert_eq!(level(200), ResponsivenessLevel::Medium);
        assert_eq!(level(799), ResponsivenessLevel::Medium);
        assert_eq!(level(800), ResponsivenessLevel::High);
        assert_eq!(Responsiveness::from_rpm(440).rpm(), 440);
    }
}

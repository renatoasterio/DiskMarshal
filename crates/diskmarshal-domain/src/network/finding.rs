//! What the analysis of the connection concluded, one fact at a time.

use std::time::Duration;

use crate::network::{
    BitRate, ResponsivenessLevel, SignalQuality, SpeedTest, TrafficSnapshot, WifiBand,
};

/// Below this total speed no program counts as taking over the connection.
const HEAVY_FROM: BitRate = BitRate::from_bits_per_second(1_000_000);
/// From this share of the traffic up, in percent, one program is taking
/// over the connection.
const HEAVY_FROM_PERCENT: u64 = 50;

/// How much a finding asks of the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Something is costing speed and can be dealt with.
    Attention,
    /// Worth knowing; nothing is wrong.
    Info,
    /// Checked and fine.
    Good,
}

/// Which way data is moving.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Arriving at this Mac.
    Download,
    /// Leaving this Mac.
    Upload,
}

/// One conclusion of the analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Finding {
    /// macOS looks for a proxy before each new connection and the network
    /// never answers, so every new connection waits for nothing.
    ProxyDiscoveryUnanswered,
    /// macOS looks for a proxy before each new connection.
    ProxyDiscoveryOn,
    /// The name server in use is much slower than a public one.
    SlowDns {
        /// Answer time of the server in use.
        current: Duration,
        /// Answer time of the fastest public server measured.
        fastest: Duration,
    },
    /// Packets sent to the internet got no answer.
    PacketLoss {
        /// Packets sent.
        sent: u32,
        /// Packets without an answer.
        lost: u32,
    },
    /// Every packet sent to the internet was answered.
    NoPacketLoss {
        /// Packets sent.
        sent: u32,
    },
    /// The internet takes long to answer.
    SlowAnswer {
        /// Average answer time.
        latency: Duration,
    },
    /// How good the wireless signal is.
    WifiSignal {
        /// The step the signal falls in.
        quality: SignalQuality,
        /// The signal, in dBm.
        signal_dbm: i32,
        /// The band in use, when known.
        band: Option<WifiBand>,
        /// The Wi-Fi generation in use, when known.
        generation: Option<&'static str>,
    },
    /// A VPN is connected, so everything takes a longer way.
    VpnConnected,
    /// One program accounts for most of the traffic.
    HeavyProgram {
        /// The program.
        name: String,
        /// Which way it is moving data.
        direction: Direction,
        /// How fast.
        rate: BitRate,
        /// Its share of everything moving that way, in percent.
        share_percent: u8,
    },
    /// How well the connection kept answering during a speed test.
    Responsiveness {
        /// The step the figure falls in.
        level: ResponsivenessLevel,
        /// Round trips per minute.
        rpm: u32,
    },
}

impl Finding {
    /// How much the finding asks of the user.
    #[must_use]
    pub const fn severity(&self) -> Severity {
        match self {
            Self::ProxyDiscoveryUnanswered
            | Self::SlowDns { .. }
            | Self::PacketLoss { .. }
            | Self::SlowAnswer { .. } => Severity::Attention,
            Self::ProxyDiscoveryOn | Self::VpnConnected | Self::HeavyProgram { .. } => {
                Severity::Info
            }
            Self::NoPacketLoss { .. } => Severity::Good,
            Self::WifiSignal { quality, .. } => match quality {
                SignalQuality::Strong => Severity::Good,
                SignalQuality::Fair => Severity::Info,
                SignalQuality::Weak => Severity::Attention,
            },
            Self::Responsiveness { level, .. } => match level {
                ResponsivenessLevel::High | ResponsivenessLevel::Medium => Severity::Good,
                ResponsivenessLevel::Low => Severity::Attention,
            },
        }
    }

    /// The program that accounts for most of what is moving right now, when
    /// one does. Sending is looked at first: a full upload stalls
    /// everything else.
    #[must_use]
    pub fn of_traffic(snapshot: &TrafficSnapshot) -> Option<Self> {
        [Direction::Upload, Direction::Download]
            .into_iter()
            .find_map(|direction| {
                let rate_of = |download: BitRate, upload: BitRate| match direction {
                    Direction::Download => download,
                    Direction::Upload => upload,
                };
                let total = rate_of(snapshot.download, snapshot.upload);
                let busiest = snapshot
                    .programs
                    .iter()
                    .max_by_key(|program| rate_of(program.download, program.upload))?;
                let rate = rate_of(busiest.download, busiest.upload);
                let share = u128::from(rate.as_bits_per_second()) * 100
                    / u128::from(total.as_bits_per_second().max(1));
                let share_percent = u8::try_from(share.min(100)).unwrap_or(100);
                (total >= HEAVY_FROM && u64::from(share_percent) >= HEAVY_FROM_PERCENT).then(|| {
                    Self::HeavyProgram {
                        name: busiest.name.clone(),
                        direction,
                        rate,
                        share_percent,
                    }
                })
            })
    }

    /// What a speed test says about how the connection behaves when busy.
    #[must_use]
    pub fn of_speed_test(test: &SpeedTest) -> Option<Self> {
        test.responsiveness
            .map(|responsiveness| Self::Responsiveness {
                level: responsiveness.level(),
                rpm: responsiveness.rpm(),
            })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{Direction, Finding, Severity};
    use crate::network::{
        BitRate, ProgramTraffic, Responsiveness, SignalQuality, SpeedTest, TrafficSnapshot,
    };
    use crate::shared::ByteSize;

    fn megabits(value: u64) -> BitRate {
        BitRate::from_bits_per_second(value * 1_000_000)
    }

    fn program(name: &str, download: u64, upload: u64) -> ProgramTraffic {
        ProgramTraffic {
            name: name.to_owned(),
            download: megabits(download),
            upload: megabits(upload),
            received: ByteSize::ZERO,
            sent: ByteSize::ZERO,
            connections: Vec::new(),
        }
    }

    fn snapshot(programs: Vec<ProgramTraffic>) -> TrafficSnapshot {
        TrafficSnapshot {
            download: programs.iter().map(|program| program.download).sum(),
            upload: programs.iter().map(|program| program.upload).sum(),
            programs,
            ..TrafficSnapshot::default()
        }
    }

    #[test]
    fn names_the_program_that_takes_most_of_the_upload() {
        let finding = Finding::of_traffic(&snapshot(vec![
            program("Browser", 40, 2),
            program("Backup", 0, 11),
        ]))
        .unwrap();

        assert_eq!(
            finding,
            Finding::HeavyProgram {
                name: "Backup".to_owned(),
                direction: Direction::Upload,
                rate: megabits(11),
                share_percent: 84,
            }
        );
        assert_eq!(finding.severity(), Severity::Info);
    }

    #[test]
    fn falls_back_to_the_download_and_ignores_a_quiet_connection() {
        let busy = snapshot(vec![program("Browser", 40, 0), program("Music", 2, 0)]);
        assert!(matches!(
            Finding::of_traffic(&busy),
            Some(Finding::HeavyProgram {
                direction: Direction::Download,
                share_percent: 95,
                ..
            })
        ));
        let shared = snapshot(vec![
            program("One", 10, 0),
            program("Two", 11, 0),
            program("Three", 9, 0),
        ]);
        assert_eq!(Finding::of_traffic(&shared), None);
        let quiet = TrafficSnapshot {
            download: BitRate::from_bits_per_second(900_000),
            programs: vec![ProgramTraffic {
                download: BitRate::from_bits_per_second(900_000),
                ..program("Browser", 0, 0)
            }],
            ..TrafficSnapshot::default()
        };
        assert_eq!(Finding::of_traffic(&quiet), None);
        assert_eq!(Finding::of_traffic(&TrafficSnapshot::default()), None);
    }

    #[test]
    fn a_speed_test_tells_how_the_connection_behaves_when_busy() {
        let test = |rpm: Option<u32>| SpeedTest {
            download: megabits(468),
            upload: megabits(383),
            idle_latency: Some(Duration::from_millis(13)),
            responsiveness: rpm.map(Responsiveness::from_rpm),
            transferred: ByteSize::ZERO,
            duration: Duration::from_secs(18),
        };
        assert_eq!(
            Finding::of_speed_test(&test(Some(440))).unwrap().severity(),
            Severity::Good
        );
        assert_eq!(
            Finding::of_speed_test(&test(Some(90))).unwrap().severity(),
            Severity::Attention
        );
        assert_eq!(Finding::of_speed_test(&test(None)), None);
    }

    #[test]
    fn a_weak_signal_asks_for_attention_and_a_strong_one_does_not() {
        let signal = |quality| Finding::WifiSignal {
            quality,
            signal_dbm: -50,
            band: None,
            generation: None,
        };
        assert_eq!(signal(SignalQuality::Strong).severity(), Severity::Good);
        assert_eq!(signal(SignalQuality::Fair).severity(), Severity::Info);
        assert_eq!(signal(SignalQuality::Weak).severity(), Severity::Attention);
        assert!(Severity::Attention < Severity::Info && Severity::Info < Severity::Good);
    }
}

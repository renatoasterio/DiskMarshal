//! Turning successive readings of the counters into speeds.

use std::cmp::Reverse;
use std::collections::HashMap;
use std::time::Duration;

use crate::network::{
    BitRate, Connection, Endpoint, ProgramTraffic, TrafficReading, TrafficSnapshot, Transport,
};
use crate::shared::ByteSize;

/// What tells one connection from another between two readings.
type ConnectionKey = (u32, Transport, Endpoint, Endpoint);

/// Bytes moved in each direction.
#[derive(Debug, Clone, Copy, Default)]
struct Moved {
    received: ByteSize,
    sent: ByteSize,
}

impl Moved {
    fn add(&mut self, other: Self) {
        self.received += other.received;
        self.sent += other.sent;
    }

    /// What was moved since `before`. A counter that went down belongs to a
    /// new connection that took the place of a closed one, so it counts
    /// from zero.
    fn since(self, before: Self) -> Self {
        let grown = |now: ByteSize, before: ByteSize| {
            if now >= before {
                now.saturating_sub(before)
            } else {
                now
            }
        };
        Self {
            received: grown(self.received, before.received),
            sent: grown(self.sent, before.sent),
        }
    }
}

/// The reading the next one is compared with.
#[derive(Debug)]
struct Baseline {
    taken_at: Duration,
    counters: HashMap<ConnectionKey, Moved>,
}

/// Compares each reading with the one before it.
///
/// macOS counts what every connection moved since it was opened. The meter
/// keeps the previous counters and reports the difference as a speed, per
/// program. The first reading only sets the starting point, so it reports
/// no speed. A connection that was opened and closed between two readings is
/// never seen: very short transfers are undercounted.
#[derive(Debug, Default)]
pub struct TrafficMeter {
    baseline: Option<Baseline>,
    received: ByteSize,
    sent: ByteSize,
    peak_download: BitRate,
}

impl TrafficMeter {
    /// Takes in a reading and reports the traffic since the previous one.
    pub fn record(&mut self, reading: &TrafficReading) -> TrafficSnapshot {
        let elapsed = self
            .baseline
            .as_ref()
            .map(|baseline| reading.taken_at.saturating_sub(baseline.taken_at));
        let mut counters: HashMap<ConnectionKey, Moved> = HashMap::new();
        let mut programs: Vec<ProgramTraffic> = Vec::new();
        let mut moved: Vec<Moved> = Vec::new();
        let mut index_of: HashMap<&str, usize> = HashMap::new();

        for process in &reading.processes {
            let active: Vec<&Connection> = process
                .connections
                .iter()
                .filter(|connection| connection.has_traffic())
                .collect();
            if active.is_empty() {
                continue;
            }
            let index = *index_of.entry(process.program.as_str()).or_insert_with(|| {
                programs.push(empty_program(&process.program));
                moved.push(Moved::default());
                programs.len() - 1
            });

            // Sockets that are not tied to an address share one key.
            let mut by_key: HashMap<ConnectionKey, Moved> = HashMap::new();
            for connection in active {
                by_key
                    .entry(key_of(process.pid, connection))
                    .or_default()
                    .add(Moved {
                        received: connection.received,
                        sent: connection.sent,
                    });
                let program = &mut programs[index];
                program.received += connection.received;
                program.sent += connection.sent;
                program.connections.push(connection.clone());
            }
            for (key, now) in by_key {
                if let Some(baseline) = &self.baseline {
                    // A connection that was not there before was opened
                    // since then: everything it moved is new.
                    let before = baseline.counters.get(&key).copied().unwrap_or_default();
                    moved[index].add(now.since(before));
                }
                counters.insert(key, now);
            }
        }

        let elapsed = elapsed.unwrap_or_default();
        let mut total = Moved::default();
        for (program, moved) in programs.iter_mut().zip(&moved) {
            program.download = BitRate::of(moved.received, elapsed);
            program.upload = BitRate::of(moved.sent, elapsed);
            program
                .connections
                .sort_by_key(|connection| Reverse(connection.received + connection.sent));
            total.add(*moved);
        }
        programs.sort_by(|a, b| {
            (Reverse(a.rate()), Reverse(a.moved()), &a.name).cmp(&(
                Reverse(b.rate()),
                Reverse(b.moved()),
                &b.name,
            ))
        });

        self.baseline = Some(Baseline {
            taken_at: reading.taken_at,
            counters,
        });
        self.received += total.received;
        self.sent += total.sent;
        let download = BitRate::of(total.received, elapsed);
        self.peak_download = self.peak_download.max(download);
        TrafficSnapshot {
            download,
            upload: BitRate::of(total.sent, elapsed),
            received: self.received,
            sent: self.sent,
            peak_download: self.peak_download,
            programs,
        }
    }
}

fn empty_program(name: &str) -> ProgramTraffic {
    ProgramTraffic {
        name: name.to_owned(),
        download: BitRate::ZERO,
        upload: BitRate::ZERO,
        received: ByteSize::ZERO,
        sent: ByteSize::ZERO,
        connections: Vec::new(),
    }
}

fn key_of(pid: u32, connection: &Connection) -> ConnectionKey {
    (
        pid,
        connection.transport,
        connection.local.clone(),
        connection.remote.clone(),
    )
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::TrafficMeter;
    use crate::network::{
        BitRate, Connection, Endpoint, ProcessTraffic, TrafficReading, Transport,
    };
    use crate::shared::ByteSize;

    fn connection(remote_port: u16, received: u64, sent: u64) -> Connection {
        Connection {
            transport: Transport::Tcp,
            local: Endpoint {
                address: "192.168.0.24".to_owned(),
                port: Some(50_000),
            },
            remote: Endpoint {
                address: "203.0.113.24".to_owned(),
                port: Some(remote_port),
            },
            interface: "en0".to_owned(),
            received: ByteSize::from_bytes(received),
            sent: ByteSize::from_bytes(sent),
            retransmitted: ByteSize::ZERO,
            round_trip: None,
        }
    }

    fn process(pid: u32, program: &str, connections: Vec<Connection>) -> ProcessTraffic {
        ProcessTraffic {
            pid,
            program: program.to_owned(),
            connections,
        }
    }

    fn reading(seconds: u64, processes: Vec<ProcessTraffic>) -> TrafficReading {
        TrafficReading {
            taken_at: Duration::from_secs(seconds),
            processes,
        }
    }

    fn megabits(value: u64) -> BitRate {
        BitRate::from_bits_per_second(value * 1_000_000)
    }

    #[test]
    fn the_first_reading_sets_the_starting_point_and_reports_no_speed() {
        let mut meter = TrafficMeter::default();
        let snapshot = meter.record(&reading(
            10,
            vec![process(
                7,
                "Browser",
                vec![connection(443, 9_000_000, 1_000)],
            )],
        ));

        assert!(snapshot.download.is_zero() && snapshot.upload.is_zero());
        assert!(snapshot.received.is_zero());
        assert_eq!(snapshot.programs.len(), 1);
        assert_eq!(
            snapshot.programs[0].received,
            ByteSize::from_bytes(9_000_000)
        );
    }

    #[test]
    fn speed_is_what_the_counters_grew_over_the_time_between_readings() {
        let mut meter = TrafficMeter::default();
        meter.record(&reading(
            10,
            vec![process(7, "Browser", vec![connection(443, 1_000_000, 0)])],
        ));
        let snapshot = meter.record(&reading(
            12,
            vec![process(
                7,
                "Browser",
                vec![connection(443, 3_000_000, 250_000)],
            )],
        ));

        assert_eq!(snapshot.download, megabits(8));
        assert_eq!(snapshot.upload, megabits(1));
        assert_eq!(snapshot.received, ByteSize::from_bytes(2_000_000));
        assert_eq!(snapshot.sent, ByteSize::from_bytes(250_000));
        assert_eq!(snapshot.peak_download, megabits(8));
    }

    #[test]
    fn the_processes_of_one_program_are_shown_as_one_and_the_busiest_comes_first() {
        let mut meter = TrafficMeter::default();
        let before = vec![
            process(7, "Browser", vec![connection(443, 0, 10)]),
            process(8, "Browser", vec![connection(443, 0, 10)]),
            process(9, "Backup", vec![connection(443, 10, 0)]),
        ];
        let after = vec![
            process(7, "Browser", vec![connection(443, 125_000, 10)]),
            process(8, "Browser", vec![connection(443, 125_000, 10)]),
            process(9, "Backup", vec![connection(443, 10, 1_000_000)]),
        ];
        meter.record(&reading(0, before));
        let snapshot = meter.record(&reading(1, after));

        let names: Vec<&str> = snapshot
            .programs
            .iter()
            .map(|program| program.name.as_str())
            .collect();
        assert_eq!(names, ["Backup", "Browser"]);
        assert_eq!(snapshot.programs[0].upload, megabits(8));
        assert_eq!(snapshot.programs[1].download, megabits(2));
        assert_eq!(snapshot.programs[1].connections.len(), 2);
        assert_eq!(snapshot.connection_count(), 3);
    }

    #[test]
    fn a_connection_opened_between_readings_counts_in_full() {
        let mut meter = TrafficMeter::default();
        meter.record(&reading(
            0,
            vec![process(7, "Browser", vec![connection(443, 100, 0)])],
        ));
        let snapshot = meter.record(&reading(
            1,
            vec![process(
                7,
                "Browser",
                vec![connection(443, 100, 0), connection(8443, 500_000, 0)],
            )],
        ));

        assert_eq!(snapshot.download, megabits(4));
    }

    #[test]
    fn a_counter_that_went_down_is_a_new_connection_counted_from_zero() {
        let mut meter = TrafficMeter::default();
        meter.record(&reading(
            0,
            vec![process(7, "Browser", vec![connection(443, 9_000_000, 0)])],
        ));
        let snapshot = meter.record(&reading(
            1,
            vec![process(7, "Browser", vec![connection(443, 250_000, 0)])],
        ));

        assert_eq!(snapshot.download, megabits(2));
    }

    #[test]
    fn connections_that_never_moved_a_byte_are_left_out() {
        let mut meter = TrafficMeter::default();
        let snapshot = meter.record(&reading(
            0,
            vec![
                process(1, "Listener", vec![connection(0, 0, 0)]),
                process(2, "Idle", Vec::new()),
            ],
        ));
        assert!(snapshot.programs.is_empty());
        assert_eq!(snapshot.connection_count(), 0);
    }

    #[test]
    fn says_how_much_of_what_was_sent_had_to_be_sent_again() {
        let mut lossy = connection(443, 0, 1_000);
        lossy.retransmitted = ByteSize::from_bytes(50);
        let mut meter = TrafficMeter::default();
        let snapshot = meter.record(&reading(0, vec![process(7, "Browser", vec![lossy])]));

        assert!((snapshot.retransmitted_ratio() - 0.05).abs() < 1e-9);
        assert!(
            TrafficMeter::default()
                .record(&reading(0, Vec::new()))
                .retransmitted_ratio()
                < f64::EPSILON
        );
    }

    #[test]
    fn readings_taken_at_the_same_instant_report_no_speed() {
        let mut meter = TrafficMeter::default();
        let same = reading(5, vec![process(7, "Browser", vec![connection(443, 1, 1)])]);
        meter.record(&same);
        assert!(meter.record(&same).download.is_zero());
    }
}

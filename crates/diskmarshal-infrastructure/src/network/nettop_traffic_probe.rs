//! Traffic probe that asks `nettop`, the program macOS ships for this.

use std::process::Command;
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use diskmarshal_application::ports::{TrafficProbe, TrafficProbeError};
use diskmarshal_domain::network::{ProcessTraffic, TrafficReading};

use crate::network::nettop_report;
use crate::network::program_names::ProgramNames;
use crate::process::bounded_command::{CommandFailure, Stdout, run_bounded};

/// Absolute path, so a different program earlier in `PATH` can never run.
const NETTOP: &str = "/usr/bin/nettop";
/// `-L 1` prints one sample as CSV and ends. `-x` writes plain numbers.
/// `-n` leaves addresses as numbers, so no name is looked up on the network.
/// `-t external` leaves out what never leaves this Mac (the loopback
/// interface).
const ARGUMENTS: [&str; 6] = ["-L", "1", "-x", "-n", "-t", "external"];
/// One sample takes a fraction of a second.
const TIMEOUT: Duration = Duration::from_secs(5);

/// Reads the counters of every connection with `/usr/bin/nettop`.
///
/// The program runs as the logged-in user, with no shell in between, a
/// cleared environment and no standard input. It reads counters the system
/// already keeps: nothing is captured, sent or changed.
pub struct NettopTrafficProbe {
    /// Readings are timed from here, on a clock that never goes back.
    started: Instant,
    programs: Mutex<ProgramNames>,
}

impl NettopTrafficProbe {
    /// Creates the probe.
    #[must_use]
    pub fn new() -> Self {
        Self {
            started: Instant::now(),
            programs: Mutex::new(ProgramNames::new()),
        }
    }
}

impl Default for NettopTrafficProbe {
    fn default() -> Self {
        Self::new()
    }
}

impl TrafficProbe for NettopTrafficProbe {
    fn read(&self) -> Result<TrafficReading, TrafficProbeError> {
        let mut command = Command::new(NETTOP);
        command.args(ARGUMENTS).env_clear();
        let output =
            run_bounded(&mut command, Stdout::Capture, TIMEOUT).map_err(
                |failure| match failure {
                    CommandFailure::NotStarted(error) => {
                        TrafficProbeError::Unavailable(error.to_string())
                    }
                    CommandFailure::TimedOut => TrafficProbeError::TimedOut,
                },
            )?;
        let taken_at = self.started.elapsed();
        if output.code != Some(0) {
            return Err(TrafficProbeError::Unavailable(output.first_error_line()));
        }
        let rows = nettop_report::parse(&String::from_utf8_lossy(&output.stdout))
            .map_err(TrafficProbeError::Unreadable)?;

        let mut programs = self.programs.lock().unwrap_or_else(PoisonError::into_inner);
        let running: Vec<(u32, &str)> = rows
            .iter()
            .filter(|row| !row.connections.is_empty())
            .map(|row| (row.pid, row.name.as_str()))
            .collect();
        programs.refresh(&running);
        let processes = rows
            .iter()
            .filter(|row| !row.connections.is_empty())
            .map(|row| ProcessTraffic {
                pid: row.pid,
                program: programs.of(row.pid).unwrap_or(&row.name).to_owned(),
                connections: row.connections.clone(),
            })
            .collect();
        Ok(TrafficReading {
            taken_at,
            processes,
        })
    }
}

#[cfg(test)]
mod tests {
    use diskmarshal_application::ports::TrafficProbe;

    use super::NettopTrafficProbe;

    #[test]
    fn reads_the_connections_of_this_machine_and_keeps_time() {
        let probe = NettopTrafficProbe::new();

        let first = probe.read().unwrap();
        let second = probe.read().unwrap();

        assert!(second.taken_at > first.taken_at);
        for process in first.processes.iter().chain(&second.processes) {
            assert!(!process.program.is_empty());
            assert!(!process.connections.is_empty());
            assert!(
                process
                    .connections
                    .iter()
                    .all(|connection| connection.interface != "lo0"),
                "what never leaves this Mac is left out"
            );
        }
    }
}

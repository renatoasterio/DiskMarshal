//! Use case: find out who is using the network and how fast. Changes nothing.

use diskmarshal_domain::network::{TrafficMeter, TrafficSnapshot};

use crate::ports::{TrafficProbe, TrafficProbeError};

/// Reads the connection counters and turns them into speeds per program.
pub struct MeasureTraffic<'a> {
    probe: &'a dyn TrafficProbe,
}

impl<'a> MeasureTraffic<'a> {
    /// Creates the use case over the source of the counters.
    #[must_use]
    pub fn new(probe: &'a dyn TrafficProbe) -> Self {
        Self { probe }
    }

    /// Takes one reading and reports the traffic since the reading `meter`
    /// saw before it. The caller keeps `meter` between calls; with a new
    /// one, the first call sets the starting point and reports no speed.
    ///
    /// # Errors
    ///
    /// Fails when the counters cannot be read. `meter` is then left as it
    /// was, so the next reading still compares with the last good one.
    pub fn execute(&self, meter: &mut TrafficMeter) -> Result<TrafficSnapshot, TrafficProbeError> {
        let reading = self.probe.read()?;
        Ok(meter.record(&reading))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::time::Duration;

    use diskmarshal_domain::network::{
        BitRate, Connection, Endpoint, ProcessTraffic, TrafficMeter, TrafficReading, Transport,
    };
    use diskmarshal_domain::shared::ByteSize;

    use super::MeasureTraffic;
    use crate::ports::{TrafficProbe, TrafficProbeError};

    /// A probe that hands out prepared answers, one per call.
    struct ScriptedProbe(Mutex<Vec<Result<TrafficReading, TrafficProbeError>>>);

    impl TrafficProbe for ScriptedProbe {
        fn read(&self) -> Result<TrafficReading, TrafficProbeError> {
            self.0.lock().unwrap().remove(0)
        }
    }

    fn reading(seconds: u64, received: u64) -> TrafficReading {
        TrafficReading {
            taken_at: Duration::from_secs(seconds),
            processes: vec![ProcessTraffic {
                pid: 7,
                program: "Browser".to_owned(),
                connections: vec![Connection {
                    transport: Transport::Tcp,
                    local: Endpoint::default(),
                    remote: Endpoint {
                        address: "203.0.113.24".to_owned(),
                        port: Some(443),
                    },
                    interface: "en0".to_owned(),
                    received: ByteSize::from_bytes(received),
                    sent: ByteSize::ZERO,
                    retransmitted: ByteSize::ZERO,
                    round_trip: None,
                }],
            }],
        }
    }

    #[test]
    fn reports_the_speed_between_two_readings() {
        let probe = ScriptedProbe(Mutex::new(vec![
            Ok(reading(0, 1_000_000)),
            Ok(reading(1, 2_000_000)),
        ]));
        let measure = MeasureTraffic::new(&probe);
        let mut meter = TrafficMeter::default();

        assert!(measure.execute(&mut meter).unwrap().download.is_zero());
        let snapshot = measure.execute(&mut meter).unwrap();

        assert_eq!(snapshot.download, BitRate::from_bits_per_second(8_000_000));
        assert_eq!(snapshot.programs[0].name, "Browser");
    }

    #[test]
    fn a_failed_reading_leaves_the_meter_comparing_with_the_last_good_one() {
        let probe = ScriptedProbe(Mutex::new(vec![
            Ok(reading(0, 1_000_000)),
            Err(TrafficProbeError::TimedOut),
            Ok(reading(2, 3_000_000)),
        ]));
        let measure = MeasureTraffic::new(&probe);
        let mut meter = TrafficMeter::default();

        measure.execute(&mut meter).unwrap();
        assert_eq!(
            measure.execute(&mut meter).unwrap_err(),
            TrafficProbeError::TimedOut
        );
        let snapshot = measure.execute(&mut meter).unwrap();

        assert_eq!(snapshot.download, BitRate::from_bits_per_second(8_000_000));
    }
}

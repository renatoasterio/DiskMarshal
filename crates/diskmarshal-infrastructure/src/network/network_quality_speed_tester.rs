//! Speed tester that asks `networkQuality`, the program macOS ships for this.

use std::process::Command;
use std::time::{Duration, Instant};

use diskmarshal_application::ports::{SpeedTestError, SpeedTester};
use diskmarshal_domain::network::{BitRate, Responsiveness, SpeedTest};
use diskmarshal_domain::shared::ByteSize;
use serde::{Deserialize, Deserializer};

use crate::process::bounded_command::{CommandFailure, Stdout, run_bounded};

/// Absolute path, so a different program earlier in `PATH` can never run.
const NETWORK_QUALITY: &str = "/usr/bin/networkQuality";
/// `-c` prints the result as JSON once the test has ended.
const ARGUMENTS: [&str; 1] = ["-c"];
/// A test takes about twenty seconds; a slow connection needs longer.
const TIMEOUT: Duration = Duration::from_secs(90);
const MILLIS_PER_SECOND: f64 = 1_000.0;

/// Measures the connection with `/usr/bin/networkQuality`, which moves data
/// to and from servers of Apple.
///
/// The program runs as the logged-in user, with no shell in between, a
/// cleared environment and no standard input. It runs only when a test is
/// asked for, and one test spends real data: more than a gigabyte on a fast
/// connection.
#[derive(Debug, Clone, Copy, Default)]
pub struct NetworkQualitySpeedTester;

impl SpeedTester for NetworkQualitySpeedTester {
    fn measure(&self) -> Result<SpeedTest, SpeedTestError> {
        let started = Instant::now();
        let mut command = Command::new(NETWORK_QUALITY);
        command.args(ARGUMENTS).env_clear();
        let output =
            run_bounded(&mut command, Stdout::Capture, TIMEOUT).map_err(
                |failure| match failure {
                    CommandFailure::NotStarted(error) => {
                        SpeedTestError::Unavailable(error.to_string())
                    }
                    CommandFailure::TimedOut => SpeedTestError::TimedOut,
                },
            )?;
        let duration = started.elapsed();
        if output.code != Some(0) {
            return Err(SpeedTestError::Failed(output.first_error_line()));
        }
        parse(&String::from_utf8_lossy(&output.stdout), duration).map_err(SpeedTestError::Failed)
    }
}

/// What `networkQuality -c` prints. It prints many more fields than these;
/// the others are ignored, and so is one of these that is not a number.
#[derive(Deserialize)]
struct Report {
    /// Bits per second.
    #[serde(default, deserialize_with = "figure")]
    dl_throughput: Option<f64>,
    /// Bits per second.
    #[serde(default, deserialize_with = "figure")]
    ul_throughput: Option<f64>,
    /// Milliseconds.
    #[serde(default, deserialize_with = "figure")]
    base_rtt: Option<f64>,
    /// Round trips per minute.
    #[serde(default, deserialize_with = "figure")]
    responsiveness: Option<f64>,
    #[serde(default, deserialize_with = "figure")]
    dl_bytes_transferred: Option<f64>,
    #[serde(default, deserialize_with = "figure")]
    ul_bytes_transferred: Option<f64>,
}

/// Reads a figure, leaving out one that is not a number.
fn figure<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<f64>, D::Error> {
    Ok(serde_json::Value::deserialize(deserializer)?.as_f64())
}

/// Reads the result of a test that took `duration`.
///
/// # Errors
///
/// Fails when the report is not JSON or lacks either speed: without them
/// there is no result to show.
fn parse(report: &str, duration: Duration) -> Result<SpeedTest, String> {
    let report: Report = serde_json::from_str(report).map_err(|error| error.to_string())?;
    let (Some(download), Some(upload)) = (report.dl_throughput, report.ul_throughput) else {
        return Err("the test ended without measuring the speed".to_owned());
    };
    Ok(SpeedTest {
        download: BitRate::from_bits_per_second(whole(download)),
        upload: BitRate::from_bits_per_second(whole(upload)),
        idle_latency: report
            .base_rtt
            .and_then(|millis| Duration::try_from_secs_f64(millis / MILLIS_PER_SECOND).ok()),
        responsiveness: report
            .responsiveness
            .and_then(|rpm| u32::try_from(whole(rpm)).ok())
            .map(Responsiveness::from_rpm),
        transferred: ByteSize::from_bytes(whole(report.dl_bytes_transferred.unwrap_or(0.0)))
            + ByteSize::from_bytes(whole(report.ul_bytes_transferred.unwrap_or(0.0))),
        duration,
    })
}

/// The nearest whole number. A negative figure, which no count can be,
/// becomes zero.
fn whole(figure: f64) -> u64 {
    figure.round() as u64
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use diskmarshal_domain::network::{BitRate, Responsiveness, SpeedTest};
    use diskmarshal_domain::shared::ByteSize;

    use super::parse;

    const TOOK: Duration = Duration::from_secs(21);

    /// A result in the form macOS 27 prints, with made-up figures and most
    /// of its long lists of samples cut short.
    const REPORT: &str = r#"{
  "base_rtt" : 12.5,
  "cli_options" : [
    "-c"
  ],
  "dl_bytes_transferred" : 700000000,
  "dl_flows" : 16,
  "dl_phase_duration" : 18.4,
  "dl_throughput" : 300250000,
  "draft_version" : 8,
  "end_date" : "1/1/2026, 12:00:21 PM",
  "il_h2_req_resp" : [
    14.5,
    13
  ],
  "il_tcp_handshake_443" : [
    12,
    13
  ],
  "interface_name" : "en0",
  "lud_self_h2_req_resp" : [
    130,
    142
  ],
  "os_version" : "Version 27.0 (Build 00A000)",
  "other" : {
    "ecn_values" : {
      "ecn_disabled" : 32
    },
    "protocols_seen" : {
      "h2" : 32
    },
    "proxy_state" : "not_proxied"
  },
  "responsiveness" : 812.4375,
  "start_date" : "1/1/2026, 12:00:00 PM",
  "test_endpoint" : "server.example.test",
  "ul_bytes_transferred" : 600000000,
  "ul_flows" : 16,
  "ul_throughput" : 150125000
}"#;

    #[test]
    fn reads_the_result_of_a_test() {
        assert_eq!(
            parse(REPORT, TOOK),
            Ok(SpeedTest {
                download: BitRate::from_bits_per_second(300_250_000),
                upload: BitRate::from_bits_per_second(150_125_000),
                idle_latency: Some(Duration::from_micros(12_500)),
                responsiveness: Some(Responsiveness::from_rpm(812)),
                transferred: ByteSize::from_bytes(1_300_000_000),
                duration: TOOK,
            })
        );
    }

    #[test]
    fn a_test_that_did_not_measure_responsiveness_still_has_a_result() {
        let report = r#"{"dl_throughput":8000000,"ul_throughput":1000000.4,"base_rtt":40}"#;
        let test = parse(report, TOOK).unwrap();

        assert_eq!(test.download, BitRate::from_bits_per_second(8_000_000));
        assert_eq!(test.upload, BitRate::from_bits_per_second(1_000_000));
        assert_eq!(test.idle_latency, Some(Duration::from_millis(40)));
        assert_eq!(test.responsiveness, None);
        assert_eq!(test.transferred, ByteSize::ZERO);
    }

    #[test]
    fn rounds_responsiveness_to_the_nearest_round_trip() {
        let rpm = |printed: &str| {
            let report =
                format!(r#"{{"dl_throughput":1,"ul_throughput":1,"responsiveness":{printed}}}"#);
            parse(&report, TOOK).unwrap().responsiveness
        };
        assert_eq!(rpm("199.5"), Some(Responsiveness::from_rpm(200)));
        assert_eq!(rpm("1250"), Some(Responsiveness::from_rpm(1_250)));
        assert_eq!(rpm("0.2"), Some(Responsiveness::from_rpm(0)));
        assert_eq!(rpm("null"), None);
        assert_eq!(rpm(r#""High""#), None);
    }

    #[test]
    fn a_figure_that_is_not_a_number_is_left_out() {
        let report = r#"{"dl_throughput":5,"ul_throughput":5,"base_rtt":"fast",
            "dl_bytes_transferred":null,"ul_bytes_transferred":900}"#;
        let test = parse(report, TOOK).unwrap();

        assert_eq!(test.idle_latency, None);
        assert_eq!(test.transferred, ByteSize::from_bytes(900));

        let negative = r#"{"dl_throughput":-5,"ul_throughput":5,"base_rtt":-1}"#;
        let test = parse(negative, TOOK).unwrap();
        assert!(test.download.is_zero());
        assert_eq!(test.idle_latency, None);
    }

    #[test]
    fn a_report_without_both_speeds_is_not_a_result() {
        for report in [
            "{}",
            r#"{"dl_throughput":300250000,"base_rtt":12.5}"#,
            r#"{"ul_throughput":150125000,"responsiveness":812}"#,
            r#"{"dl_throughput":"300250000","ul_throughput":150125000}"#,
            r#"{"error":"The Internet connection appears to be offline."}"#,
        ] {
            assert!(parse(report, TOOK).is_err(), "{report}");
        }
    }

    #[test]
    fn what_is_not_json_is_not_a_result() {
        assert!(parse("", TOOK).is_err());
        assert!(parse("==== SUMMARY ====\nUplink capacity: 365 Mbps\n", TOOK).is_err());
        assert!(parse("null", TOOK).is_err());
    }
}

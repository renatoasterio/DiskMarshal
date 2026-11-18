//! Reachability probe that asks `ping` and `dig`, programs macOS ships.

use std::net::IpAddr;
use std::process::Command;
use std::time::Duration;

use diskmarshal_application::ports::ReachabilityProbe;
use diskmarshal_domain::network::PingResult;

use crate::process::bounded_command::{Stdout, run_bounded};

/// Absolute paths, so a different program earlier in `PATH` can never run.
const PING: &str = "/sbin/ping";
/// The same program for IPv6 addresses.
const PING6: &str = "/sbin/ping6";
const DIG: &str = "/usr/bin/dig";

/// Ten pings, a fifth of a second apart. `-q` prints only the summary.
const PING_ARGUMENTS: [&str; 5] = ["-c", "10", "-i", "0.2", "-q"];
/// Ends `ping` after five seconds, however many answers are still missing.
/// `ping6` is not given a limit of its own.
const PING_DEADLINE: [&str; 2] = ["-t", "5"];
const PING_TIMEOUT: Duration = Duration::from_secs(15);
/// One try and two seconds to answer. `+noall +stats` prints only how the
/// question went, not the answer.
const DIG_OPTIONS: [&str; 4] = ["+tries=1", "+time=2", "+noall", "+stats"];
/// What every server is asked: the address of a name reserved for examples,
/// which any name server can answer.
const QUESTION: [&str; 2] = ["example.com", "A"];
const DIG_TIMEOUT: Duration = Duration::from_secs(5);
/// How many times a server is asked. The first answer may have to be
/// fetched from further away; the second one shows the server itself.
const QUESTIONS_PER_SERVER: usize = 2;

/// What the line that counts the pings says after each count.
const TRANSMITTED: &str = "packets transmitted";
const RECEIVED: &str = "packets received";
/// How the line with the answer times starts. It names its figures, as in
/// `round-trip min/avg/max/stddev = …`, before giving them.
const ROUND_TRIP_START: &str = "round-trip";
const AVERAGE: &str = "avg";
/// What comes before the answer time of a name server.
const QUERY_TIME: &str = "Query time:";
const MILLISECONDS: &str = "msec";
const MILLIS_PER_SECOND: f64 = 1_000.0;

/// Times answers with `/sbin/ping` and `/usr/bin/dig`.
///
/// Each program runs as the logged-in user, with no shell in between, a
/// cleared environment, no standard input and a time limit. They run only
/// when asked and send a few small packets: ten pings to an address, or two
/// questions to a name server.
#[derive(Debug, Clone, Copy, Default)]
pub struct PingReachabilityProbe;

impl ReachabilityProbe for PingReachabilityProbe {
    fn ping(&self, target: IpAddr) -> PingResult {
        run_bounded(&mut ping_command(target), Stdout::Capture, PING_TIMEOUT)
            // All pings lost is an error for `ping`, and still a result.
            .map(|output| parse_ping(&String::from_utf8_lossy(&output.stdout)))
            .unwrap_or_default()
    }

    fn time_dns(&self, server: IpAddr) -> Option<Duration> {
        (0..QUESTIONS_PER_SERVER)
            .filter_map(|_| {
                run_bounded(&mut dig_command(server), Stdout::Capture, DIG_TIMEOUT).ok()
            })
            .filter_map(|output| parse_query_time(&String::from_utf8_lossy(&output.stdout)))
            .min()
    }
}

fn ping_command(target: IpAddr) -> Command {
    let (program, deadline): (&str, &[&str]) = match target {
        IpAddr::V4(_) => (PING, &PING_DEADLINE),
        IpAddr::V6(_) => (PING6, &[]),
    };
    let mut command = Command::new(program);
    command
        .args(PING_ARGUMENTS)
        .args(deadline)
        .arg(target.to_string())
        .env_clear();
    command
}

fn dig_command(server: IpAddr) -> Command {
    let mut command = Command::new(DIG);
    command
        .args(DIG_OPTIONS)
        .arg(format!("@{server}"))
        .args(QUESTION)
        .env_clear();
    command
}

/// Reads the summary `ping` prints when it ends, such as
/// `10 packets transmitted, 9 packets received, 10.0% packet loss`.
///
/// A run that printed no summary sent nothing that can be counted.
fn parse_ping(report: &str) -> PingResult {
    let Some(counts) = report.lines().find(|line| line.contains(TRANSMITTED)) else {
        return PingResult::default();
    };
    let count = |what: &str| {
        counts
            .split(',')
            .find_map(|part| part.trim().strip_suffix(what)?.trim().parse::<u32>().ok())
    };
    let (Some(sent), Some(received)) = (count(TRANSMITTED), count(RECEIVED)) else {
        return PingResult::default();
    };
    let average = report
        .lines()
        .find(|line| line.starts_with(ROUND_TRIP_START))
        .and_then(average_of)
        .filter(|_| received > 0);
    PingResult {
        sent,
        received,
        average,
    }
}

/// Reads the average in a line like
/// `round-trip min/avg/max/stddev = 12.910/18.388/31.392/6.746 ms`.
fn average_of(line: &str) -> Option<Duration> {
    let (names, figures) = line.strip_prefix(ROUND_TRIP_START)?.split_once('=')?;
    let position = names.trim().split('/').position(|name| name == AVERAGE)?;
    let millis: f64 = figures
        .split_whitespace()
        .next()?
        .split('/')
        .nth(position)?
        .parse()
        .ok()?;
    Duration::try_from_secs_f64(millis / MILLIS_PER_SECOND).ok()
}

/// Reads how long a name server took, in a line like
/// `;; Query time: 14 msec`. A server that did not answer has no such line.
fn parse_query_time(report: &str) -> Option<Duration> {
    let (_, time) = report
        .lines()
        .find_map(|line| line.split_once(QUERY_TIME))?;
    let mut words = time.split_whitespace();
    let millis: u64 = words.next()?.parse().ok()?;
    (words.next()? == MILLISECONDS).then(|| Duration::from_millis(millis))
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::net::IpAddr;
    use std::process::Command;
    use std::time::Duration;

    use diskmarshal_domain::network::PingResult;

    use super::{dig_command, parse_ping, parse_query_time, ping_command};

    /// The summaries below have the form macOS 27 prints, with addresses
    /// from the ranges reserved for documentation.
    const SOME_LOST: &str = "\
PING 192.0.2.1 (192.0.2.1): 56 data bytes

--- 192.0.2.1 ping statistics ---
10 packets transmitted, 9 packets received, 10.0% packet loss
round-trip min/avg/max/stddev = 12.910/18.388/31.392/6.746 ms
";

    const ALL_LOST: &str = "\
PING 192.0.2.1 (192.0.2.1): 56 data bytes

--- 192.0.2.1 ping statistics ---
10 packets transmitted, 0 packets received, 100.0% packet loss
";

    const OVER_IPV6: &str = "\
PING6(56=40+8+8 bytes) 2001:db8::10 --> 2001:db8::1

--- 2001:db8::1 ping6 statistics ---
10 packets transmitted, 10 packets received, 0.0% packet loss
round-trip min/avg/max/std-dev = 1.250/2.375/4.500/1.125 ms
";

    const ANSWERED: &str = "\
;; Query time: 14 msec
;; SERVER: 192.0.2.53#53(192.0.2.53)
;; WHEN: Thu Jan 01 12:00:00 UTC 2026
;; MSG SIZE  rcvd: 72

";

    fn line(command: &Command) -> Vec<&str> {
        std::iter::once(command.get_program())
            .chain(command.get_args())
            .filter_map(OsStr::to_str)
            .collect()
    }

    #[test]
    fn reads_how_many_pings_were_answered_and_how_fast() {
        assert_eq!(
            parse_ping(SOME_LOST),
            PingResult {
                sent: 10,
                received: 9,
                average: Some(Duration::from_micros(18_388)),
            }
        );
    }

    #[test]
    fn pings_that_were_all_lost_are_still_a_result() {
        let result = parse_ping(ALL_LOST);
        assert_eq!(
            result,
            PingResult {
                sent: 10,
                received: 0,
                average: None,
            }
        );
        assert!(result.is_silent());
    }

    #[test]
    fn reads_the_summary_of_pings_over_ipv6() {
        assert_eq!(
            parse_ping(OVER_IPV6),
            PingResult {
                sent: 10,
                received: 10,
                average: Some(Duration::from_micros(2_375)),
            }
        );
    }

    #[test]
    fn answers_that_came_twice_do_not_hide_the_counts() {
        let report = "\
10 packets transmitted, 10 packets received, +2 duplicates, 0.0% packet loss
round-trip min/avg/max/stddev = 1.000/2.500/4.000/1.000 ms
";
        assert_eq!(
            parse_ping(report),
            PingResult {
                sent: 10,
                received: 10,
                average: Some(Duration::from_micros(2_500)),
            }
        );
    }

    #[test]
    fn a_run_without_a_summary_sent_nothing() {
        for report in [
            "",
            "PING 192.0.2.1 (192.0.2.1): 56 data bytes\n",
            "ping: cannot resolve nowhere.invalid: Unknown host\n",
            "some packets transmitted, none packets received\n",
            "10 packets transmitted\n",
        ] {
            assert_eq!(parse_ping(report), PingResult::default(), "{report:?}");
        }
    }

    #[test]
    fn an_answer_time_that_cannot_be_read_is_left_out() {
        let counts = "3 packets transmitted, 3 packets received, 0.0% packet loss\n";
        for times in [
            "",
            "round-trip min/max = 1.000/4.000 ms\n",
            "round-trip min/avg/max/stddev = 1.000\n",
            "round-trip min/avg/max/stddev = fast/faster/slow/none ms\n",
            "round-trip min/avg/max/stddev\n",
        ] {
            let result = parse_ping(&format!("{counts}{times}"));
            assert_eq!((result.sent, result.received), (3, 3), "{times:?}");
            assert_eq!(result.average, None, "{times:?}");
        }
    }

    #[test]
    fn reads_how_long_a_name_server_took_to_answer() {
        assert_eq!(parse_query_time(ANSWERED), Some(Duration::from_millis(14)));
        assert_eq!(
            parse_query_time(";; Query time: 0 msec\n"),
            Some(Duration::ZERO)
        );
    }

    #[test]
    fn a_name_server_that_did_not_answer_has_no_time() {
        for report in [
            "",
            ";; connection timed out; no servers could be reached\n",
            ";; Query time: soon\n",
            ";; Query time: 14\n",
            ";; Query time: 14 years\n",
        ] {
            assert_eq!(parse_query_time(report), None, "{report:?}");
        }
    }

    #[test]
    fn each_kind_of_address_is_pinged_by_its_own_program() {
        let old: IpAddr = "192.0.2.1".parse().unwrap();
        assert_eq!(
            line(&ping_command(old)),
            [
                "/sbin/ping",
                "-c",
                "10",
                "-i",
                "0.2",
                "-q",
                "-t",
                "5",
                "192.0.2.1"
            ]
        );
        let modern: IpAddr = "2001:db8::1".parse().unwrap();
        assert_eq!(
            line(&ping_command(modern)),
            ["/sbin/ping6", "-c", "10", "-i", "0.2", "-q", "2001:db8::1"]
        );
    }

    #[test]
    fn a_name_server_is_asked_for_a_name_reserved_for_examples() {
        let server: IpAddr = "192.0.2.53".parse().unwrap();
        assert_eq!(
            line(&dig_command(server)),
            [
                "/usr/bin/dig",
                "+tries=1",
                "+time=2",
                "+noall",
                "+stats",
                "@192.0.2.53",
                "example.com",
                "A"
            ]
        );
    }
}

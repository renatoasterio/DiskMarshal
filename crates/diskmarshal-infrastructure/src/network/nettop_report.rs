//! Reading what `nettop` prints in its CSV logging mode.
//!
//! The first line names the columns. After it comes one line per process,
//! labelled `name.pid`, each followed by one line per connection of that
//! process, labelled like `tcp4 192.0.2.10:50000<->203.0.113.24:443`.

use std::time::Duration;

use diskmarshal_domain::network::{Connection, Endpoint, Transport};
use diskmarshal_domain::shared::ByteSize;

const SEPARATOR: char = ',';
/// First cell of the line that names the columns.
const HEADER_START: &str = "time";
const COLUMN_INTERFACE: &str = "interface";
const COLUMN_RECEIVED: &str = "bytes_in";
const COLUMN_SENT: &str = "bytes_out";
const COLUMN_RETRANSMITTED: &str = "re-tx";
const COLUMN_ROUND_TRIP: &str = "rtt_avg";
/// What separates the two sides of a connection in its label.
const BETWEEN_SIDES: &str = "<->";
/// What stands for a part of an address that is not tied to anything.
const ANY: &str = "*";
const MILLIS_PER_SECOND: f64 = 1_000.0;

/// One process and its connections, as `nettop` lists them.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ProcessRow {
    /// Identifier of the process.
    pub pid: u32,
    /// Its name, which `nettop` cuts short.
    pub name: String,
    /// Its connections.
    pub connections: Vec<Connection>,
}

/// Where each figure is in a line.
struct Columns {
    label: usize,
    interface: Option<usize>,
    received: usize,
    sent: usize,
    retransmitted: Option<usize>,
    round_trip: Option<usize>,
}

impl Columns {
    fn of(header: &str) -> Result<Self, String> {
        let names: Vec<&str> = header.split(SEPARATOR).collect();
        let find = |wanted: &str| names.iter().position(|name| *name == wanted);
        let missing = |column: &str| format!("no {column} column");
        Ok(Self {
            // The column of the labels is the one without a name.
            label: find("").ok_or_else(|| missing("label"))?,
            interface: find(COLUMN_INTERFACE),
            received: find(COLUMN_RECEIVED).ok_or_else(|| missing(COLUMN_RECEIVED))?,
            sent: find(COLUMN_SENT).ok_or_else(|| missing(COLUMN_SENT))?,
            retransmitted: find(COLUMN_RETRANSMITTED),
            round_trip: find(COLUMN_ROUND_TRIP),
        })
    }
}

/// Reads one sample of `nettop -L`.
///
/// A line that cannot be understood is skipped: what the other lines say is
/// still true.
///
/// # Errors
///
/// Fails when the report does not start by naming the columns this app reads.
pub(crate) fn parse(report: &str) -> Result<Vec<ProcessRow>, String> {
    let mut lines = report.lines().filter(|line| !line.trim().is_empty());
    let header = lines
        .next()
        .filter(|line| line.starts_with(HEADER_START))
        .ok_or_else(|| "no header line".to_owned())?;
    let columns = Columns::of(header)?;

    let mut processes: Vec<ProcessRow> = Vec::new();
    // Connections belong to the process listed before them. After a process
    // line that could not be read, they belong to nobody.
    let mut is_inside_process = false;
    for line in lines {
        let cells: Vec<&str> = line.split(SEPARATOR).collect();
        let Some(label) = cells.get(columns.label).copied() else {
            continue;
        };
        if let Some((transport, sides)) = transport_of(label) {
            if let (true, Some(process), Some(connection)) = (
                is_inside_process,
                processes.last_mut(),
                connection(transport, sides, &cells, &columns),
            ) {
                process.connections.push(connection);
            }
        } else if let Some(process) = process(label) {
            processes.push(process);
            is_inside_process = true;
        } else {
            is_inside_process = false;
        }
    }
    Ok(processes)
}

/// Splits the label of a connection into how it carries data, whether its
/// addresses are IPv6, and its two sides.
fn transport_of(label: &str) -> Option<((Transport, bool), &str)> {
    let (kind, sides) = label.split_once(' ')?;
    let transport = match kind.get(..3)? {
        "tcp" => Transport::Tcp,
        "udp" => Transport::Udp,
        _ => return None,
    };
    Some(((transport, kind.ends_with('6')), sides))
}

fn process(label: &str) -> Option<ProcessRow> {
    let (name, pid) = label.rsplit_once('.')?;
    Some(ProcessRow {
        pid: pid.parse().ok()?,
        name: name.to_owned(),
        connections: Vec::new(),
    })
}

fn connection(
    (transport, is_ipv6): (Transport, bool),
    sides: &str,
    cells: &[&str],
    columns: &Columns,
) -> Option<Connection> {
    let (local, remote) = sides.split_once(BETWEEN_SIDES)?;
    let cell = |index: usize| cells.get(index).copied().unwrap_or_default();
    let bytes = |index: usize| ByteSize::from_bytes(cell(index).parse().unwrap_or(0));
    Some(Connection {
        transport,
        local: endpoint(local, is_ipv6),
        remote: endpoint(remote, is_ipv6),
        interface: columns.interface.map(cell).unwrap_or_default().to_owned(),
        received: bytes(columns.received),
        sent: bytes(columns.sent),
        retransmitted: columns.retransmitted.map(bytes).unwrap_or_default(),
        round_trip: columns.round_trip.and_then(|index| round_trip(cell(index))),
    })
}

/// Reads one side of a connection. `nettop` writes the port after a colon
/// for IPv4 and after a dot for IPv6, whose addresses have colons of their own.
fn endpoint(text: &str, is_ipv6: bool) -> Endpoint {
    let port_separator = if is_ipv6 { '.' } else { ':' };
    let (address, port) = text.rsplit_once(port_separator).unwrap_or((text, ANY));
    Endpoint {
        address: if address == ANY {
            String::new()
        } else {
            address.to_owned()
        },
        port: port.parse().ok(),
    }
}

/// Reads an answer time written as `14.19 ms`.
fn round_trip(text: &str) -> Option<Duration> {
    let millis: f64 = text.split_whitespace().next()?.parse().ok()?;
    Duration::try_from_secs_f64(millis / MILLIS_PER_SECOND).ok()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use diskmarshal_domain::network::Transport;
    use diskmarshal_domain::shared::ByteSize;

    use super::parse;

    /// One sample in the form macOS 27 prints, with made-up programs and
    /// addresses from the ranges reserved for documentation.
    const SAMPLE: &str = "\
time,,interface,state,bytes_in,bytes_out,rx_dupe,rx_ooo,re-tx,rtt_avg,rcvsize,tx_win,tc_class,tc_mgt,cc_algo,P,C,R,W,arch,
10:17:38.931554,notifier.591,,,2583484,222872,0,0,0,,,,,,,,,,,,
10:17:38.929222,tcp4 192.0.2.10:57689<->203.0.113.5:5223,en0,Established,2583484,222872,0,0,0,123.62 ms,131072,31872,RD,-,cubic,-,-,-,-,so,
10:17:38.931565,resolver.677,,,10975300,3452238,186,499,0,,,,,,,,,,,,
10:17:38.929252,tcp4 192.0.2.10:57807<->198.51.100.2:443,en0,Established,1006722,301084,186,499,212,14.19 ms,131072,2220032,BE,-,cubic,-,-,-,-,so,
10:17:38.926892,udp6 *.5353<->*.*,en0,,5188097,1441669,,,,,786896,,CTL,,,,,,,so,
10:17:38.926875,udp4 *:5353<->*:*,en0,,4780481,1709485,,,,,786896,,CTL,,,,,,,so,
10:17:38.931586,Web Helper (Ren.688,,,0,0,0,0,0,,,,,,,,,,,,
10:17:38.928074,tcp6 2001:db8::10.52000<->2001:db8:1::24.443,en0,Established,900,100,0,0,0,9.5 ms,,,,,,,,,,so,
";

    #[test]
    fn reads_each_process_with_its_connections() {
        let processes = parse(SAMPLE).unwrap();

        let summary: Vec<(u32, &str, usize)> = processes
            .iter()
            .map(|process| {
                (
                    process.pid,
                    process.name.as_str(),
                    process.connections.len(),
                )
            })
            .collect();
        assert_eq!(
            summary,
            [
                (591, "notifier", 1),
                (677, "resolver", 3),
                (688, "Web Helper (Ren", 1)
            ]
        );
    }

    #[test]
    fn reads_the_figures_of_a_connection() {
        let processes = parse(SAMPLE).unwrap();
        let secure = &processes[1].connections[0];

        assert_eq!(secure.transport, Transport::Tcp);
        assert_eq!(secure.local.to_string(), "192.0.2.10:57807");
        assert_eq!(secure.remote.to_string(), "198.51.100.2:443");
        assert_eq!(secure.interface, "en0");
        assert_eq!(secure.received, ByteSize::from_bytes(1_006_722));
        assert_eq!(secure.sent, ByteSize::from_bytes(301_084));
        assert_eq!(secure.retransmitted, ByteSize::from_bytes(212));
        assert_eq!(secure.round_trip, Some(Duration::from_micros(14_190)));
    }

    #[test]
    fn reads_open_sides_and_both_address_families() {
        let processes = parse(SAMPLE).unwrap();
        let multicast = &processes[1].connections[1];
        assert_eq!(multicast.transport, Transport::Udp);
        assert_eq!(multicast.local.to_string(), "*:5353");
        assert!(multicast.remote.is_unspecified());
        assert_eq!(multicast.round_trip, None);

        let modern = &processes[2].connections[0];
        assert_eq!(modern.local.to_string(), "[2001:db8::10]:52000");
        assert_eq!(modern.remote.to_string(), "[2001:db8:1::24]:443");
    }

    #[test]
    fn skips_lines_it_cannot_read_and_the_connections_under_them() {
        let report = "\
time,,interface,state,bytes_in,bytes_out
10:00:00,no pid here,,,1,1
10:00:00,tcp4 192.0.2.10:1<->203.0.113.5:2,en0,Established,5,5
short
10:00:00,good.12,,,7,7
10:00:00,tcp4 192.0.2.10:3<->203.0.113.5:4,en0,Established,7,7
10:00:00,tcp4 without sides,en0,Established,9,9
";
        let processes = parse(report).unwrap();

        assert_eq!(processes.len(), 1);
        assert_eq!(processes[0].pid, 12);
        assert_eq!(processes[0].connections.len(), 1);
        assert_eq!(processes[0].connections[0].retransmitted, ByteSize::ZERO);
    }

    #[test]
    fn refuses_a_report_that_does_not_name_its_columns() {
        assert!(parse("").is_err());
        assert!(parse("nettop: something went wrong\n").is_err());
        assert!(parse("time,,interface,state\n").is_err());
    }
}

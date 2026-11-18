//! Reading what `system_profiler SPAirPortDataType -json` says about the
//! wireless link.
//!
//! The report lists every wireless interface. Each one may carry the network
//! it is connected to and the other networks it can hear. Names of networks
//! are never read.

use diskmarshal_domain::network::{BitRate, WifiBand, WifiLink};
use serde_json::Value;

const SECTIONS: &str = "SPAirPortDataType";
const INTERFACES: &str = "spairport_airport_interfaces";
const INTERFACE_NAME: &str = "_name";
const CURRENT_NETWORK: &str = "spairport_current_network_information";
const OTHER_NETWORKS: &str = "spairport_airport_other_local_wireless_networks";
const CHANNEL: &str = "spairport_network_channel";
const STANDARD: &str = "spairport_network_phymode";
/// The speed of the link, in megabits per second.
const RATE: &str = "spairport_network_rate";
const SIGNAL_AND_NOISE: &str = "spairport_signal_noise";
/// What separates the signal from the noise.
const BETWEEN_SIGNAL_AND_NOISE: char = '/';
/// How the band and the width of a channel end.
const GIGAHERTZ: &str = "GHz";
const MEGAHERTZ: &str = "MHz";
const BITS_PER_MEGABIT: f64 = 1_000_000.0;

/// A channel as the report writes it, such as `44 (5GHz, 160MHz)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Channel {
    number: u32,
    band: Option<WifiBand>,
    width_mhz: Option<u32>,
}

impl Channel {
    fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        let number_end = text
            .find(|letter: char| !letter.is_ascii_digit())
            .unwrap_or(text.len());
        let (number, details) = text.split_at(number_end);
        let mut channel = Self {
            number: number.parse().ok()?,
            band: None,
            width_mhz: None,
        };
        let details = details.trim().trim_start_matches('(').trim_end_matches(')');
        for detail in details.split(',').map(str::trim) {
            if let Some(gigahertz) = detail.strip_suffix(GIGAHERTZ) {
                channel.band = band(gigahertz.trim());
            } else if let Some(megahertz) = detail.strip_suffix(MEGAHERTZ) {
                channel.width_mhz = megahertz.trim().parse().ok();
            }
        }
        Some(channel)
    }

    /// Whether a network on `other` shares the air with one on this channel.
    /// The same number names different channels in different bands.
    fn is_shared_with(self, other: Self) -> bool {
        self.number == other.number
            && match (self.band, other.band) {
                (Some(band), Some(other_band)) => band == other_band,
                _ => true,
            }
    }
}

/// Reads the report and returns the link of `interface`.
///
/// `None` when the interface is not a wireless one, is not connected to a
/// network, or the report leaves out any figure of the link.
pub(crate) fn wifi_link(report: &str, interface: &str) -> Option<WifiLink> {
    let report: Value = serde_json::from_str(report).ok()?;
    let interface = report
        .get(SECTIONS)?
        .as_array()?
        .iter()
        .filter_map(|section| section.get(INTERFACES)?.as_array())
        .flatten()
        .find(|candidate| {
            candidate.get(INTERFACE_NAME).and_then(Value::as_str) == Some(interface)
        })?;
    let current = interface.get(CURRENT_NETWORK)?;

    let channel = channel_of(current)?;
    let (signal_dbm, noise_dbm) = signal_and_noise(current.get(SIGNAL_AND_NOISE)?.as_str()?)?;
    let megabits = current
        .get(RATE)?
        .as_f64()
        .filter(|megabits| *megabits >= 0.0)?;
    let neighbors_on_channel = interface
        .get(OTHER_NETWORKS)
        .and_then(Value::as_array)
        .map_or(0, |others| {
            others
                .iter()
                .filter_map(channel_of)
                .filter(|other| channel.is_shared_with(*other))
                .count()
        });
    Some(WifiLink {
        signal_dbm,
        noise_dbm,
        channel: channel.number,
        band: channel.band,
        width_mhz: channel.width_mhz,
        standard: current.get(STANDARD)?.as_str()?.trim().to_owned(),
        transmit_rate: BitRate::from_bits_per_second((megabits * BITS_PER_MEGABIT).round() as u64),
        neighbors_on_channel,
    })
}

fn channel_of(network: &Value) -> Option<Channel> {
    Channel::parse(network.get(CHANNEL)?.as_str()?)
}

/// The band named by the figure before `GHz`. The report rounds 2.4 down.
fn band(gigahertz: &str) -> Option<WifiBand> {
    match gigahertz {
        "2" | "2.4" => Some(WifiBand::TwoPointFourGigahertz),
        "5" => Some(WifiBand::FiveGigahertz),
        "6" => Some(WifiBand::SixGigahertz),
        _ => None,
    }
}

/// Reads a pair written as `-52 dBm / -93 dBm`.
fn signal_and_noise(text: &str) -> Option<(i32, i32)> {
    let (signal, noise) = text.split_once(BETWEEN_SIGNAL_AND_NOISE)?;
    let dbm = |side: &str| side.split_whitespace().next()?.parse().ok();
    Some((dbm(signal)?, dbm(noise)?))
}

#[cfg(test)]
mod tests {
    use diskmarshal_domain::network::{BitRate, WifiBand, WifiLink};

    use super::{Channel, signal_and_noise, wifi_link};

    /// A report in the form macOS 27 prints, cut down to what is read plus a
    /// few of the fields around it, with made-up networks.
    const REPORT: &str = r#"{
  "SPAirPortDataType" : [
    {
      "spairport_airport_interfaces" : [
        {
          "_name" : "en0",
          "spairport_airport_other_local_wireless_networks" : [
            {
              "_name" : "Example Cafe",
              "spairport_network_channel" : "11 (2GHz, 20MHz)",
              "spairport_network_phymode" : "802.11g/n",
              "spairport_network_type" : "spairport_network_type_station",
              "spairport_security_mode" : "spairport_security_mode_none"
            },
            {
              "_name" : "Example Neighbor",
              "spairport_network_channel" : "149 (5GHz, 20MHz)",
              "spairport_network_phymode" : "802.11a/n/ac/ax",
              "spairport_network_type" : "spairport_network_type_station",
              "spairport_security_mode" : "spairport_security_mode_wpa2_personal",
              "spairport_signal_noise" : "-71 dBm / -93 dBm"
            },
            {
              "_name" : "Example Office",
              "spairport_network_channel" : "149 (5GHz, 80MHz)",
              "spairport_network_phymode" : "802.11a/n/ac/ax",
              "spairport_network_type" : "spairport_network_type_station",
              "spairport_security_mode" : "spairport_security_mode_wpa3_transition"
            },
            {
              "_name" : "Example Six",
              "spairport_network_channel" : "149 (6GHz, 20MHz)",
              "spairport_network_phymode" : "802.11ax"
            },
            {
              "_name" : "Example Hidden"
            }
          ],
          "spairport_caps_airdrop" : "spairport_caps_supported",
          "spairport_current_network_information" : {
            "_name" : "Example Home",
            "spairport_network_channel" : "149 (5GHz, 80MHz)",
            "spairport_network_country_code" : "ZZ",
            "spairport_network_mcs" : 10,
            "spairport_network_phymode" : "802.11ax",
            "spairport_network_rate" : 1201,
            "spairport_network_type" : "spairport_network_type_station",
            "spairport_security_mode" : "spairport_security_mode_wpa2_personal",
            "spairport_signal_noise" : "-52 dBm / -93 dBm"
          },
          "spairport_status_information" : "spairport_status_connected",
          "spairport_supported_channels" : [
            "1 (2GHz)",
            "36 (5GHz)"
          ],
          "spairport_supported_phymodes" : "802.11 a/b/g/n/ac/ax",
          "spairport_wireless_mac_address" : "02:00:00:00:00:01"
        },
        {
          "_name" : "awdl0",
          "spairport_current_network_information" : {
            "spairport_network_type" : "spairport_network_type_station"
          },
          "spairport_supported_channels" : [
            "1 (2GHz)"
          ],
          "spairport_wireless_mac_address" : "02:00:00:00:00:02"
        }
      ],
      "spairport_software_information" : {
        "spairport_corewlan_version" : "1.0 (100)"
      }
    }
  ]
}"#;

    /// A report with one interface, connected to the network given.
    fn report_of(current_network: &str) -> String {
        format!(
            r#"{{"SPAirPortDataType":[{{"spairport_airport_interfaces":[
            {{"_name":"en0","spairport_current_network_information":{current_network}}}]}}]}}"#
        )
    }

    #[test]
    fn reads_the_link_of_the_interface_in_use() {
        assert_eq!(
            wifi_link(REPORT, "en0"),
            Some(WifiLink {
                signal_dbm: -52,
                noise_dbm: -93,
                channel: 149,
                band: Some(WifiBand::FiveGigahertz),
                width_mhz: Some(80),
                standard: "802.11ax".to_owned(),
                transmit_rate: BitRate::from_bits_per_second(1_201_000_000),
                neighbors_on_channel: 2,
            })
        );
    }

    #[test]
    fn a_network_on_the_same_number_of_another_band_is_not_a_neighbor() {
        let five = Channel::parse("149 (5GHz, 80MHz)").unwrap();
        assert!(five.is_shared_with(Channel::parse("149 (5GHz, 20MHz)").unwrap()));
        assert!(five.is_shared_with(Channel::parse("149").unwrap()));
        assert!(!five.is_shared_with(Channel::parse("149 (6GHz, 20MHz)").unwrap()));
        assert!(!five.is_shared_with(Channel::parse("153 (5GHz, 80MHz)").unwrap()));
    }

    #[test]
    fn an_interface_that_is_not_connected_has_no_link() {
        assert_eq!(wifi_link(REPORT, "awdl0"), None);
        let idle = r#"{"SPAirPortDataType":[{"spairport_airport_interfaces":[{"_name":"en0"}]}]}"#;
        assert_eq!(wifi_link(idle, "en0"), None);
    }

    #[test]
    fn an_interface_the_report_does_not_list_has_no_link() {
        assert_eq!(wifi_link(REPORT, "en7"), None);
        assert_eq!(wifi_link(r#"{"SPAirPortDataType":[]}"#, "en0"), None);
        assert_eq!(wifi_link("{}", "en0"), None);
    }

    #[test]
    fn a_report_that_cannot_be_read_gives_no_link() {
        assert_eq!(wifi_link("", "en0"), None);
        assert_eq!(wifi_link("not json", "en0"), None);
        assert_eq!(wifi_link(r#"{"SPAirPortDataType":"none"}"#, "en0"), None);
    }

    #[test]
    fn a_link_with_a_figure_missing_is_not_reported() {
        let complete = r#"{"spairport_network_channel":"6 (2GHz, 20MHz)",
            "spairport_network_phymode":"802.11n","spairport_network_rate":144,
            "spairport_signal_noise":"-60 dBm / -90 dBm"}"#;
        let link = wifi_link(&report_of(complete), "en0").unwrap();
        assert_eq!(link.channel, 6);
        assert_eq!(link.band, Some(WifiBand::TwoPointFourGigahertz));
        assert_eq!(link.neighbors_on_channel, 0);

        for missing in [
            r#"{"spairport_network_phymode":"802.11n","spairport_network_rate":144,
                "spairport_signal_noise":"-60 dBm / -90 dBm"}"#,
            r#"{"spairport_network_channel":"6 (2GHz, 20MHz)","spairport_network_rate":144,
                "spairport_signal_noise":"-60 dBm / -90 dBm"}"#,
            r#"{"spairport_network_channel":"6 (2GHz, 20MHz)",
                "spairport_network_phymode":"802.11n",
                "spairport_signal_noise":"-60 dBm / -90 dBm"}"#,
            r#"{"spairport_network_channel":"6 (2GHz, 20MHz)",
                "spairport_network_phymode":"802.11n","spairport_network_rate":144}"#,
            r#"{"spairport_network_channel":"6 (2GHz, 20MHz)",
                "spairport_network_phymode":"802.11n","spairport_network_rate":"fast",
                "spairport_signal_noise":"-60 dBm / -90 dBm"}"#,
        ] {
            assert_eq!(wifi_link(&report_of(missing), "en0"), None, "{missing}");
        }
    }

    #[test]
    fn reads_channels_with_and_without_their_band_and_width() {
        let channel =
            |text| Channel::parse(text).map(|read| (read.number, read.band, read.width_mhz));
        assert_eq!(
            channel("44 (5GHz, 160MHz)"),
            Some((44, Some(WifiBand::FiveGigahertz), Some(160)))
        );
        assert_eq!(
            channel("11 (2GHz, 20MHz)"),
            Some((11, Some(WifiBand::TwoPointFourGigahertz), Some(20)))
        );
        assert_eq!(
            channel("37 (6GHz, 320MHz)"),
            Some((37, Some(WifiBand::SixGigahertz), Some(320)))
        );
        assert_eq!(
            channel("1 (2GHz)"),
            Some((1, Some(WifiBand::TwoPointFourGigahertz), None))
        );
        assert_eq!(channel("149"), Some((149, None, None)));
        assert_eq!(channel("36,+1"), Some((36, None, None)));
        assert_eq!(channel("7 (60GHz, wideMHz)"), Some((7, None, None)));
        assert_eq!(channel(""), None);
        assert_eq!(channel("auto"), None);
    }

    #[test]
    fn reads_the_signal_and_the_noise() {
        assert_eq!(signal_and_noise("-52 dBm / -93 dBm"), Some((-52, -93)));
        assert_eq!(signal_and_noise("-52 dBm/-93 dBm"), Some((-52, -93)));
        assert_eq!(signal_and_noise("-52 dBm"), None);
        assert_eq!(signal_and_noise("strong / quiet"), None);
        assert_eq!(signal_and_noise(""), None);
    }
}

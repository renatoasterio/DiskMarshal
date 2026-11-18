//! The wireless link between this Mac and its access point.

use crate::network::BitRate;

/// From this signal up the link is strong, in dBm.
const STRONG_FROM_DBM: i32 = -60;
/// From this signal up the link still holds, in dBm.
const FAIR_FROM_DBM: i32 = -70;
/// The ends of the scale a signal is drawn on, in dBm.
const SIGNAL_SCALE_DBM: (i32, i32) = (-90, -30);
/// The ends of the scale noise is drawn on, in dBm.
const NOISE_SCALE_DBM: (i32, i32) = (-100, -60);
/// The Wi-Fi generation each standard is sold as, newest first.
const GENERATIONS: [(&str, &str); 4] = [
    ("be", "Wi-Fi 7"),
    ("ax", "Wi-Fi 6"),
    ("ac", "Wi-Fi 5"),
    ("n", "Wi-Fi 4"),
];
/// How every standard name starts.
const STANDARD_PREFIX: &str = "802.11";

/// The part of the radio spectrum a wireless network uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiBand {
    /// Reaches far, carries little and is crowded.
    TwoPointFourGigahertz,
    /// Carries much more, over a shorter distance.
    FiveGigahertz,
    /// The newest and least crowded.
    SixGigahertz,
}

impl WifiBand {
    /// The band as people name it.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::TwoPointFourGigahertz => "2,4 GHz",
            Self::FiveGigahertz => "5 GHz",
            Self::SixGigahertz => "6 GHz",
        }
    }
}

/// How good the signal is, in the three steps the interface tells apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalQuality {
    /// Nothing to gain by moving closer.
    Strong,
    /// Works, with less speed than the network can give.
    Fair,
    /// Slow and unstable.
    Weak,
}

/// The wireless link as macOS reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WifiLink {
    /// Strength of the signal, in dBm: closer to zero is stronger.
    pub signal_dbm: i32,
    /// Background noise, in dBm: further from zero is quieter.
    pub noise_dbm: i32,
    /// Channel in use.
    pub channel: u32,
    /// Band of that channel, when macOS says it.
    pub band: Option<WifiBand>,
    /// Width of the channel in megahertz, when macOS says it.
    pub width_mhz: Option<u32>,
    /// Standard in use, such as `802.11ax`.
    pub standard: String,
    /// Speed the link is transmitting at.
    pub transmit_rate: BitRate,
    /// Other networks in range that use the same channel.
    pub neighbors_on_channel: usize,
}

impl WifiLink {
    /// How good the signal is.
    #[must_use]
    pub const fn quality(&self) -> SignalQuality {
        if self.signal_dbm >= STRONG_FROM_DBM {
            SignalQuality::Strong
        } else if self.signal_dbm >= FAIR_FROM_DBM {
            SignalQuality::Fair
        } else {
            SignalQuality::Weak
        }
    }

    /// Where the signal sits on the drawing scale, from 0.0 to 1.0.
    #[must_use]
    pub fn signal_ratio(&self) -> f32 {
        ratio_on(SIGNAL_SCALE_DBM, self.signal_dbm)
    }

    /// Where the noise sits on the drawing scale, from 0.0 to 1.0.
    #[must_use]
    pub fn noise_ratio(&self) -> f32 {
        ratio_on(NOISE_SCALE_DBM, self.noise_dbm)
    }

    /// The generation the standard is sold as, such as `Wi-Fi 6`.
    #[must_use]
    pub fn generation(&self) -> Option<&'static str> {
        let letters = self.standard.strip_prefix(STANDARD_PREFIX)?;
        // A link lists every standard it speaks (`a/n/ac/ax`): the newest counts.
        GENERATIONS
            .iter()
            .find(|(suffix, _)| letters.split('/').any(|spoken| spoken == *suffix))
            .map(|(_, generation)| *generation)
    }
}

fn ratio_on((low, high): (i32, i32), value: i32) -> f32 {
    ((value - low) as f32 / (high - low) as f32).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::{SignalQuality, WifiBand, WifiLink};
    use crate::network::BitRate;

    fn link(signal_dbm: i32, standard: &str) -> WifiLink {
        WifiLink {
            signal_dbm,
            noise_dbm: -93,
            channel: 44,
            band: Some(WifiBand::FiveGigahertz),
            width_mhz: Some(80),
            standard: standard.to_owned(),
            transmit_rate: BitRate::from_bits_per_second(1_201_000_000),
            neighbors_on_channel: 0,
        }
    }

    #[test]
    fn classifies_the_signal_at_the_thresholds() {
        assert_eq!(link(-52, "802.11ax").quality(), SignalQuality::Strong);
        assert_eq!(link(-60, "802.11ax").quality(), SignalQuality::Strong);
        assert_eq!(link(-61, "802.11ax").quality(), SignalQuality::Fair);
        assert_eq!(link(-70, "802.11ax").quality(), SignalQuality::Fair);
        assert_eq!(link(-71, "802.11ax").quality(), SignalQuality::Weak);
    }

    #[test]
    fn the_drawing_scales_never_leave_their_ends() {
        assert!((link(-60, "").signal_ratio() - 0.5).abs() < f32::EPSILON);
        assert!((link(-10, "").signal_ratio() - 1.0).abs() < f32::EPSILON);
        assert!(link(-120, "").signal_ratio().abs() < f32::EPSILON);
        assert!((link(-60, "").noise_ratio() - 0.175).abs() < 1e-6);
    }

    #[test]
    fn names_the_generation_of_the_newest_standard_spoken() {
        assert_eq!(link(-50, "802.11ax").generation(), Some("Wi-Fi 6"));
        assert_eq!(link(-50, "802.11a/n/ac").generation(), Some("Wi-Fi 5"));
        assert_eq!(link(-50, "802.11be").generation(), Some("Wi-Fi 7"));
        assert_eq!(link(-50, "802.11g").generation(), None);
        assert_eq!(link(-50, "").generation(), None);
        assert_eq!(WifiBand::TwoPointFourGigahertz.label(), "2,4 GHz");
    }
}

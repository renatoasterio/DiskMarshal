//! Speed of a transfer, in bits per second.

use std::fmt;
use std::iter::Sum;
use std::ops::Add;
use std::time::Duration;

use crate::shared::{ByteSize, scale_by_thousands};

/// How fast data moves, in bits per second: the unit internet plans are
/// sold in, so a reading can be compared with the plan.
///
/// Arithmetic saturates instead of overflowing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BitRate(u64);

impl BitRate {
    /// Nothing moving.
    pub const ZERO: Self = Self(0);

    const UNITS: [&'static str; 4] = ["b/s", "kb/s", "Mb/s", "Gb/s"];
    const BITS_PER_BYTE: u128 = 8;
    const MICROS_PER_SECOND: u128 = 1_000_000;
    const BITS_PER_MEGABIT: f64 = 1_000_000.0;

    /// Creates a rate from a raw count of bits per second.
    #[must_use]
    pub const fn from_bits_per_second(bits: u64) -> Self {
        Self(bits)
    }

    /// Returns the raw count of bits per second.
    #[must_use]
    pub const fn as_bits_per_second(self) -> u64 {
        self.0
    }

    /// The rate at which `amount` moved during `elapsed`. No time at all
    /// reports no speed, since nothing can be said about it.
    #[must_use]
    pub fn of(amount: ByteSize, elapsed: Duration) -> Self {
        let micros = elapsed.as_micros();
        if micros == 0 {
            return Self::ZERO;
        }
        let bits = u128::from(amount.as_bytes())
            .saturating_mul(Self::BITS_PER_BYTE)
            .saturating_mul(Self::MICROS_PER_SECOND)
            / micros;
        Self(u64::try_from(bits).unwrap_or(u64::MAX))
    }

    /// The rate in megabits per second, the figure speed tests show.
    #[must_use]
    pub fn megabits_per_second(self) -> f64 {
        self.0 as f64 / Self::BITS_PER_MEGABIT
    }

    /// Returns `true` when nothing is moving.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl Add for BitRate {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }
}

impl Sum for BitRate {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, Add::add)
    }
}

/// Formats the rate for people, with one decimal place and the pt-BR decimal
/// comma: `12,4 Mb/s`.
impl fmt::Display for BitRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&scale_by_thousands(self.0, &Self::UNITS))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::BitRate;
    use crate::shared::ByteSize;

    #[test]
    fn a_megabyte_in_one_second_is_eight_megabits_per_second() {
        let rate = BitRate::of(ByteSize::from_bytes(1_000_000), Duration::from_secs(1));
        assert_eq!(rate, BitRate::from_bits_per_second(8_000_000));
        assert!((rate.megabits_per_second() - 8.0).abs() < f64::EPSILON);
    }

    #[test]
    fn the_same_amount_in_less_time_is_faster() {
        let amount = ByteSize::from_bytes(125_000);
        assert_eq!(
            BitRate::of(amount, Duration::from_millis(500)),
            BitRate::from_bits_per_second(2_000_000)
        );
    }

    #[test]
    fn no_elapsed_time_reports_no_speed() {
        assert!(BitRate::of(ByteSize::from_bytes(900), Duration::ZERO).is_zero());
    }

    #[test]
    fn formats_in_the_units_of_an_internet_plan() {
        assert_eq!(BitRate::from_bits_per_second(640).to_string(), "640 b/s");
        assert_eq!(
            BitRate::from_bits_per_second(840_000).to_string(),
            "840,0 kb/s"
        );
        assert_eq!(
            BitRate::from_bits_per_second(468_415_520).to_string(),
            "468,4 Mb/s"
        );
        assert_eq!(
            BitRate::from_bits_per_second(2_500_000_000).to_string(),
            "2,5 Gb/s"
        );
    }

    #[test]
    fn sums_and_saturates() {
        let total: BitRate = [1, 2, 3]
            .into_iter()
            .map(BitRate::from_bits_per_second)
            .sum();
        assert_eq!(total, BitRate::from_bits_per_second(6));
        let huge = BitRate::from_bits_per_second(u64::MAX);
        assert_eq!(huge + BitRate::from_bits_per_second(1), huge);
        assert_eq!(
            BitRate::of(ByteSize::from_bytes(u64::MAX), Duration::from_micros(1)),
            huge
        );
    }
}

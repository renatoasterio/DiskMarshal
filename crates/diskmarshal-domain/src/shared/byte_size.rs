//! Amount of data, in bytes.

use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign};

use crate::shared::scale_by_thousands;

/// A non-negative amount of bytes.
///
/// Arithmetic saturates instead of overflowing, so summing sizes reported by the
/// file system can never panic or wrap around.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteSize(u64);

impl ByteSize {
    /// No bytes at all.
    pub const ZERO: Self = Self(0);

    /// Decimal units, the same ones Finder shows (1 GB = 1 000 000 000 bytes).
    const UNITS: [&'static str; 5] = ["B", "KB", "MB", "GB", "TB"];

    /// Creates a size from a raw byte count.
    #[must_use]
    pub const fn from_bytes(bytes: u64) -> Self {
        Self(bytes)
    }

    /// Returns the raw byte count.
    #[must_use]
    pub const fn as_bytes(self) -> u64 {
        self.0
    }

    /// Subtracts `other`, stopping at zero instead of underflowing.
    #[must_use]
    pub const fn saturating_sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }

    /// Returns `true` when the size is zero bytes.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl Add for ByteSize {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }
}

impl AddAssign for ByteSize {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Sum for ByteSize {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, Add::add)
    }
}

/// Formats the size for people, with one decimal place and the pt-BR decimal
/// comma: `12,4 GB`. Plain bytes are shown without decimals: `512 B`.
impl fmt::Display for ByteSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&scale_by_thousands(self.0, &Self::UNITS))
    }
}

#[cfg(test)]
mod tests {
    use super::ByteSize;

    #[test]
    fn formats_plain_bytes_without_decimals() {
        assert_eq!(ByteSize::from_bytes(0).to_string(), "0 B");
        assert_eq!(ByteSize::from_bytes(999).to_string(), "999 B");
    }

    #[test]
    fn formats_decimal_units_with_comma() {
        assert_eq!(ByteSize::from_bytes(1_000).to_string(), "1,0 KB");
        assert_eq!(ByteSize::from_bytes(12_400_000_000).to_string(), "12,4 GB");
        assert_eq!(
            ByteSize::from_bytes(460_000_000_000).to_string(),
            "460,0 GB"
        );
    }

    #[test]
    fn rounds_up_into_the_next_unit() {
        assert_eq!(ByteSize::from_bytes(999_950_000).to_string(), "1,0 GB");
    }

    #[test]
    fn keeps_terabytes_as_the_largest_unit() {
        assert_eq!(
            ByteSize::from_bytes(5_000_000_000_000_000).to_string(),
            "5000,0 TB"
        );
    }

    #[test]
    fn addition_saturates_instead_of_overflowing() {
        let huge = ByteSize::from_bytes(u64::MAX);
        assert_eq!(huge + ByteSize::from_bytes(1), huge);
    }

    #[test]
    fn sums_an_iterator_of_sizes() {
        let total: ByteSize = [1, 2, 3].into_iter().map(ByteSize::from_bytes).sum();
        assert_eq!(total, ByteSize::from_bytes(6));
    }
}

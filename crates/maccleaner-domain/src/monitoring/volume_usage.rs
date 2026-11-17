//! How full a disk volume is.

use crate::shared::ByteSize;

/// Capacity and free space of one volume.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VolumeUsage {
    /// Capacity of the volume.
    pub total: ByteSize,
    /// Space still available to the user.
    pub available: ByteSize,
}

impl VolumeUsage {
    /// Below this share of free space the volume is considered critical.
    const CRITICAL_FREE_RATIO: f64 = 0.10;

    /// Space in use.
    #[must_use]
    pub fn used(&self) -> ByteSize {
        self.total.saturating_sub(self.available)
    }

    /// Share of the volume in use, from 0.0 to 1.0. An empty volume reports 0.0.
    #[must_use]
    pub fn used_ratio(&self) -> f64 {
        if self.total.is_zero() {
            return 0.0;
        }
        self.used().as_bytes() as f64 / self.total.as_bytes() as f64
    }

    /// Returns `true` when less than 10% of the volume is free.
    #[must_use]
    pub fn is_critical(&self) -> bool {
        !self.total.is_zero() && 1.0 - self.used_ratio() < Self::CRITICAL_FREE_RATIO
    }
}

#[cfg(test)]
mod tests {
    use super::VolumeUsage;
    use crate::shared::ByteSize;

    fn usage(total: u64, available: u64) -> VolumeUsage {
        VolumeUsage {
            total: ByteSize::from_bytes(total),
            available: ByteSize::from_bytes(available),
        }
    }

    #[test]
    fn reports_used_space_and_its_share() {
        let volume = usage(1_000, 250);
        assert_eq!(volume.used(), ByteSize::from_bytes(750));
        assert!((volume.used_ratio() - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn is_critical_below_ten_percent_free() {
        assert!(usage(460, 36).is_critical());
        assert!(!usage(460, 100).is_critical());
    }

    #[test]
    fn an_empty_or_inconsistent_volume_never_divides_by_zero_or_underflows() {
        assert!(usage(0, 0).used_ratio().abs() < f64::EPSILON);
        assert!(!usage(0, 0).is_critical());
        assert_eq!(usage(10, 50).used(), ByteSize::ZERO);
    }
}

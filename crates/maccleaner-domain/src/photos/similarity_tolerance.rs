//! How different two images may be and still count as the same photo.

/// How strict the comparison is.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SimilarityTolerance {
    /// The same picture saved again: another size, format or compression.
    Strict,
    /// Also pictures with small differences, such as shots taken in a burst.
    #[default]
    Balanced,
    /// Also pictures that only resemble each other. Needs a careful review.
    Loose,
}

impl SimilarityTolerance {
    /// Every tolerance, strictest first.
    pub const ALL: [Self; 3] = [Self::Strict, Self::Balanced, Self::Loose];

    /// Most bits two outline hashes may differ by, out of 64.
    #[must_use]
    pub const fn max_hash_distance(self) -> u32 {
        match self {
            Self::Strict => 2,
            Self::Balanced => 6,
            Self::Loose => 10,
        }
    }

    /// Largest average difference in brightness between the two images, on
    /// the 0 to 255 scale.
    #[must_use]
    pub const fn max_mean_difference(self) -> u32 {
        match self {
            Self::Strict => 3,
            Self::Balanced => 8,
            Self::Loose => 14,
        }
    }
}

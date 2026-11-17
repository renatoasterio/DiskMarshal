//! How much the interface is allowed to move.

use crate::i18n::pt_br;

/// The user's choice of how much animation to show.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum MotionPreference {
    /// Every animation, including the ones that loop in the background.
    #[default]
    Full,
    /// Short transitions only. Nothing loops.
    Reduced,
    /// No animation at all: every change is immediate.
    Off,
}

impl MotionPreference {
    /// Every option, in the order they are offered.
    pub const ALL: [Self; 3] = [Self::Full, Self::Reduced, Self::Off];

    /// Name shown to the user.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Full => pt_br::MOTION_FULL,
            Self::Reduced => pt_br::MOTION_REDUCED,
            Self::Off => pt_br::MOTION_OFF,
        }
    }

    /// Returns `true` when animations that loop forever may run.
    #[must_use]
    pub const fn allows_ambient(self) -> bool {
        matches!(self, Self::Full)
    }

    /// Returns `true` when short, one-off transitions may run.
    #[must_use]
    pub const fn allows_transitions(self) -> bool {
        !matches!(self, Self::Off)
    }
}

#[cfg(test)]
mod tests {
    use super::MotionPreference;

    #[test]
    fn only_full_motion_allows_looping_animations() {
        assert!(MotionPreference::Full.allows_ambient());
        assert!(!MotionPreference::Reduced.allows_ambient());
        assert!(!MotionPreference::Off.allows_ambient());
    }

    #[test]
    fn switching_motion_off_stops_transitions_too() {
        assert!(MotionPreference::Reduced.allows_transitions());
        assert!(!MotionPreference::Off.allows_transitions());
    }
}

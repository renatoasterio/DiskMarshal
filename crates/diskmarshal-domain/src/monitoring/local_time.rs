//! Time of day on the user's clock.

use std::fmt;

/// A time of day in the user's time zone.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct LocalTime {
    /// Hour, from 0 to 23.
    pub hour: u8,
    /// Minute, from 0 to 59.
    pub minute: u8,
    /// Second, from 0 to 59.
    pub second: u8,
}

/// Formats as `HH:MM:SS`.
impl fmt::Display for LocalTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
    }
}

#[cfg(test)]
mod tests {
    use super::LocalTime;

    #[test]
    fn pads_every_field_to_two_digits() {
        let time = LocalTime {
            hour: 9,
            minute: 5,
            second: 0,
        };
        assert_eq!(time.to_string(), "09:05:00");
    }
}

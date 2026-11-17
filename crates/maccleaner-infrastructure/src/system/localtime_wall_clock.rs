//! Wall clock read with `localtime_r`.

use std::mem::MaybeUninit;
use std::ptr;

use maccleaner_application::ports::WallClock;
use maccleaner_domain::monitoring::LocalTime;

/// Reads the time of day in the user's time zone with `localtime_r`.
#[derive(Debug, Clone, Copy, Default)]
pub struct LocaltimeWallClock;

impl WallClock for LocaltimeWallClock {
    fn now(&self) -> LocalTime {
        // SAFETY: `time` accepts a null pointer and then only returns the value.
        let seconds = unsafe { libc::time(ptr::null_mut()) };
        let mut parts = MaybeUninit::<libc::tm>::uninit();
        // SAFETY: both pointers are valid for the call; `localtime_r` writes the
        // result into `parts` and is the thread-safe variant of `localtime`.
        let result = unsafe { libc::localtime_r(&raw const seconds, parts.as_mut_ptr()) };
        if result.is_null() {
            return LocalTime::default();
        }
        // SAFETY: `localtime_r` returned non-null, so it initialized `parts`.
        let parts = unsafe { parts.assume_init() };
        LocalTime {
            hour: u8::try_from(parts.tm_hour).unwrap_or_default(),
            minute: u8::try_from(parts.tm_min).unwrap_or_default(),
            second: u8::try_from(parts.tm_sec).unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use maccleaner_application::ports::WallClock;

    use super::LocaltimeWallClock;

    #[test]
    fn returns_a_valid_time_of_day() {
        let time = LocaltimeWallClock.now();
        assert!(time.hour < 24 && time.minute < 60 && time.second < 61);
    }
}

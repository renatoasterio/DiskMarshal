//! A one-off animation measured against the frame clock.

/// Tracks how far a one-off animation has gone since it started.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Timeline {
    started_at: f64,
}

impl Timeline {
    /// Starts a timeline at `now`, in seconds on the frame clock.
    pub(crate) const fn start(now: f64) -> Self {
        Self { started_at: now }
    }

    /// Seconds elapsed since the start. Never negative.
    pub(crate) fn elapsed(&self, now: f64) -> f64 {
        (now - self.started_at).max(0.0)
    }

    /// Share of `duration_seconds` already elapsed, from 0.0 to 1.0. A zero or
    /// negative duration is already finished.
    pub(crate) fn progress(&self, now: f64, duration_seconds: f64) -> f32 {
        if duration_seconds <= 0.0 {
            return 1.0;
        }
        (self.elapsed(now) / duration_seconds).clamp(0.0, 1.0) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::Timeline;

    #[test]
    fn progress_runs_from_zero_to_one_and_stops_there() {
        let timeline = Timeline::start(10.0);
        assert!(timeline.progress(10.0, 2.0).abs() < f32::EPSILON);
        assert!((timeline.progress(11.0, 2.0) - 0.5).abs() < f32::EPSILON);
        assert!((timeline.progress(99.0, 2.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn a_clock_that_goes_backwards_never_gives_negative_progress() {
        let timeline = Timeline::start(10.0);
        assert!(timeline.progress(4.0, 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn a_zero_duration_is_already_finished() {
        assert!((Timeline::start(0.0).progress(0.0, 0.0) - 1.0).abs() < f32::EPSILON);
    }
}

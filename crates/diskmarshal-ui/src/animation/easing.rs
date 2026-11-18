//! Easing curves.

/// Starts fast and settles gently. `progress` is clamped to 0.0..=1.0.
pub(crate) fn ease_out_cubic(progress: f32) -> f32 {
    let remaining = 1.0 - progress.clamp(0.0, 1.0);
    1.0 - remaining * remaining * remaining
}

#[cfg(test)]
mod tests {
    use super::ease_out_cubic;

    #[test]
    fn starts_at_zero_and_ends_at_one() {
        assert!(ease_out_cubic(0.0).abs() < f32::EPSILON);
        assert!((ease_out_cubic(1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn moves_faster_at_the_start_than_at_the_end() {
        assert!(ease_out_cubic(0.5) > 0.5);
    }

    #[test]
    fn clamps_progress_outside_the_range() {
        assert!(ease_out_cubic(-3.0).abs() < f32::EPSILON);
        assert!((ease_out_cubic(7.0) - 1.0).abs() < f32::EPSILON);
    }
}

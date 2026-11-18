//! State of the live readings, shared by the monitor and the dashboard.

use std::collections::VecDeque;
use std::time::Duration;

use diskmarshal_domain::monitoring::SystemSnapshot;
use egui::Context;

use crate::app::UiServices;
use crate::tasks::Feed;

/// Time between readings.
const SAMPLE_INTERVAL: Duration = Duration::from_secs(1);
/// Readings taken, and the time between them, when the screen is filled in
/// before the first frame.
const PRELOAD_READINGS: usize = 24;
const PRELOAD_INTERVAL: Duration = Duration::from_millis(120);
/// Readings kept for the processor trace: two minutes.
const HISTORY_LENGTH: usize = 120;

/// The latest reading and the recent history of processor use.
#[derive(Default)]
pub(crate) struct MonitorViewModel {
    feed: Option<Feed<SystemSnapshot>>,
    latest: Option<SystemSnapshot>,
    cpu_history: VecDeque<f32>,
}

impl MonitorViewModel {
    /// Starts sampling in the background the first time it is needed.
    pub(crate) fn ensure_started(&mut self, ctx: &Context, services: &UiServices) {
        if self.feed.is_none() {
            let metrics = services.metrics.clone();
            self.feed = Some(Feed::spawn(ctx, SAMPLE_INTERVAL, move || metrics.sample()));
        }
    }

    /// Takes a short burst of readings right now. Used to open the app with
    /// figures and a trace already on screen.
    pub(crate) fn sample_now(&mut self, services: &UiServices) {
        // Processor use compares two readings, so the first one alone says nothing.
        services.metrics.sample();
        for _ in 0..PRELOAD_READINGS {
            std::thread::sleep(PRELOAD_INTERVAL);
            self.record(services.metrics.sample());
        }
    }

    /// Picks up the readings that arrived since the last frame.
    pub(crate) fn poll(&mut self) {
        let arrived = self.feed.as_ref().map(Feed::drain).unwrap_or_default();
        for snapshot in arrived {
            self.record(snapshot);
        }
    }

    fn record(&mut self, snapshot: SystemSnapshot) {
        if self.cpu_history.len() == HISTORY_LENGTH {
            self.cpu_history.pop_front();
        }
        self.cpu_history.push_back(snapshot.cpu_usage);
        self.latest = Some(snapshot);
    }

    /// The latest reading, when the system could be queried.
    pub(crate) fn latest(&self) -> Option<&SystemSnapshot> {
        self.latest
            .as_ref()
            .filter(|snapshot| !snapshot.memory_total.is_zero())
    }

    /// Processor use over time, oldest first.
    pub(crate) fn cpu_history(&self) -> Vec<f32> {
        self.cpu_history.iter().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use diskmarshal_domain::monitoring::SystemSnapshot;
    use diskmarshal_domain::shared::ByteSize;

    use super::{HISTORY_LENGTH, MonitorViewModel};

    fn snapshot(cpu_usage: f32) -> SystemSnapshot {
        SystemSnapshot {
            cpu_usage,
            memory_total: ByteSize::from_bytes(1_000),
            ..SystemSnapshot::default()
        }
    }

    #[test]
    fn keeps_only_the_most_recent_readings() {
        let mut model = MonitorViewModel::default();
        for index in 0..(HISTORY_LENGTH + 30) {
            model.record(snapshot(index as f32));
        }
        let history = model.cpu_history();
        assert_eq!(history.len(), HISTORY_LENGTH);
        assert!((history[0] - 30.0).abs() < f32::EPSILON);
        assert!((model.latest().unwrap().cpu_usage - (HISTORY_LENGTH + 29) as f32).abs() < 0.01);
    }

    #[test]
    fn an_empty_reading_counts_as_no_reading() {
        let mut model = MonitorViewModel::default();
        model.record(SystemSnapshot::default());
        assert!(model.latest().is_none());
    }
}

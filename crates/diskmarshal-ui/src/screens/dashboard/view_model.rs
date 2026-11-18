//! State of the dashboard: the recent removals it lists.

use diskmarshal_domain::deletion::HistoryEntry;
use diskmarshal_domain::shared::ByteSize;

use crate::app::UiServices;
use crate::navigation::Screen;

/// How often the list of recent removals is read again, in seconds.
const HISTORY_REFRESH_SECONDS: f64 = 5.0;
/// How many recent removals are listed.
const HISTORY_ROWS: usize = 7;

/// What the user asked for on the dashboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DashboardAction {
    /// Open another screen.
    GoTo(Screen),
    /// Analyze the cleanup rules now.
    AnalyzeCleanup,
}

/// What the other screens found so far. `None` means not analyzed yet.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Recoverable {
    pub cleanup: Option<ByteSize>,
    pub developer: Option<ByteSize>,
    pub docker: Option<ByteSize>,
    pub duplicates: Option<ByteSize>,
    pub photos: Option<ByteSize>,
    /// Whether the cleanup analysis is running right now.
    pub is_analyzing: bool,
}

impl Recoverable {
    /// The sum of what has been found. `None` when nothing was analyzed.
    pub(crate) fn total(&self) -> Option<ByteSize> {
        let found: Vec<ByteSize> = [
            self.cleanup,
            self.developer,
            self.docker,
            self.duplicates,
            self.photos,
        ]
        .into_iter()
        .flatten()
        .collect();
        (!found.is_empty()).then(|| found.into_iter().sum())
    }
}

/// Everything the dashboard remembers between frames.
#[derive(Default)]
pub(crate) struct DashboardViewModel {
    history: Vec<HistoryEntry>,
    history_read_at: Option<f64>,
}

impl DashboardViewModel {
    /// Reads the recent removals again when the list is stale.
    pub(crate) fn refresh(&mut self, services: &UiServices, now: f64) {
        let is_stale = self
            .history_read_at
            .is_none_or(|read_at| now - read_at >= HISTORY_REFRESH_SECONDS);
        if is_stale {
            self.history = services.history.recent(HISTORY_ROWS);
            self.history_read_at = Some(now);
        }
    }

    pub(crate) fn history(&self) -> &[HistoryEntry] {
        &self.history
    }
}

#[cfg(test)]
mod tests {
    use diskmarshal_domain::shared::ByteSize;

    use super::Recoverable;

    #[test]
    fn the_total_counts_only_what_was_analyzed() {
        assert_eq!(Recoverable::default().total(), None);
        let partial = Recoverable {
            cleanup: Some(ByteSize::from_bytes(300)),
            duplicates: Some(ByteSize::from_bytes(50)),
            ..Recoverable::default()
        };
        assert_eq!(partial.total(), Some(ByteSize::from_bytes(350)));
    }
}

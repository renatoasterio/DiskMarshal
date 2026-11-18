//! Use case: remove the Docker resources the user approved.
//!
//! What Docker holds is not made of files this app can reach, so this is the
//! one removal that does not go through `DeleteItems`: Docker itself removes
//! each resource, and refuses any that came into use since the preview.

use diskmarshal_domain::deletion::{DeletionOutcome, DeletionStatus, SkipReason};
use diskmarshal_domain::docker::DockerResource;

use crate::ports::{ContainerEngine, HistoryRepository};
use crate::use_cases::DeletionReport;

/// Removes approved Docker resources and records what was removed.
pub struct PruneDocker<'a> {
    engine: &'a dyn ContainerEngine,
    history: &'a dyn HistoryRepository,
}

impl<'a> PruneDocker<'a> {
    /// Creates the use case over the Docker engine and the history.
    #[must_use]
    pub fn new(engine: &'a dyn ContainerEngine, history: &'a dyn HistoryRepository) -> Self {
        Self { engine, history }
    }

    /// Asks Docker to remove every resource and reports what happened to
    /// each. One refusal does not stop the others.
    ///
    /// The space reported as freed is what Docker's own disk use dropped
    /// by, measured before and after. Images share layers, so adding up the
    /// sizes of what was removed would promise too much. When Docker cannot
    /// be measured, that sum is all there is and is reported instead.
    #[must_use]
    pub fn execute(&self, resources: &[DockerResource]) -> DeletionReport {
        let before = self.engine.disk_usage().ok();
        let mut report = DeletionReport::default();
        for resource in resources {
            let status = match self.engine.remove(resource) {
                Ok(()) => DeletionStatus::Deleted,
                Err(error) => DeletionStatus::Skipped(SkipReason::Failed(error.to_string())),
            };
            let outcome = DeletionOutcome {
                target: resource.history_label(),
                status,
                reclaimable: resource.size,
            };
            report.record_in_history(outcome, self.history);
        }

        let removed_something = report.outcomes.iter().any(DeletionOutcome::is_removed);
        if removed_something && let (Some(before), Ok(after)) = (before, self.engine.disk_usage()) {
            report.reclaimed = before.saturating_sub(after);
        }
        report
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Mutex;

    use diskmarshal_domain::deletion::{DeletionOutcome, DeletionStatus, HistoryEntry, SkipReason};
    use diskmarshal_domain::docker::{DockerInventory, DockerResource, EngineStatus, ResourceKind};
    use diskmarshal_domain::shared::ByteSize;

    use super::PruneDocker;
    use crate::ApplicationError;
    use crate::ports::{ContainerEngine, EngineError, HistoryRepository};

    /// An engine that remembers what it was asked to remove, refuses the
    /// resources named in `in_use`, and reports a disk use that drops by
    /// `freed_per_removal` each time something is removed.
    struct FakeEngine {
        in_use: Vec<&'static str>,
        removed: Mutex<Vec<String>>,
        usage: u64,
        freed_per_removal: u64,
        is_measurable: bool,
    }

    impl FakeEngine {
        fn new(in_use: &[&'static str]) -> Self {
            Self {
                in_use: in_use.to_vec(),
                removed: Mutex::new(Vec::new()),
                usage: 10_000,
                freed_per_removal: 300,
                is_measurable: true,
            }
        }
    }

    impl ContainerEngine for FakeEngine {
        fn status(&self) -> EngineStatus {
            EngineStatus::Running {
                version: "24.0.2".into(),
            }
        }

        fn inventory(&self) -> Result<DockerInventory, EngineError> {
            Ok(DockerInventory::default())
        }

        fn disk_usage(&self) -> Result<ByteSize, EngineError> {
            if !self.is_measurable {
                return Err(EngineError::TimedOut);
            }
            let removed = self.removed.lock().map_or(0, |removed| removed.len()) as u64;
            Ok(ByteSize::from_bytes(
                self.usage - removed * self.freed_per_removal,
            ))
        }

        fn remove(&self, resource: &DockerResource) -> Result<(), EngineError> {
            if self.in_use.contains(&resource.reference.as_str()) {
                return Err(EngineError::Failed("in use".into()));
            }
            if let Ok(mut removed) = self.removed.lock() {
                removed.push(resource.reference.clone());
            }
            Ok(())
        }
    }

    /// A history that remembers what was appended, or refuses everything.
    #[derive(Default)]
    struct RecordingHistory {
        appended: Mutex<Vec<PathBuf>>,
        is_broken: bool,
    }

    impl HistoryRepository for RecordingHistory {
        fn append(&self, outcome: &DeletionOutcome) -> Result<(), ApplicationError> {
            if self.is_broken {
                return Err(ApplicationError::Cancelled);
            }
            if let Ok(mut appended) = self.appended.lock() {
                appended.push(outcome.target.clone());
            }
            Ok(())
        }

        fn recent(&self, _limit: usize) -> Vec<HistoryEntry> {
            Vec::new()
        }
    }

    fn image(name: &'static str) -> DockerResource {
        DockerResource {
            kind: ResourceKind::Image,
            reference: name.into(),
            name: name.into(),
            size: ByteSize::from_bytes(1_000),
            created_unix_seconds: None,
        }
    }

    #[test]
    fn removes_through_docker_and_reports_the_space_docker_really_freed() {
        let engine = FakeEngine::new(&[]);
        let history = RecordingHistory::default();

        let report = PruneDocker::new(&engine, &history).execute(&[image("web:1"), image("web:2")]);

        assert_eq!(*engine.removed.lock().unwrap(), ["web:1", "web:2"]);
        assert_eq!(
            report.reclaimed,
            ByteSize::from_bytes(600),
            "the two images shared layers: less than their sizes added up"
        );
        assert_eq!(
            *history.appended.lock().unwrap(),
            [
                PathBuf::from("docker/image/web:1"),
                PathBuf::from("docker/image/web:2")
            ]
        );
    }

    #[test]
    fn a_resource_docker_refuses_is_skipped_and_the_others_still_go() {
        let engine = FakeEngine::new(&["db:1"]);
        let history = RecordingHistory::default();

        let report = PruneDocker::new(&engine, &history).execute(&[image("db:1"), image("web:1")]);

        assert!(matches!(
            report.outcomes[0].status,
            DeletionStatus::Skipped(SkipReason::Failed(_))
        ));
        assert_eq!(report.outcomes[1].status, DeletionStatus::Deleted);
        assert_eq!(report.reclaimed, ByteSize::from_bytes(300));
        assert_eq!(history.appended.lock().unwrap().len(), 1);
    }

    #[test]
    fn when_nothing_is_removed_nothing_is_reported_as_freed() {
        let engine = FakeEngine::new(&["db:1"]);
        let history = RecordingHistory::default();

        let report = PruneDocker::new(&engine, &history).execute(&[image("db:1")]);

        assert_eq!(report.reclaimed, ByteSize::ZERO);
        assert!(history.appended.lock().unwrap().is_empty());
    }

    #[test]
    fn falls_back_to_the_listed_sizes_when_docker_cannot_be_measured() {
        let engine = FakeEngine {
            is_measurable: false,
            ..FakeEngine::new(&[])
        };
        let history = RecordingHistory::default();

        let report = PruneDocker::new(&engine, &history).execute(&[image("web:1")]);

        assert_eq!(report.reclaimed, ByteSize::from_bytes(1_000));
    }

    #[test]
    fn counts_removals_the_history_could_not_keep() {
        let engine = FakeEngine::new(&[]);
        let history = RecordingHistory {
            is_broken: true,
            ..RecordingHistory::default()
        };

        let report = PruneDocker::new(&engine, &history).execute(&[image("web:1")]);

        assert_eq!(report.unrecorded, 1);
    }
}

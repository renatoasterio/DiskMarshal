//! Use case: remove the items the user approved. The only code path that deletes.

use std::path::PathBuf;

use maccleaner_domain::deletion::{
    DeletionMethod, DeletionOutcome, DeletionRequest, DeletionStatus, ProtectedPaths, SkipReason,
};
use maccleaner_domain::shared::ByteSize;

use crate::ports::{FileInspector, FileRemover, HistoryRepository, TrashBin};

/// Whether a deletion really removes anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Runs every check and reports what would be removed. Touches nothing.
    DryRun,
    /// Removes the items.
    Real,
}

/// The adapters a deletion needs.
#[derive(Clone, Copy)]
pub struct DeletionPorts<'a> {
    /// Reads the real path and identity of an item.
    pub inspector: &'a dyn FileInspector,
    /// Removes items permanently.
    pub remover: &'a dyn FileRemover,
    /// Moves items to the Trash.
    pub trash_bin: &'a dyn TrashBin,
    /// Records what was removed.
    pub history: &'a dyn HistoryRepository,
}

/// What a deletion did, item by item.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeletionReport {
    /// One outcome per request, in the same order.
    pub outcomes: Vec<DeletionOutcome>,
    /// Space freed at once by permanent removals, as estimated in the preview.
    pub reclaimed: ByteSize,
    /// Space that will be freed when the user empties the Trash.
    pub trashed: ByteSize,
    /// Removals that happened but could not be written to the history.
    pub unrecorded: usize,
}

/// Removes approved items, checking each one again right before touching it.
pub struct DeleteItems<'a> {
    policy: &'a ProtectedPaths,
    ports: DeletionPorts<'a>,
}

impl<'a> DeleteItems<'a> {
    /// Creates the use case over the deletion policy and its adapters.
    #[must_use]
    pub fn new(policy: &'a ProtectedPaths, ports: DeletionPorts<'a>) -> Self {
        Self { policy, ports }
    }

    /// Processes every request and reports what happened to each.
    ///
    /// A request is skipped, never forced, when the policy forbids its path,
    /// when the item is gone, or when it changed since the preview. One
    /// failure does not stop the others. `allowed_roots` are the folders the
    /// user scanned; nothing outside them is ever removed.
    #[must_use]
    pub fn execute(
        &self,
        requests: &[DeletionRequest],
        allowed_roots: &[PathBuf],
        mode: ExecutionMode,
    ) -> DeletionReport {
        let roots: Vec<PathBuf> = allowed_roots
            .iter()
            .filter_map(|root| self.ports.inspector.resolve(root))
            .collect();

        let mut report = DeletionReport::default();
        for request in requests {
            let outcome = self.process(request, &roots, mode);
            match outcome.status {
                DeletionStatus::Deleted => report.reclaimed += outcome.reclaimable,
                DeletionStatus::Trashed => report.trashed += outcome.reclaimable,
                DeletionStatus::WouldDelete | DeletionStatus::Skipped(_) => {}
            }
            if outcome.is_removed() && self.ports.history.append(&outcome).is_err() {
                report.unrecorded += 1;
            }
            report.outcomes.push(outcome);
        }
        report
    }

    fn process(
        &self,
        request: &DeletionRequest,
        roots: &[PathBuf],
        mode: ExecutionMode,
    ) -> DeletionOutcome {
        let Some(target) = self.ports.inspector.resolve(&request.target) else {
            return Self::outcome(
                request.target.clone(),
                request,
                DeletionStatus::Skipped(SkipReason::Missing),
            );
        };
        let status = self.decide(request, &target, roots, mode);
        Self::outcome(target, request, status)
    }

    fn decide(
        &self,
        request: &DeletionRequest,
        target: &std::path::Path,
        roots: &[PathBuf],
        mode: ExecutionMode,
    ) -> DeletionStatus {
        if let Err(reason) = self.policy.check(target, roots) {
            return DeletionStatus::Skipped(SkipReason::Protected(reason));
        }
        match self.ports.inspector.fingerprint(target) {
            None => return DeletionStatus::Skipped(SkipReason::Missing),
            Some(current) if current != request.fingerprint => {
                return DeletionStatus::Skipped(SkipReason::ChangedSincePreview);
            }
            Some(_) => {}
        }
        if mode == ExecutionMode::DryRun {
            return DeletionStatus::WouldDelete;
        }

        let (result, removed) = match request.method {
            DeletionMethod::Trash => (
                self.ports.trash_bin.move_to_trash(target),
                DeletionStatus::Trashed,
            ),
            DeletionMethod::Permanent => {
                (self.ports.remover.remove(target), DeletionStatus::Deleted)
            }
        };
        match result {
            Ok(()) => removed,
            Err(error) => DeletionStatus::Skipped(SkipReason::Failed(error.to_string())),
        }
    }

    fn outcome(
        target: PathBuf,
        request: &DeletionRequest,
        status: DeletionStatus,
    ) -> DeletionOutcome {
        DeletionOutcome {
            target,
            status,
            reclaimable: request.reclaimable,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;

    use maccleaner_domain::deletion::{
        DeletionMethod, DeletionOutcome, DeletionRequest, DeletionStatus, ItemFingerprint,
        ProtectedPaths, ProtectionReason, SkipReason,
    };
    use maccleaner_domain::shared::ByteSize;

    use super::{DeleteItems, DeletionPorts, DeletionReport, ExecutionMode};
    use crate::ApplicationError;
    use crate::ports::{FileInspector, FileRemover, HistoryRepository, TrashBin};

    const HOME: &str = "/Users/tester";
    const ROOT: &str = "/Users/tester/code";

    fn fingerprint(inode: u64) -> ItemFingerprint {
        ItemFingerprint {
            device: 1,
            inode,
            size: 10,
            modified_unix_seconds: 100,
        }
    }

    /// An in-memory disk: knows which paths exist and records every removal.
    #[derive(Default)]
    struct FakeDisk {
        items: HashMap<PathBuf, ItemFingerprint>,
        links: HashMap<PathBuf, PathBuf>,
        removed: Mutex<Vec<PathBuf>>,
        trashed: Mutex<Vec<PathBuf>>,
        recorded: Mutex<Vec<PathBuf>>,
        fails_removal: bool,
    }

    impl FakeDisk {
        fn with_items(paths: &[&str]) -> Self {
            let items = paths
                .iter()
                .zip(1..)
                .map(|(path, inode)| (PathBuf::from(path), fingerprint(inode)))
                .collect();
            Self {
                items,
                ..Self::default()
            }
        }

        fn request(&self, path: &str, method: DeletionMethod) -> DeletionRequest {
            DeletionRequest {
                target: PathBuf::from(path),
                method,
                fingerprint: self
                    .items
                    .get(Path::new(path))
                    .copied()
                    .unwrap_or(fingerprint(999)),
                reclaimable: ByteSize::from_bytes(1_000),
            }
        }

        fn run(&self, requests: &[DeletionRequest], mode: ExecutionMode) -> DeletionReport {
            let policy = ProtectedPaths::new(Path::new(HOME));
            let ports = DeletionPorts {
                inspector: self,
                remover: self,
                trash_bin: self,
                history: self,
            };
            DeleteItems::new(&policy, ports).execute(requests, &[PathBuf::from(ROOT)], mode)
        }

        fn touched(&self) -> usize {
            self.removed.lock().unwrap().len() + self.trashed.lock().unwrap().len()
        }
    }

    impl FileInspector for FakeDisk {
        fn resolve(&self, path: &Path) -> Option<PathBuf> {
            Some(
                self.links
                    .get(path)
                    .cloned()
                    .unwrap_or_else(|| path.to_path_buf()),
            )
        }

        fn fingerprint(&self, path: &Path) -> Option<ItemFingerprint> {
            self.items.get(path).copied()
        }
    }

    impl FileRemover for FakeDisk {
        fn remove(&self, path: &Path) -> Result<(), ApplicationError> {
            if self.fails_removal {
                return Err(ApplicationError::Cancelled);
            }
            self.removed.lock().unwrap().push(path.to_path_buf());
            Ok(())
        }
    }

    impl TrashBin for FakeDisk {
        fn move_to_trash(&self, path: &Path) -> Result<(), ApplicationError> {
            self.trashed.lock().unwrap().push(path.to_path_buf());
            Ok(())
        }
    }

    impl HistoryRepository for FakeDisk {
        fn append(&self, outcome: &DeletionOutcome) -> Result<(), ApplicationError> {
            self.recorded.lock().unwrap().push(outcome.target.clone());
            Ok(())
        }
    }

    #[test]
    fn removes_permanently_or_to_the_trash_as_requested() {
        let disk = FakeDisk::with_items(&[
            "/Users/tester/code/web/node_modules",
            "/Users/tester/code/copy.mov",
        ]);
        let requests = [
            disk.request(
                "/Users/tester/code/web/node_modules",
                DeletionMethod::Permanent,
            ),
            disk.request("/Users/tester/code/copy.mov", DeletionMethod::Trash),
        ];

        let report = disk.run(&requests, ExecutionMode::Real);

        assert_eq!(
            *disk.removed.lock().unwrap(),
            [PathBuf::from("/Users/tester/code/web/node_modules")]
        );
        assert_eq!(
            *disk.trashed.lock().unwrap(),
            [PathBuf::from("/Users/tester/code/copy.mov")]
        );
        assert_eq!(report.reclaimed, ByteSize::from_bytes(1_000));
        assert_eq!(report.trashed, ByteSize::from_bytes(1_000));
        assert_eq!(disk.recorded.lock().unwrap().len(), 2);
    }

    #[test]
    fn a_dry_run_touches_nothing_and_lists_what_a_real_run_removes() {
        let paths = ["/Users/tester/code/a/target", "/Users/tester/code/b/target"];
        let disk = FakeDisk::with_items(&paths);
        let requests: Vec<_> = paths
            .iter()
            .map(|path| disk.request(path, DeletionMethod::Permanent))
            .collect();

        let dry = disk.run(&requests, ExecutionMode::DryRun);
        assert_eq!(disk.touched(), 0);
        assert!(disk.recorded.lock().unwrap().is_empty());
        assert!(
            dry.outcomes
                .iter()
                .all(|o| o.status == DeletionStatus::WouldDelete)
        );

        let real = disk.run(&requests, ExecutionMode::Real);
        let dry_targets: Vec<_> = dry.outcomes.iter().map(|o| &o.target).collect();
        let real_targets: Vec<_> = real
            .outcomes
            .iter()
            .filter(|o| o.is_removed())
            .map(|o| &o.target)
            .collect();
        assert_eq!(dry_targets, real_targets);
    }

    #[test]
    fn skips_protected_paths_without_touching_them() {
        let disk = FakeDisk::with_items(&[
            "/Users/tester/code/web/.git",
            "/Users/tester/Documents",
            "/Users/tester/code",
        ]);
        let requests = [
            disk.request("/Users/tester/code/web/.git", DeletionMethod::Permanent),
            disk.request("/Users/tester/Documents", DeletionMethod::Permanent),
            disk.request("/Users/tester/code", DeletionMethod::Permanent),
        ];

        let report = disk.run(&requests, ExecutionMode::Real);

        assert_eq!(disk.touched(), 0);
        let reasons: Vec<_> = report.outcomes.iter().map(|o| o.status.clone()).collect();
        assert_eq!(
            reasons,
            [
                DeletionStatus::Skipped(SkipReason::Protected(ProtectionReason::InsideGitFolder)),
                DeletionStatus::Skipped(SkipReason::Protected(ProtectionReason::ProtectedLocation)),
                DeletionStatus::Skipped(SkipReason::Protected(ProtectionReason::IsAllowedRoot)),
            ]
        );
    }

    #[test]
    fn skips_an_item_that_changed_since_the_preview() {
        let disk = FakeDisk::with_items(&["/Users/tester/code/a/target"]);
        let mut request = disk.request("/Users/tester/code/a/target", DeletionMethod::Permanent);
        request.fingerprint.modified_unix_seconds += 1;

        let report = disk.run(&[request], ExecutionMode::Real);

        assert_eq!(disk.touched(), 0);
        assert_eq!(
            report.outcomes[0].status,
            DeletionStatus::Skipped(SkipReason::ChangedSincePreview)
        );
    }

    #[test]
    fn skips_an_item_that_no_longer_exists() {
        let disk = FakeDisk::default();
        let request = disk.request("/Users/tester/code/gone", DeletionMethod::Permanent);

        let report = disk.run(&[request], ExecutionMode::Real);

        assert_eq!(
            report.outcomes[0].status,
            DeletionStatus::Skipped(SkipReason::Missing)
        );
    }

    #[test]
    fn judges_the_real_location_of_a_path_that_goes_through_a_link() {
        let mut disk = FakeDisk::with_items(&["/Users/tester/Documents/thesis"]);
        disk.links.insert(
            PathBuf::from("/Users/tester/code/shortcut/thesis"),
            PathBuf::from("/Users/tester/Documents/thesis"),
        );
        let mut request = disk.request(
            "/Users/tester/code/shortcut/thesis",
            DeletionMethod::Permanent,
        );
        request.fingerprint = fingerprint(1);

        let report = disk.run(&[request], ExecutionMode::Real);

        assert_eq!(disk.touched(), 0);
        assert_eq!(
            report.outcomes[0].status,
            DeletionStatus::Skipped(SkipReason::Protected(ProtectionReason::OutsideAllowedRoots))
        );
    }

    #[test]
    fn one_failure_does_not_stop_the_other_items() {
        let paths = ["/Users/tester/code/a/target", "/Users/tester/code/b.mov"];
        let mut disk = FakeDisk::with_items(&paths);
        disk.fails_removal = true;
        let requests = [
            disk.request(paths[0], DeletionMethod::Permanent),
            disk.request(paths[1], DeletionMethod::Trash),
        ];

        let report = disk.run(&requests, ExecutionMode::Real);

        assert!(matches!(
            report.outcomes[0].status,
            DeletionStatus::Skipped(SkipReason::Failed(_))
        ));
        assert_eq!(report.outcomes[1].status, DeletionStatus::Trashed);
        assert_eq!(report.reclaimed, ByteSize::ZERO);
    }
}

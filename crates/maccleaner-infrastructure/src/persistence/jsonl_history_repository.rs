//! Removal history stored as one JSON object per line.

use std::fs::{DirBuilder, OpenOptions};
use std::io::{self, Write as _};
use std::os::unix::fs::{DirBuilderExt as _, OpenOptionsExt as _};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use maccleaner_application::ApplicationError;
use maccleaner_application::ports::HistoryRepository;
use maccleaner_domain::deletion::{DeletionOutcome, DeletionStatus};
use serde::Serialize;

/// Only the owner may enter the folder or read the file.
const DIRECTORY_MODE: u32 = 0o700;
const FILE_MODE: u32 = 0o600;

/// One line of the history file.
#[derive(Serialize)]
struct HistoryRecord<'a> {
    timestamp_unix_seconds: u64,
    action: &'static str,
    path: &'a str,
    reclaimable_bytes: u64,
}

/// Appends every removal to a JSON Lines file readable only by its owner.
#[derive(Debug, Clone)]
pub struct JsonlHistoryRepository {
    file: PathBuf,
}

impl JsonlHistoryRepository {
    /// Location of the history, relative to the home folder.
    const DEFAULT_RELATIVE_PATH: &'static str =
        "Library/Application Support/MacCleaner/history.jsonl";

    /// Creates a repository that writes to `file`.
    #[must_use]
    pub fn new(file: PathBuf) -> Self {
        Self { file }
    }

    /// Creates a repository at the standard location for the given home folder.
    #[must_use]
    pub fn for_home(home: &Path) -> Self {
        Self::new(home.join(Self::DEFAULT_RELATIVE_PATH))
    }

    fn write(&self, record: &HistoryRecord<'_>) -> io::Result<()> {
        if let Some(parent) = self.file.parent() {
            DirBuilder::new()
                .recursive(true)
                .mode(DIRECTORY_MODE)
                .create(parent)?;
        }
        let mut line = serde_json::to_string(record)?;
        line.push('\n');
        // One `write` of a whole line in append mode keeps concurrent
        // writers from interleaving records.
        OpenOptions::new()
            .create(true)
            .append(true)
            .mode(FILE_MODE)
            .open(&self.file)?
            .write_all(line.as_bytes())
    }
}

impl HistoryRepository for JsonlHistoryRepository {
    fn append(&self, outcome: &DeletionOutcome) -> Result<(), ApplicationError> {
        let action = match outcome.status {
            DeletionStatus::Deleted => "deleted",
            DeletionStatus::Trashed => "trashed",
            DeletionStatus::WouldDelete | DeletionStatus::Skipped(_) => return Ok(()),
        };
        let timestamp_unix_seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_secs());
        let path = outcome.target.to_string_lossy();
        let record = HistoryRecord {
            timestamp_unix_seconds,
            action,
            path: &path,
            reclaimable_bytes: outcome.reclaimable.as_bytes(),
        };
        self.write(&record).map_err(|source| ApplicationError::Io {
            path: self.file.clone(),
            source,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt as _;
    use std::path::PathBuf;

    use maccleaner_application::ports::HistoryRepository;
    use maccleaner_domain::deletion::{DeletionOutcome, DeletionStatus, SkipReason};
    use maccleaner_domain::shared::ByteSize;
    use maccleaner_test_support::TempTree;

    use super::JsonlHistoryRepository;

    fn outcome(path: &str, status: DeletionStatus) -> DeletionOutcome {
        DeletionOutcome {
            target: PathBuf::from(path),
            status,
            reclaimable: ByteSize::from_bytes(4_096),
        }
    }

    #[test]
    fn appends_one_json_line_per_removal() {
        let fixture = TempTree::new().unwrap();
        let file = fixture.path("support/history.jsonl");
        let history = JsonlHistoryRepository::new(file.clone());

        history
            .append(&outcome("/tmp/cache \"a\"", DeletionStatus::Deleted))
            .unwrap();
        history
            .append(&outcome("/tmp/copy.mov", DeletionStatus::Trashed))
            .unwrap();

        let content = fs::read_to_string(&file).unwrap();
        let records: Vec<serde_json::Value> = content
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0]["action"], "deleted");
        assert_eq!(records[0]["path"], "/tmp/cache \"a\"");
        assert_eq!(records[0]["reclaimable_bytes"], 4_096);
        assert_eq!(records[1]["action"], "trashed");
    }

    #[test]
    fn the_history_file_is_readable_only_by_its_owner() {
        let fixture = TempTree::new().unwrap();
        let file = fixture.path("support/history.jsonl");
        let history = JsonlHistoryRepository::new(file.clone());

        history
            .append(&outcome("/tmp/x", DeletionStatus::Deleted))
            .unwrap();

        let mode = fs::metadata(&file).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn items_that_were_not_removed_are_not_recorded() {
        let fixture = TempTree::new().unwrap();
        let file = fixture.path("history.jsonl");
        let history = JsonlHistoryRepository::new(file.clone());

        history
            .append(&outcome("/tmp/x", DeletionStatus::WouldDelete))
            .unwrap();
        history
            .append(&outcome(
                "/tmp/y",
                DeletionStatus::Skipped(SkipReason::Missing),
            ))
            .unwrap();

        assert!(!file.exists());
    }
}

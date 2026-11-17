//! Ports: what the use cases need from the outside world.

mod file_inspector;
mod file_remover;
mod file_system_walker;
mod history_repository;
mod trash_bin;
mod volume_stats;
mod wall_clock;

pub use file_inspector::FileInspector;
pub use file_remover::FileRemover;
pub use file_system_walker::{FileSystemWalker, WalkOutcome};
pub use history_repository::HistoryRepository;
pub use trash_bin::TrashBin;
pub use volume_stats::VolumeStats;
pub use wall_clock::WallClock;

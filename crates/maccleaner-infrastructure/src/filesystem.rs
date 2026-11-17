//! File system adapters.

mod dataless_policy;
mod lstat_file_inspector;
mod readdir_walker;
mod safe_file_remover;
mod statfs_volume_stats;
mod sync_roots;

pub use dataless_policy::disable_dataless_materialization;
pub use lstat_file_inspector::LstatFileInspector;
pub use readdir_walker::ReaddirWalker;
pub use safe_file_remover::SafeFileRemover;
pub use statfs_volume_stats::StatfsVolumeStats;
pub use sync_roots::icloud_sync_roots;

//! File system adapters.

mod dataless_policy;
mod lstat_file_inspector;
mod readdir_folder_lister;
mod readdir_walker;
mod safe_file_remover;
mod statfs_volume_stats;
mod sync_roots;

pub use dataless_policy::disable_dataless_materialization;
pub use lstat_file_inspector::LstatFileInspector;
pub use readdir_folder_lister::ReaddirFolderLister;
pub use readdir_walker::ReaddirWalker;
pub use safe_file_remover::SafeFileRemover;
pub use statfs_volume_stats::StatfsVolumeStats;
pub use sync_roots::icloud_sync_roots;

/// Unit of `st_blocks`, fixed by POSIX regardless of the volume's block size.
const STAT_BLOCK_BYTES: u64 = 512;

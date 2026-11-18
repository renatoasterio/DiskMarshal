//! Ports: what the use cases need from the outside world.

mod content_hasher;
mod external_opener;
mod file_inspector;
mod file_remover;
mod file_system_walker;
mod folder_picker;
mod history_repository;
mod image_thumbnailer;
mod metrics_provider;
mod permission_probe;
mod privileged_cleaner;
mod rule_catalog;
mod trash_bin;
mod version_control_probe;
mod volume_stats;
mod wall_clock;

pub use content_hasher::ContentHasher;
pub use external_opener::ExternalOpener;
pub use file_inspector::FileInspector;
pub use file_remover::FileRemover;
pub use file_system_walker::{FileSystemWalker, WalkOutcome};
pub use folder_picker::FolderPicker;
pub use history_repository::HistoryRepository;
pub use image_thumbnailer::ImageThumbnailer;
pub use metrics_provider::MetricsProvider;
pub use permission_probe::{AccessStatus, PermissionProbe};
pub use privileged_cleaner::{PrivilegeError, PrivilegedCleaner};
pub use rule_catalog::RuleCatalog;
pub use trash_bin::TrashBin;
pub use version_control_probe::VersionControlProbe;
pub use volume_stats::VolumeStats;
pub use wall_clock::WallClock;

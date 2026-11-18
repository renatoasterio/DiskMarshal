//! Use cases: one module per thing the user can ask DiskMarshal to do.

mod clean_rules;
mod clean_system_areas;
mod delete_items;
mod find_dev_artifacts;
mod find_duplicates;
mod find_similar_photos;
mod inspect_docker;
mod list_installed_apps;
mod plan_cleanup;
mod plan_uninstall;
mod prune_docker;
mod remove_app_items;
mod scan_disk;
mod trash_requests;
mod uninstall_system_items;

pub use clean_rules::CleanRules;
pub use clean_system_areas::CleanSystemAreas;
pub use delete_items::{DeleteItems, DeletionPorts, DeletionReport, ExecutionMode};
pub use find_dev_artifacts::{DevArtifactsReport, FindDevArtifacts, FoundArtifact};
pub use find_duplicates::{DuplicatesReport, FindDuplicates, FoundDuplicates};
pub use find_similar_photos::{
    FindSimilarPhotos, FoundSimilarPhotos, SimilarPhotosOptions, SimilarPhotosReport,
};
pub use inspect_docker::InspectDocker;
pub use list_installed_apps::ListInstalledApps;
pub use plan_cleanup::PlanCleanup;
pub use plan_uninstall::PlanUninstall;
pub use prune_docker::PruneDocker;
pub use remove_app_items::RemoveAppItems;
pub use scan_disk::{ScanDisk, ScanReport};
pub use uninstall_system_items::UninstallSystemItems;

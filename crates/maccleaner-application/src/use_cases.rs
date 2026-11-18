//! Use cases: one module per thing the user can ask MacCleaner to do.

mod clean_rules;
mod clean_system_areas;
mod delete_items;
mod find_dev_artifacts;
mod find_duplicates;
mod find_similar_photos;
mod plan_cleanup;
mod scan_disk;
mod trash_requests;

pub use clean_rules::CleanRules;
pub use clean_system_areas::CleanSystemAreas;
pub use delete_items::{DeleteItems, DeletionPorts, DeletionReport, ExecutionMode};
pub use find_dev_artifacts::{DevArtifactsReport, FindDevArtifacts, FoundArtifact};
pub use find_duplicates::{DuplicatesReport, FindDuplicates, FoundDuplicates};
pub use find_similar_photos::{
    FindSimilarPhotos, FoundSimilarPhotos, SimilarPhotosOptions, SimilarPhotosReport,
};
pub use plan_cleanup::PlanCleanup;
pub use scan_disk::{ScanDisk, ScanReport};

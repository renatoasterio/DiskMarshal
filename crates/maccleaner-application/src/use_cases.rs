//! Use cases: one module per thing the user can ask MacCleaner to do.

mod delete_items;
mod scan_disk;

pub use delete_items::{DeleteItems, DeletionPorts, DeletionReport, ExecutionMode};
pub use scan_disk::{ScanDisk, ScanReport};

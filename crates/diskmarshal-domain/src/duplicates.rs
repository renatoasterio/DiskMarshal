//! Duplicate files: groups of files with identical content.

mod content_hash;
mod duplicate_candidates;
mod duplicate_group;

pub use content_hash::ContentHash;
pub use duplicate_candidates::candidates_by_size;
pub use duplicate_group::{DuplicateCopy, DuplicateGroup};

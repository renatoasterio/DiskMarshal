//! Deletion model: what may be removed, how, and what happened.

mod deletion_outcome;
mod deletion_request;
mod item_fingerprint;
mod protected_paths;

pub use deletion_outcome::{DeletionOutcome, DeletionStatus, SkipReason};
pub use deletion_request::{DeletionMethod, DeletionRequest};
pub use item_fingerprint::ItemFingerprint;
pub use protected_paths::{ProtectedPaths, ProtectionReason};

//! Domain layer of MacCleaner: entities, value objects and pure rules.
//!
//! This crate performs no I/O and depends on no other MacCleaner crate, so every
//! rule in it can be tested without touching the disk.

#![forbid(unsafe_code)]

pub mod cleanup;
pub mod deletion;
pub mod dev_artifacts;
pub mod duplicates;
pub mod monitoring;
pub mod photos;
pub mod scan;
pub mod shared;
pub mod treemap;

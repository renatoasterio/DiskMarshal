//! Domain layer of DiskMarshal: entities, value objects and pure rules.
//!
//! This crate performs no I/O and depends on no other DiskMarshal crate, so every
//! rule in it can be tested without touching the disk.

#![forbid(unsafe_code)]

pub mod applications;
pub mod cleanup;
pub mod deletion;
pub mod dev_artifacts;
pub mod docker;
pub mod duplicates;
pub mod monitoring;
pub mod network;
pub mod photos;
pub mod scan;
pub mod shared;
pub mod treemap;

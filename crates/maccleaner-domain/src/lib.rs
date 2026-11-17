//! Domain layer of MacCleaner: entities, value objects and pure rules.
//!
//! This crate performs no I/O and depends on no other MacCleaner crate, so every
//! rule in it can be tested without touching the disk.

#![forbid(unsafe_code)]

pub mod deletion;
pub mod monitoring;
pub mod scan;
pub mod shared;

//! Infrastructure layer of DiskMarshal: adapters that talk to macOS.
//!
//! This is the only library crate allowed to contain `unsafe` code. Every
//! `unsafe` block lives in a small module and carries a `SAFETY:` comment.

pub mod bundles;
pub mod catalog;
pub mod filesystem;
pub mod hashing;
pub mod imaging;
pub mod metrics;
pub mod network;
pub mod permissions;
pub mod persistence;
pub mod privilege;
pub mod process;
pub mod system;
pub mod trash;

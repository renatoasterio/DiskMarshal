//! Infrastructure layer of MacCleaner: adapters that talk to macOS.
//!
//! This is the only library crate allowed to contain `unsafe` code. Every
//! `unsafe` block lives in a small module and carries a `SAFETY:` comment.

pub mod filesystem;
pub mod persistence;
pub mod system;
pub mod trash;

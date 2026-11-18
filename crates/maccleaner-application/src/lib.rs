//! Application layer of MacCleaner: use cases and the ports they depend on.
//!
//! A use case orchestrates domain rules through ports (traits). Adapters that
//! talk to macOS live in `maccleaner-infrastructure` and are injected by the
//! binaries, so this crate never touches the operating system directly.

#![forbid(unsafe_code)]

mod application_error;
mod parallel;
pub mod ports;
pub mod progress;
pub mod use_cases;

pub use application_error::ApplicationError;

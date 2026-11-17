//! Fixtures shared by MacCleaner tests. Never linked into a shipped binary.

#![forbid(unsafe_code)]

mod temp_tree;

pub use temp_tree::TempTree;

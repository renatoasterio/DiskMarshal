//! Fixtures shared by DiskMarshal tests. Never linked into a shipped binary.

#![forbid(unsafe_code)]

mod bitmap;
mod temp_tree;

pub use bitmap::bitmap_bytes;
pub use temp_tree::TempTree;

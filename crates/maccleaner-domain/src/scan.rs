//! Disk scan model: the file tree, what each node is, and what a scan may enter.

mod cloud_state;
mod file_entry;
mod file_node;
mod file_tree;
mod scan_policy;

pub use cloud_state::CloudState;
pub use file_entry::{EntryKind, FileEntry};
pub use file_node::{FileNode, NodeId};
pub use file_tree::FileTree;
pub use scan_policy::ScanPolicy;

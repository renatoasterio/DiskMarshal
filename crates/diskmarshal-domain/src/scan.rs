//! Disk scan model: the file tree, what each node is, and what a scan may enter.

mod cloud_state;
mod file_category;
mod file_entry;
mod file_node;
mod file_tree;
mod local_file;
mod scan_policy;

pub use cloud_state::CloudState;
pub use file_category::FileCategory;
pub use file_entry::{EntryKind, FileEntry};
pub use file_node::{FileNode, NodeId};
pub use file_tree::FileTree;
pub use local_file::{LocalFile, local_files};
pub use scan_policy::ScanPolicy;

//! The result of a scan: every item found, arranged as a tree.

use crate::scan::{FileEntry, FileNode, NodeId};
use crate::shared::ByteSize;

/// Every item found by a scan, stored in one contiguous arena.
///
/// A child is always stored after its parent. Aggregation relies on that
/// invariant to roll sizes up in a single backwards pass.
#[derive(Debug, Clone)]
pub struct FileTree {
    nodes: Vec<FileNode>,
    is_aggregated: bool,
}

impl FileTree {
    /// The node every tree starts with.
    pub const ROOT: NodeId = NodeId(0);

    /// Creates a tree that contains only its root.
    #[must_use]
    pub fn new(root: FileEntry) -> Self {
        Self {
            nodes: vec![FileNode::new(root, None)],
            is_aggregated: false,
        }
    }

    /// Appends `entry` as the last child of `parent`.
    ///
    /// Returns `None` when `parent` is not part of this tree, when the tree has
    /// already been aggregated, or when the tree is full.
    pub fn add_child(&mut self, parent: NodeId, entry: FileEntry) -> Option<NodeId> {
        if self.is_aggregated {
            return None;
        }
        let child = NodeId(u32::try_from(self.nodes.len()).ok()?);
        let previous_last = {
            let parent_node = self.nodes.get_mut(parent.index())?;
            let previous_last = parent_node.last_child.replace(child);
            if previous_last.is_none() {
                parent_node.first_child = Some(child);
            }
            previous_last
        };
        if let Some(sibling) = previous_last.and_then(|id| self.nodes.get_mut(id.index())) {
            sibling.next_sibling = Some(child);
        }
        self.nodes.push(FileNode::new(entry, Some(parent)));
        Some(child)
    }

    /// Rolls sizes and item counts up from every node into its ancestors.
    ///
    /// Calling it again has no effect, so totals are never counted twice.
    pub fn aggregate(&mut self) {
        if self.is_aggregated {
            return;
        }
        for index in (1..self.nodes.len()).rev() {
            let Some(node) = self.nodes.get(index) else {
                continue;
            };
            let Some(parent) = node.parent else { continue };
            let (logical, allocated, cloud_only, items) = (
                node.entry.logical_size,
                node.entry.allocated_size,
                node.cloud_only_size,
                node.item_count,
            );
            if let Some(parent_node) = self.nodes.get_mut(parent.index()) {
                parent_node.entry.logical_size += logical;
                parent_node.entry.allocated_size += allocated;
                parent_node.cloud_only_size += cloud_only;
                parent_node.item_count = parent_node.item_count.saturating_add(items);
            }
        }
        self.is_aggregated = true;
    }

    /// Returns the node with the given id, when it belongs to this tree.
    #[must_use]
    pub fn node(&self, id: NodeId) -> Option<&FileNode> {
        self.nodes.get(id.index())
    }

    /// Returns the root node.
    #[must_use]
    pub fn root(&self) -> Option<&FileNode> {
        self.node(Self::ROOT)
    }

    /// Number of nodes in the tree, the root included.
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// A tree always holds at least its root, so this is always `false`.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Iterates over the direct children of `parent`, in insertion order.
    pub fn children(&self, parent: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        let first = self.node(parent).and_then(|node| node.first_child);
        std::iter::successors(first, |id| {
            self.node(*id).and_then(|node| node.next_sibling)
        })
    }

    /// Returns up to `limit` direct children of `parent`, largest on disk first.
    #[must_use]
    pub fn largest_children(&self, parent: NodeId, limit: usize) -> Vec<NodeId> {
        let mut children: Vec<NodeId> = self.children(parent).collect();
        children.sort_by_key(|id| {
            std::cmp::Reverse(
                self.node(*id)
                    .map_or(ByteSize::ZERO, FileNode::allocated_size),
            )
        });
        children.truncate(limit);
        children
    }

    /// Returns the names from the root down to `id`, the root's name included.
    #[must_use]
    pub fn path_segments(&self, id: NodeId) -> Vec<&str> {
        let mut segments: Vec<&str> =
            std::iter::successors(self.node(id), |node| node.parent.and_then(|p| self.node(p)))
                .map(FileNode::name)
                .collect();
        segments.reverse();
        segments
    }
}

#[cfg(test)]
mod tests {
    use super::FileTree;
    use crate::scan::{CloudState, FileEntry, FileNode, NodeId};
    use crate::shared::ByteSize;

    fn file(name: &str, bytes: u64) -> FileEntry {
        FileEntry::file(name, ByteSize::from_bytes(bytes))
    }

    /// root/{a.bin(10), docs/{b.bin(20), c.bin(30)}, d.bin(5)}
    fn sample_tree() -> (FileTree, NodeId) {
        let mut tree = FileTree::new(FileEntry::directory("root"));
        tree.add_child(FileTree::ROOT, file("a.bin", 10)).unwrap();
        let docs = tree
            .add_child(FileTree::ROOT, FileEntry::directory("docs"))
            .unwrap();
        tree.add_child(docs, file("b.bin", 20)).unwrap();
        tree.add_child(docs, file("c.bin", 30)).unwrap();
        tree.add_child(FileTree::ROOT, file("d.bin", 5)).unwrap();
        (tree, docs)
    }

    #[test]
    fn aggregate_rolls_sizes_and_counts_up_to_the_root() {
        let (mut tree, docs) = sample_tree();
        tree.aggregate();

        let root = tree.root().unwrap();
        assert_eq!(root.logical_size(), ByteSize::from_bytes(65));
        assert_eq!(root.allocated_size(), ByteSize::from_bytes(65));
        assert_eq!(root.item_count(), 6);
        assert_eq!(
            tree.node(docs).unwrap().logical_size(),
            ByteSize::from_bytes(50)
        );
        assert_eq!(tree.node(docs).unwrap().item_count(), 3);
    }

    #[test]
    fn aggregate_twice_does_not_count_twice() {
        let (mut tree, _) = sample_tree();
        tree.aggregate();
        tree.aggregate();
        assert_eq!(
            tree.root().unwrap().logical_size(),
            ByteSize::from_bytes(65)
        );
    }

    #[test]
    fn children_keep_insertion_order() {
        let (tree, _) = sample_tree();
        let names: Vec<&str> = tree
            .children(FileTree::ROOT)
            .filter_map(|id| tree.node(id))
            .map(FileNode::name)
            .collect();
        assert_eq!(names, ["a.bin", "docs", "d.bin"]);
    }

    #[test]
    fn largest_children_are_sorted_by_disk_usage() {
        let (mut tree, docs) = sample_tree();
        tree.aggregate();
        let largest = tree.largest_children(FileTree::ROOT, 2);
        assert_eq!(largest.first(), Some(&docs));
        assert_eq!(largest.len(), 2);
    }

    #[test]
    fn path_segments_run_from_the_root_to_the_node() {
        let (tree, docs) = sample_tree();
        let leaf = tree.children(docs).last().unwrap();
        assert_eq!(tree.path_segments(leaf), ["root", "docs", "c.bin"]);
    }

    #[test]
    fn cloud_only_content_is_totalled_separately() {
        let mut tree = FileTree::new(FileEntry::directory("root"));
        let mut placeholder = file("movie.mov", 1_000);
        placeholder.cloud_state = CloudState::CloudOnly;
        placeholder.allocated_size = ByteSize::ZERO;
        tree.add_child(FileTree::ROOT, placeholder).unwrap();
        tree.add_child(FileTree::ROOT, file("local.txt", 10))
            .unwrap();
        tree.aggregate();

        let root = tree.root().unwrap();
        assert_eq!(root.cloud_only_size(), ByteSize::from_bytes(1_000));
        assert_eq!(root.allocated_size(), ByteSize::from_bytes(10));
    }

    #[test]
    fn rejects_children_for_unknown_parents_and_after_aggregation() {
        let (mut tree, _) = sample_tree();
        let other = FileTree::new(FileEntry::directory("other"));
        let foreign = NodeId(u32::try_from(other.len() + 100).unwrap());
        assert!(tree.add_child(foreign, file("x", 1)).is_none());

        tree.aggregate();
        assert!(tree.add_child(FileTree::ROOT, file("late", 1)).is_none());
    }
}

//! State and logic of the disk map: the scan, where the user is in the tree,
//! and the blocks to draw.

use std::path::PathBuf;
use std::sync::Arc;

use diskmarshal_application::ports::FileSystemWalker;
use diskmarshal_application::progress::{CancellationToken, NoProgress, ScanProgress};
use diskmarshal_application::use_cases::{ScanDisk, ScanReport};
use diskmarshal_domain::scan::{CloudState, EntryKind, FileCategory, FileNode, FileTree, NodeId};
use diskmarshal_domain::shared::ByteSize;
use diskmarshal_domain::treemap::LayoutRect;
use egui::Context;

use crate::animation::Timeline;
use crate::i18n::pt_br;
use crate::tasks::Search;

/// Most blocks drawn at once. Smaller items are merged into one "others" block.
const MAX_BLOCKS: usize = 40;

/// What the colors of the map mean.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColorMode {
    /// Kind of content: code, photos, documents and so on.
    Category,
    /// Where the content lives: this Mac, iCloud and this Mac, or iCloud only.
    Cloud,
}

/// One rectangle of the map.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MapBlock {
    /// The node it stands for; `None` for the merged "others" block.
    pub node: Option<NodeId>,
    pub name: String,
    /// What its area is proportional to.
    pub weight: f64,
    pub allocated: ByteSize,
    pub logical: ByteSize,
    pub item_count: u64,
    pub category: FileCategory,
    pub cloud: CloudState,
    /// Whether clicking it shows what is inside.
    pub is_enterable: bool,
}

/// A zoom in progress: the map grows out of the block that was clicked.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Zoom {
    /// The clicked block, in coordinates from 0.0 to 1.0 of the map.
    pub origin: LayoutRect,
    pub timeline: Timeline,
}

/// Everything the disk map screen remembers between frames.
pub(crate) struct DiskMapViewModel {
    scan: Search<ScanReport>,
    report: Option<ScanReport>,
    trail: Vec<NodeId>,
    zoom: Option<Zoom>,
    pub color_mode: ColorMode,
}

impl Default for DiskMapViewModel {
    fn default() -> Self {
        Self {
            scan: Search::default(),
            report: None,
            trail: Vec::new(),
            zoom: None,
            color_mode: ColorMode::Category,
        }
    }
}

impl DiskMapViewModel {
    /// Starts scanning `root` in the background, replacing any scan under way.
    pub(crate) fn start_scan(
        &mut self,
        ctx: &Context,
        walker: Arc<dyn FileSystemWalker>,
        root: PathBuf,
    ) {
        self.scan.start(ctx, move |progress, cancellation| {
            ScanDisk::new(walker.as_ref()).execute(&root, progress, cancellation)
        });
    }

    /// Scans `root` right now, blocking until it ends. Used to open the app
    /// with a folder already measured.
    pub(crate) fn scan_now(&mut self, walker: &dyn FileSystemWalker, root: &std::path::Path) {
        let outcome = ScanDisk::new(walker).execute(root, &NoProgress, &CancellationToken::new());
        if let Some(report) = self.scan.settle(outcome) {
            self.accept(report);
        }
    }

    /// Stops the scan under way and keeps the previous result, if any.
    pub(crate) fn cancel_scan(&mut self) {
        self.scan.cancel();
    }

    /// Picks up the result of a finished scan.
    pub(crate) fn poll(&mut self) {
        if let Some(report) = self.scan.poll() {
            self.accept(report);
        }
    }

    fn accept(&mut self, report: ScanReport) {
        self.report = Some(report);
        self.trail.clear();
        self.zoom = None;
    }

    /// Totals of the scan under way, when there is one.
    pub(crate) fn progress(&self) -> Option<ScanProgress> {
        self.scan.progress()
    }

    pub(crate) fn report(&self) -> Option<&ScanReport> {
        self.report.as_ref()
    }

    pub(crate) fn failure(&self) -> Option<&str> {
        self.scan.failure()
    }

    pub(crate) fn zoom(&self) -> Option<Zoom> {
        self.zoom
    }

    /// The folder whose content is on screen.
    fn current(&self) -> NodeId {
        self.trail.last().copied().unwrap_or(FileTree::ROOT)
    }

    /// Names from the scanned folder down to the one on screen.
    pub(crate) fn crumbs(&self) -> Vec<String> {
        let Some(report) = &self.report else {
            return Vec::new();
        };
        std::iter::once(FileTree::ROOT)
            .chain(self.trail.iter().copied())
            .filter_map(|id| report.tree.node(id))
            .map(|node| node.name().to_owned())
            .collect()
    }

    /// Goes back to the folder at `depth` in the trail; 0 is the scanned folder.
    pub(crate) fn go_to_depth(&mut self, depth: usize) {
        self.trail.truncate(depth);
        self.zoom = None;
    }

    /// Shows what is inside `node`, growing the map out of `origin`.
    pub(crate) fn enter(&mut self, node: NodeId, origin: LayoutRect, now: f64) {
        self.trail.push(node);
        self.zoom = Some(Zoom {
            origin,
            timeline: Timeline::start(now),
        });
    }

    /// Absolute path of a node of the current scan.
    pub(crate) fn path_of(&self, node: NodeId) -> Option<PathBuf> {
        let report = self.report.as_ref()?;
        Some(report.tree.path_segments(node).into_iter().collect())
    }

    /// The blocks of the folder on screen, largest first.
    pub(crate) fn blocks(&self) -> Vec<MapBlock> {
        let Some(report) = &self.report else {
            return Vec::new();
        };
        let tree = &report.tree;
        let mut blocks: Vec<MapBlock> = tree
            .children(self.current())
            .filter_map(|id| tree.node(id).map(|node| self.block(tree, id, node)))
            .filter(|block| block.weight > 0.0)
            .collect();
        blocks.sort_by(|a, b| b.weight.total_cmp(&a.weight));

        if blocks.len() > MAX_BLOCKS {
            let others = blocks.split_off(MAX_BLOCKS - 1);
            blocks.push(Self::merged(&others));
        }
        blocks
    }

    fn block(&self, tree: &FileTree, id: NodeId, node: &FileNode) -> MapBlock {
        let size = match self.color_mode {
            ColorMode::Category => node.allocated_size(),
            // Cloud-only content takes no disk space, so the cloud view sizes
            // blocks by content instead. That is what makes it visible.
            ColorMode::Cloud => node.logical_size(),
        };
        MapBlock {
            node: Some(id),
            name: node.name().to_owned(),
            weight: size.as_bytes() as f64,
            allocated: node.allocated_size(),
            logical: node.logical_size(),
            item_count: node.item_count(),
            category: FileCategory::of(node.name(), node.kind()),
            cloud: cloud_state_of(node),
            is_enterable: node.kind() == EntryKind::Directory && tree.children(id).next().is_some(),
        }
    }

    fn merged(blocks: &[MapBlock]) -> MapBlock {
        MapBlock {
            node: None,
            name: format!("{} ({})", pt_br::MAP_OTHERS, blocks.len()),
            weight: blocks.iter().map(|block| block.weight).sum(),
            allocated: blocks.iter().map(|block| block.allocated).sum(),
            logical: blocks.iter().map(|block| block.logical).sum(),
            item_count: blocks.iter().map(|block| block.item_count).sum(),
            category: FileCategory::Other,
            cloud: CloudState::Local,
            is_enterable: false,
        }
    }
}

/// Where the content of a node lives, counting everything beneath it.
fn cloud_state_of(node: &FileNode) -> CloudState {
    let cloud_only = node.cloud_only_size();
    if !cloud_only.is_zero() && cloud_only >= node.logical_size() {
        CloudState::CloudOnly
    } else if !cloud_only.is_zero() || node.cloud_state() == CloudState::Synced {
        CloudState::Synced
    } else {
        CloudState::Local
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use diskmarshal_application::use_cases::ScanReport;
    use diskmarshal_domain::scan::{CloudState, FileEntry, FileTree};
    use diskmarshal_domain::shared::ByteSize;
    use diskmarshal_domain::treemap::LayoutRect;

    use super::{ColorMode, DiskMapViewModel, MAX_BLOCKS};

    fn file(name: &str, bytes: u64) -> FileEntry {
        FileEntry::file(name, ByteSize::from_bytes(bytes))
    }

    fn model_with(tree: FileTree) -> DiskMapViewModel {
        let mut tree = tree;
        tree.aggregate();
        DiskMapViewModel {
            report: Some(ScanReport {
                root: PathBuf::from("/scan"),
                tree,
                inaccessible_paths: Vec::new(),
                elapsed: Duration::ZERO,
            }),
            ..DiskMapViewModel::default()
        }
    }

    #[test]
    fn lists_the_content_of_the_scanned_folder_largest_first() {
        let mut tree = FileTree::new(FileEntry::directory("/scan"));
        tree.add_child(FileTree::ROOT, file("small.txt", 10));
        let photos = tree
            .add_child(FileTree::ROOT, FileEntry::directory("photos"))
            .unwrap();
        tree.add_child(photos, file("a.heic", 500));
        tree.add_child(FileTree::ROOT, file("empty.txt", 0));

        let blocks = model_with(tree).blocks();

        let names: Vec<&str> = blocks.iter().map(|block| block.name.as_str()).collect();
        assert_eq!(names, ["photos", "small.txt"]);
        assert!(blocks[0].is_enterable);
        assert!(!blocks[1].is_enterable);
    }

    #[test]
    fn merges_the_smallest_items_beyond_the_limit() {
        let mut tree = FileTree::new(FileEntry::directory("/scan"));
        for index in 0..(MAX_BLOCKS + 10) {
            tree.add_child(
                FileTree::ROOT,
                file(&format!("f{index}"), 1_000 + index as u64),
            );
        }

        let blocks = model_with(tree).blocks();

        assert_eq!(blocks.len(), MAX_BLOCKS);
        let others = blocks.last().unwrap();
        assert_eq!(others.node, None);
        assert_eq!(others.item_count, 11);
        let total: f64 = blocks.iter().map(|block| block.weight).sum();
        let expected: u64 = (0..(MAX_BLOCKS + 10) as u64)
            .map(|index| 1_000 + index)
            .sum();
        assert!((total - expected as f64).abs() < 0.5);
    }

    #[test]
    fn the_cloud_view_shows_content_that_takes_no_disk_space() {
        let mut tree = FileTree::new(FileEntry::directory("/scan"));
        let mut placeholder = file("movie.mov", 9_000);
        placeholder.cloud_state = CloudState::CloudOnly;
        placeholder.allocated_size = ByteSize::ZERO;
        tree.add_child(FileTree::ROOT, placeholder);
        tree.add_child(FileTree::ROOT, file("local.txt", 100));
        let mut model = model_with(tree);

        assert_eq!(model.blocks().len(), 1);

        model.color_mode = ColorMode::Cloud;
        let blocks = model.blocks();
        assert_eq!(blocks[0].name, "movie.mov");
        assert_eq!(blocks[0].cloud, CloudState::CloudOnly);
        assert_eq!(blocks[1].cloud, CloudState::Local);
    }

    #[test]
    fn entering_a_folder_and_going_back_follow_the_trail() {
        let mut tree = FileTree::new(FileEntry::directory("/scan"));
        let docs = tree
            .add_child(FileTree::ROOT, FileEntry::directory("docs"))
            .unwrap();
        tree.add_child(docs, file("a.pdf", 10));
        let mut model = model_with(tree);

        model.enter(docs, LayoutRect::new(0.0, 0.0, 0.5, 0.5), 1.0);
        assert_eq!(model.crumbs(), ["/scan", "docs"]);
        assert_eq!(model.blocks()[0].name, "a.pdf");
        assert_eq!(model.path_of(docs), Some(PathBuf::from("/scan/docs")));
        assert!(model.zoom().is_some());

        model.go_to_depth(0);
        assert_eq!(model.crumbs(), ["/scan"]);
        assert!(model.zoom().is_none());
    }
}

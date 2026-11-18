//! Reviewing groups of files that stand for each other: what is marked for
//! removal, under the rule that one file of every group always stays.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use egui::Context;
use maccleaner_domain::deletion::DeletionRequest;
use maccleaner_domain::shared::ByteSize;

use crate::app::UiServices;
use crate::components::RemovalFlow;
use crate::i18n::pt_br;

/// The groups under review and what the user marked in them.
pub(crate) struct GroupReview {
    /// The removal of every file, group by group, the one to keep first.
    groups: Vec<Vec<DeletionRequest>>,
    marked: BTreeSet<PathBuf>,
    notice: Option<&'static str>,
    pub removal: RemovalFlow,
}

impl Default for GroupReview {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl GroupReview {
    /// Starts a review with every file but the first of each group marked.
    pub(crate) fn new(groups: Vec<Vec<DeletionRequest>>) -> Self {
        let marked = groups
            .iter()
            .flat_map(|group| group.iter().skip(1))
            .map(|request| request.target.clone())
            .collect();
        Self {
            groups,
            marked,
            notice: None,
            removal: RemovalFlow::Closed,
        }
    }

    pub(crate) fn is_marked(&self, path: &Path) -> bool {
        self.marked.contains(path)
    }

    /// Marks or unmarks a file. Marking the last unmarked file of a group is
    /// refused, and `refusal` becomes the notice that explains why.
    pub(crate) fn toggle(&mut self, path: &Path, refusal: &'static str) {
        self.notice = None;
        if self.marked.remove(path) {
            return;
        }
        let would_leave_none = self
            .groups
            .iter()
            .find(|group| group.iter().any(|request| request.target == path))
            .is_some_and(|group| {
                group
                    .iter()
                    .all(|request| request.target == path || self.is_marked(&request.target))
            });
        if would_leave_none {
            self.notice = Some(refusal);
        } else {
            self.marked.insert(path.to_path_buf());
        }
    }

    /// How many files are marked and how much space they use.
    pub(crate) fn selection(&self) -> (usize, ByteSize) {
        let sizes: Vec<ByteSize> = self
            .groups
            .iter()
            .flatten()
            .filter(|request| self.is_marked(&request.target))
            .map(|request| request.reclaimable)
            .collect();
        (sizes.len(), sizes.into_iter().sum())
    }

    /// What removing every file but the one to keep of each group would free.
    pub(crate) fn recoverable(&self) -> ByteSize {
        self.groups
            .iter()
            .flat_map(|group| group.iter().skip(1))
            .map(|request| request.reclaimable)
            .sum()
    }

    /// What to tell the user about the last thing they tried.
    pub(crate) fn notice(&self) -> Option<&'static str> {
        self.notice
    }

    /// Opens the confirmation, or explains why there is nothing to confirm.
    pub(crate) fn request_removal(&mut self) {
        if self.selection().0 == 0 {
            self.notice = Some(pt_br::SELECT_SOMETHING);
        } else {
            self.removal.open();
        }
    }

    /// Removes the marked files in the background. `root` is the folder the
    /// groups were found in; nothing outside it can be removed.
    pub(crate) fn confirm(&mut self, ctx: &Context, services: &UiServices, root: PathBuf) {
        let requests = self
            .groups
            .iter()
            .flatten()
            .filter(|request| self.is_marked(&request.target))
            .cloned()
            .collect();
        self.removal.start(ctx, services, requests, vec![root]);
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use maccleaner_domain::deletion::{DeletionMethod, DeletionRequest, ItemFingerprint};
    use maccleaner_domain::shared::ByteSize;

    use super::GroupReview;

    const REFUSAL: &str = "one stays";

    fn group(paths: &[&str], size: u64) -> Vec<DeletionRequest> {
        paths
            .iter()
            .map(|path| DeletionRequest {
                target: PathBuf::from(path),
                method: DeletionMethod::Trash,
                fingerprint: ItemFingerprint {
                    device: 1,
                    inode: 1,
                    size,
                    modified_unix_seconds: 0,
                },
                reclaimable: ByteSize::from_bytes(size),
            })
            .collect()
    }

    fn review() -> GroupReview {
        GroupReview::new(vec![
            group(&["/d/a.mov", "/d/x/a-copy.mov", "/d/y/a-2.mov"], 500),
            group(&["/d/b.zip", "/d/x/b.zip"], 200),
        ])
    }

    #[test]
    fn marks_every_file_but_the_one_to_keep() {
        let review = review();
        assert!(!review.is_marked(Path::new("/d/a.mov")));
        assert!(review.is_marked(Path::new("/d/x/a-copy.mov")));
        assert!(review.is_marked(Path::new("/d/y/a-2.mov")));
        assert_eq!(review.selection(), (3, ByteSize::from_bytes(1_200)));
        assert_eq!(review.recoverable(), ByteSize::from_bytes(1_200));
    }

    #[test]
    fn never_lets_every_file_of_a_group_be_marked() {
        let mut review = review();

        review.toggle(Path::new("/d/b.zip"), REFUSAL);
        assert!(
            !review.is_marked(Path::new("/d/b.zip")),
            "it is the last one left"
        );
        assert_eq!(review.notice(), Some(REFUSAL));

        review.toggle(Path::new("/d/x/b.zip"), REFUSAL);
        review.toggle(Path::new("/d/b.zip"), REFUSAL);
        assert!(
            review.is_marked(Path::new("/d/b.zip")),
            "the other one now stays"
        );
        assert!(!review.is_marked(Path::new("/d/x/b.zip")));
        assert!(review.notice().is_none());
    }

    #[test]
    fn asking_to_remove_nothing_only_explains() {
        let mut review = GroupReview::default();
        review.request_removal();
        assert!(review.notice().is_some());
    }
}

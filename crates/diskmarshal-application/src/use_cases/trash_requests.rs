//! Removals that send personal files to the Trash.

use std::path::Path;

use diskmarshal_domain::deletion::{DeletionMethod, DeletionRequest};
use diskmarshal_domain::shared::ByteSize;

use crate::ports::FileInspector;

/// Builds one removal to the Trash per file, in the same order.
///
/// Returns `None` when any of the files can no longer be identified: a group
/// is offered whole or not at all.
pub(crate) fn trash_requests<'a>(
    inspector: &dyn FileInspector,
    files: impl IntoIterator<Item = (&'a Path, ByteSize)>,
) -> Option<Vec<DeletionRequest>> {
    files
        .into_iter()
        .map(|(path, reclaimable)| {
            Some(DeletionRequest {
                target: path.to_path_buf(),
                method: DeletionMethod::Trash,
                fingerprint: inspector.fingerprint(path)?,
                reclaimable,
            })
        })
        .collect()
}

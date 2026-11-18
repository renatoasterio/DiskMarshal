//! Disk map: a treemap of what uses the disk, by kind of content or by iCloud state.

mod view;
mod view_model;

pub(crate) use view::show;
pub(crate) use view_model::DiskMapViewModel;

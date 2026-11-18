//! Cleanup: caches, temporary files and logs, removed after a preview.

mod view;
mod view_model;

pub(crate) use view::show;
pub(crate) use view_model::CleanupViewModel;

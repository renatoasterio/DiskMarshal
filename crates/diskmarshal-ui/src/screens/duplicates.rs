//! Duplicates: files with identical content, where one copy always stays.

mod view;
mod view_model;

pub(crate) use view::show;
pub(crate) use view_model::DuplicatesViewModel;

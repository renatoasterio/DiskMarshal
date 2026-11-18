//! Monitor: live processor, memory, disk and temperature readings.

mod view;
mod view_model;

pub(crate) use view::show;
pub(crate) use view_model::MonitorViewModel;

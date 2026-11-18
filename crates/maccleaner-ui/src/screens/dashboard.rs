//! Dashboard: the state of the machine and what can be recovered, at a glance.

mod view;
mod view_model;

pub(crate) use view::show;
pub(crate) use view_model::{DashboardAction, DashboardViewModel, Recoverable};

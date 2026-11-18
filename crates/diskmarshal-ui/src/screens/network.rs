//! Network: who is using the connection, how good it is, and what can be
//! changed to make it faster.

mod analysis_tab;
mod traffic_tab;
mod tuning_tab;
mod view;
mod view_model;

pub(crate) use view::show;
pub(crate) use view_model::NetworkViewModel;

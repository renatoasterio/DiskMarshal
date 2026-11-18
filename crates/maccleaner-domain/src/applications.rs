//! Installed apps and what they leave behind: the model of the uninstaller.

mod installed_app;
mod leftover_kind;
mod leftover_location;
mod leftover_matcher;
mod uninstall_plan;

pub use installed_app::InstalledApp;
pub use leftover_kind::LeftoverKind;
pub use leftover_location::LeftoverLocation;
pub use leftover_matcher::LeftoverMatcher;
pub use uninstall_plan::{UninstallItem, UninstallPlan};

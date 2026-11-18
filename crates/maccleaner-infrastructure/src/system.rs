//! Adapters for facts about the running system.

mod accessibility_settings;
mod localtime_wall_clock;
mod native_folder_picker;
mod open_command_opener;
mod sysinfo_running_app_probe;

pub use accessibility_settings::prefers_reduced_motion;
pub use localtime_wall_clock::LocaltimeWallClock;
pub use native_folder_picker::NativeFolderPicker;
pub use open_command_opener::OpenCommandOpener;
pub use sysinfo_running_app_probe::SysinfoRunningAppProbe;

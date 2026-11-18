//! Working with administrator rights: the app asks macOS to authenticate
//! the user and to run a small helper once; the two talk in a fixed format.

mod helper_launcher;
mod helper_program;
mod helper_protocol;
mod macos_network_tuner;
mod macos_privileged_cleaner;
mod network_request;
mod osascript_launch;
mod sudo_launch;

pub use helper_protocol::{
    DRY_RUN_FLAG, HELPER_NAME, HelperRequest, HelperTask, UNINSTALL_FLAG, decode_outcomes,
    encode_outcomes,
};
pub use macos_network_tuner::MacosNetworkTuner;
pub use macos_privileged_cleaner::MacosPrivilegedCleaner;
pub use network_request::NETWORK_FLAG;

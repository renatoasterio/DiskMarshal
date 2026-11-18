//! Cleaning with administrator rights: the app asks macOS to authenticate
//! the user and to run a small helper once; the two talk in a fixed format.

mod helper_program;
mod helper_protocol;
mod macos_privileged_cleaner;
mod osascript_launch;
mod sudo_launch;

pub use helper_protocol::{
    DRY_RUN_FLAG, HELPER_NAME, HelperRequest, HelperTask, UNINSTALL_FLAG, decode_outcomes,
    encode_outcomes,
};
pub use macos_privileged_cleaner::MacosPrivilegedCleaner;

//! Cleaning with administrator rights: the app asks macOS to authenticate
//! the user and to run a small helper once; the two talk in a fixed format.

mod helper_protocol;
mod osascript_privileged_cleaner;

pub use helper_protocol::{
    DRY_RUN_FLAG, HELPER_NAME, HelperRequest, decode_outcomes, encode_outcomes,
};
pub use osascript_privileged_cleaner::OsascriptPrivilegedCleaner;

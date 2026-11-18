//! Process-wide policy that keeps a scan from downloading iCloud files.

use std::io;

use libc::c_int;

// Values from <sys/resource.h> in the macOS SDK.
const IOPOL_TYPE_VFS_MATERIALIZE_DATALESS_FILES: c_int = 3;
const IOPOL_SCOPE_PROCESS: c_int = 0;
const IOPOL_MATERIALIZE_DATALESS_FILES_OFF: c_int = 1;

unsafe extern "C" {
    fn setiopolicy_np(iotype: c_int, scope: c_int, policy: c_int) -> c_int;
}

/// Forbids this process from materializing dataless (cloud-only) items.
///
/// macOS normally downloads a cloud-only file or folder the moment a program
/// reads it. After this call, such an access fails with `EDEADLK` instead, so
/// a scan can never start a download. Call it first thing in `main`, before
/// any file is touched.
///
/// # Errors
///
/// Fails when the kernel rejects the policy. A caller must not scan in that
/// case, because nothing would stop downloads from starting.
pub fn disable_dataless_materialization() -> io::Result<()> {
    // SAFETY: `setiopolicy_np` takes three plain integers and only changes a
    // per-process kernel flag. It has no memory-safety preconditions.
    let status = unsafe {
        setiopolicy_np(
            IOPOL_TYPE_VFS_MATERIALIZE_DATALESS_FILES,
            IOPOL_SCOPE_PROCESS,
            IOPOL_MATERIALIZE_DATALESS_FILES_OFF,
        )
    };
    if status == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(test)]
mod tests {
    use super::disable_dataless_materialization;

    #[test]
    fn the_kernel_accepts_the_policy() {
        disable_dataless_materialization().unwrap();
    }
}

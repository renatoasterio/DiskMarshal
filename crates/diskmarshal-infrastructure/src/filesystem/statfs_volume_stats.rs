//! Volume usage read with `statfs`.

use std::ffi::CString;
use std::io;
use std::mem::MaybeUninit;
use std::os::unix::ffi::OsStrExt as _;
use std::path::Path;

use diskmarshal_application::ApplicationError;
use diskmarshal_application::ports::VolumeStats;
use diskmarshal_domain::monitoring::VolumeUsage;
use diskmarshal_domain::shared::ByteSize;

/// Reads capacity and free space of a volume with `statfs`.
#[derive(Debug, Clone, Copy, Default)]
pub struct StatfsVolumeStats;

impl VolumeStats for StatfsVolumeStats {
    fn usage(&self, path: &Path) -> Result<VolumeUsage, ApplicationError> {
        read(path).map_err(|source| ApplicationError::Io {
            path: path.to_path_buf(),
            source,
        })
    }
}

fn read(path: &Path) -> io::Result<VolumeUsage> {
    let c_path = CString::new(path.as_os_str().as_bytes())
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    let mut stats = MaybeUninit::<libc::statfs>::uninit();
    // SAFETY: `c_path` is a valid NUL-terminated string that outlives the call,
    // and `stats` points to writable memory of exactly the size `statfs` fills.
    let status = unsafe { libc::statfs(c_path.as_ptr(), stats.as_mut_ptr()) };
    if status != 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `statfs` returned 0, which means it initialized the whole struct.
    let stats = unsafe { stats.assume_init() };

    let block_bytes = u64::from(stats.f_bsize);
    Ok(VolumeUsage {
        total: ByteSize::from_bytes(stats.f_blocks.saturating_mul(block_bytes)),
        available: ByteSize::from_bytes(stats.f_bavail.saturating_mul(block_bytes)),
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use diskmarshal_application::ports::VolumeStats;

    use super::StatfsVolumeStats;

    #[test]
    fn reads_the_startup_volume() {
        let usage = StatfsVolumeStats.usage(Path::new("/")).unwrap();
        assert!(!usage.total.is_zero());
        assert!(usage.available <= usage.total);
    }

    #[test]
    fn fails_for_a_path_that_does_not_exist() {
        assert!(
            StatfsVolumeStats
                .usage(Path::new("/definitely/not/here"))
                .is_err()
        );
    }
}

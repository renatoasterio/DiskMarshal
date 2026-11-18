//! Running-app probe built on the `sysinfo` crate.

use std::path::{Path, PathBuf};

use maccleaner_application::ports::RunningAppProbe;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

/// Lists the running programs with `sysinfo`, reading nothing but where
/// each one was started from.
#[derive(Debug, Clone, Copy, Default)]
pub struct SysinfoRunningAppProbe;

impl RunningAppProbe for SysinfoRunningAppProbe {
    fn running_executables(&self) -> Vec<PathBuf> {
        // When the system cannot be asked, no program is reported as running.
        let Ok(mut system) = System::new() else {
            return Vec::new();
        };
        system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet),
        );
        system
            .processes()
            .values()
            .filter_map(|process| process.exe().map(Path::to_path_buf))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use maccleaner_application::ports::RunningAppProbe;

    use super::SysinfoRunningAppProbe;

    #[test]
    fn sees_the_program_that_is_asking() {
        let this_program = env::current_exe().unwrap().canonicalize().unwrap();

        let running = SysinfoRunningAppProbe.running_executables();

        assert!(
            running
                .iter()
                .filter_map(|path| path.canonicalize().ok())
                .any(|path| path == this_program)
        );
    }
}

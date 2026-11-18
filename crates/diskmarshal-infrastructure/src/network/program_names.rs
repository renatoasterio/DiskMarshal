//! Finding out which program each process belongs to.

use std::collections::HashMap;

use diskmarshal_domain::network::program_name;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

/// Remembers the program of every process seen, so the system is asked
/// about a process only once.
pub(crate) struct ProgramNames {
    /// `None` when the system cannot be asked: names then stay as given.
    system: Option<System>,
    known: HashMap<u32, String>,
}

impl ProgramNames {
    pub(crate) fn new() -> Self {
        Self {
            system: System::new().ok(),
            known: HashMap::new(),
        }
    }

    /// Looks up the processes of `running` that were not seen before and
    /// forgets the ones that are gone. Each entry is a process identifier
    /// and the name it was listed with, which is the fallback.
    pub(crate) fn refresh(&mut self, running: &[(u32, &str)]) {
        self.known
            .retain(|pid, _| running.iter().any(|(listed, _)| listed == pid));
        let unknown: Vec<(u32, &str)> = running
            .iter()
            .filter(|(pid, _)| !self.known.contains_key(pid))
            .copied()
            .collect();
        if unknown.is_empty() {
            return;
        }
        if let Some(system) = &mut self.system {
            let pids: Vec<Pid> = unknown.iter().map(|(pid, _)| Pid::from_u32(*pid)).collect();
            system.refresh_processes_specifics(
                ProcessesToUpdate::Some(&pids),
                true,
                ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet),
            );
        }
        for (pid, listed_name) in unknown {
            let process = self
                .system
                .as_ref()
                .and_then(|system| system.process(Pid::from_u32(pid)));
            let name = match process {
                Some(process) => program_name(process.exe(), &process.name().to_string_lossy()),
                None => listed_name.to_owned(),
            };
            self.known.insert(pid, name);
        }
    }

    /// The program of a process that [`refresh`](Self::refresh) was told about.
    pub(crate) fn of(&self, pid: u32) -> Option<&str> {
        self.known.get(&pid).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use std::process;

    use super::ProgramNames;

    #[test]
    fn names_this_process_after_its_executable_and_forgets_it_when_gone() {
        let mut names = ProgramNames::new();
        let this = process::id();

        names.refresh(&[(this, "cut-short")]);
        let name = names.of(this).unwrap().to_owned();
        assert!(!name.is_empty());
        assert_ne!(name, "cut-short", "the full name comes from the system");

        names.refresh(&[]);
        assert_eq!(names.of(this), None);
    }

    #[test]
    fn a_process_the_system_does_not_know_keeps_the_name_it_was_listed_with() {
        let mut names = ProgramNames::new();
        // No process has this identifier: macOS stops far below it.
        let nobody = u32::MAX - 1;

        names.refresh(&[(nobody, "listed")]);

        assert_eq!(names.of(nobody), Some("listed"));
    }
}

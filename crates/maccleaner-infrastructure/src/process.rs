//! Adapters that run other programs. Every program is named by an absolute
//! path and receives its arguments as a list, never through a shell.

mod git_version_control_probe;

pub use git_version_control_probe::GitVersionControlProbe;

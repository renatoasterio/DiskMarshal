//! Adapters that run other programs. Every program is named by an absolute
//! path and receives its arguments as a list, never through a shell.

mod bounded_command;
mod docker_cli_container_engine;
mod git_version_control_probe;

pub use docker_cli_container_engine::DockerCliContainerEngine;
pub use git_version_control_probe::GitVersionControlProbe;

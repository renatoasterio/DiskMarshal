//! One module per subcommand.

pub mod apps;
pub mod docker;
pub mod scan;

use std::env;
use std::path::PathBuf;

use anyhow::Context as _;

/// The home folder of the user running the command.
fn home_folder() -> anyhow::Result<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .context("a pasta pessoal é desconhecida: a variável HOME não está definida")
}

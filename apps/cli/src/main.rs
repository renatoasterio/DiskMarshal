//! Headless MacCleaner. Composition root for the command line: wires the macOS
//! adapters into the use cases and prints the result. It never deletes anything.

#![forbid(unsafe_code)]

mod commands;

use std::process::ExitCode;

use anyhow::Context as _;
use clap::{Parser, Subcommand};
use maccleaner_infrastructure::filesystem::disable_dataless_materialization;

#[derive(Parser)]
#[command(
    name = "maccleaner-cli",
    version,
    about = "MacCleaner em linha de comando (somente leitura)"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Mede uma pasta e lista os maiores itens
    Scan(commands::scan::ScanArguments),
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("erro: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    // Before any file is touched: without this, reading a cloud-only item
    // would make macOS download it.
    disable_dataless_materialization()
        .context("não foi possível impedir downloads do iCloud; nada foi lido")?;
    match cli.command {
        Command::Scan(arguments) => commands::scan::run(&arguments),
    }
}

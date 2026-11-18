//! Privileged helper of MacCleaner. Composition root for the one thing that
//! runs as an administrator: cleaning the system areas of the rule catalog.
//!
//! macOS starts it after authenticating the user, it cleans once, prints
//! what happened as one line of JSON and exits. It accepts rule ids and
//! nothing else: the folders come from the catalog built into this program,
//! and every removal goes through the same deletion policy as the app.

#![forbid(unsafe_code)]

mod unused_ports;

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use maccleaner_application::ports::WallClock;
use maccleaner_application::use_cases::{
    CleanRules, DeleteItems, DeletionPorts, ExecutionMode, PlanCleanup,
};
use maccleaner_domain::cleanup::PrivilegeScope;
use maccleaner_domain::deletion::ProtectedPaths;
use maccleaner_infrastructure::catalog::TomlRuleCatalog;
use maccleaner_infrastructure::filesystem::{
    LstatFileInspector, ReaddirWalker, SafeFileRemover, disable_dataless_materialization,
};
use maccleaner_infrastructure::privilege::{HelperRequest, encode_outcomes};
use maccleaner_infrastructure::system::LocaltimeWallClock;

use crate::unused_ports::{NoHistory, NoTrashBin};

const PROGRAM: &str = env!("CARGO_PKG_NAME");
/// Home folder of the administrator account. Administrator rules name
/// absolute folders, so this only anchors the deletion policy.
const ADMINISTRATOR_HOME: &str = "/var/root";

fn main() -> ExitCode {
    match run() {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(reason) => {
            eprintln!("{PROGRAM}: {reason}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<String, String> {
    // Before any file is touched: without this, reading a cloud-only item
    // would make macOS download it.
    disable_dataless_materialization().map_err(|error| error.to_string())?;
    let request = HelperRequest::parse(env::args().skip(1))?;

    let home = PathBuf::from(ADMINISTRATOR_HOME);
    let catalog = TomlRuleCatalog::new(home.clone(), None);
    let (walker, inspector) = (ReaddirWalker::default(), LstatFileInspector);
    let policy = ProtectedPaths::new(&home);
    let ports = DeletionPorts {
        inspector: &inspector,
        remover: &SafeFileRemover,
        trash_bin: &NoTrashBin,
        history: &NoHistory,
    };
    let mode = if request.is_dry_run {
        ExecutionMode::DryRun
    } else {
        ExecutionMode::Real
    };

    let report = CleanRules::new(
        &catalog,
        PlanCleanup::new(&catalog, &walker, &inspector),
        DeleteItems::new(&policy, ports),
    )
    .execute(
        &request.rule_ids,
        PrivilegeScope::Administrator,
        LocaltimeWallClock.unix_seconds(),
        mode,
    )
    .map_err(|error| error.to_string())?;
    Ok(encode_outcomes(&report.outcomes))
}

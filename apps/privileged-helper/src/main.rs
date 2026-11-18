//! Privileged helper of MacCleaner. Composition root for the two things that
//! run as an administrator: cleaning the system areas of the rule catalog,
//! and removing the system items of an app that is being uninstalled.
//!
//! macOS starts it after authenticating the user, it works once, prints what
//! happened as one line of JSON and exits. A cleanup names rule ids and
//! nothing else: the folders come from the catalog built into this program.
//! An uninstall names an app by its identifier and the items to remove, and
//! only an item this program finds for that app by itself is removed. Every
//! removal goes through the same deletion policy as the app.

#![forbid(unsafe_code)]

mod unused_ports;

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use maccleaner_application::ports::WallClock;
use maccleaner_application::use_cases::{
    CleanRules, DeleteItems, DeletionPorts, ExecutionMode, ListInstalledApps, PlanCleanup,
    PlanUninstall, RemoveAppItems,
};
use maccleaner_domain::cleanup::PrivilegeScope;
use maccleaner_domain::deletion::ProtectedPaths;
use maccleaner_infrastructure::bundles::{PlistAppBundleReader, application_folders};
use maccleaner_infrastructure::catalog::TomlRuleCatalog;
use maccleaner_infrastructure::filesystem::{
    LstatFileInspector, ReaddirFolderLister, ReaddirWalker, SafeFileRemover,
    disable_dataless_materialization,
};
use maccleaner_infrastructure::privilege::{HelperRequest, HelperTask, encode_outcomes};
use maccleaner_infrastructure::system::{LocaltimeWallClock, SysinfoRunningAppProbe};

use crate::unused_ports::{NoHistory, NoTrashBin};

const PROGRAM: &str = env!("CARGO_PKG_NAME");
/// Home folder of the administrator account. Administrator rules and
/// folders are absolute, so this only anchors the deletion policy and the
/// library that is never touched from here.
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
    let delete = DeleteItems::new(&policy, ports);
    let mode = if request.is_dry_run {
        ExecutionMode::DryRun
    } else {
        ExecutionMode::Real
    };

    let report = match &request.task {
        HelperTask::CleanRules(rule_ids) => CleanRules::new(
            &catalog,
            PlanCleanup::new(&catalog, &walker, &inspector),
            delete,
        )
        .execute(
            rule_ids,
            PrivilegeScope::Administrator,
            LocaltimeWallClock.unix_seconds(),
            mode,
        ),
        HelperTask::Uninstall { bundle_id, items } => RemoveAppItems::new(
            ListInstalledApps::new(
                &walker,
                &inspector,
                &PlistAppBundleReader::for_home(&home),
                &SysinfoRunningAppProbe,
            ),
            PlanUninstall::new(&catalog, &ReaddirFolderLister, &walker, &inspector),
            delete,
        )
        .execute(bundle_id, items, &application_folders(&home), mode),
    }
    .map_err(|error| error.to_string())?;
    Ok(encode_outcomes(&report.outcomes))
}

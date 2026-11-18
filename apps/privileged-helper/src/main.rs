//! Privileged helper of DiskMarshal. Composition root for the three things
//! that run as an administrator: cleaning the system areas of the rule
//! catalog, removing the system items of an app that is being uninstalled,
//! and changing network settings.
//!
//! macOS starts it after authenticating the user, it works once, prints what
//! happened and exits. A cleanup names rule ids and nothing else: the folders
//! come from the catalog built into this program. An uninstall names an app
//! by its identifier and the items to remove, and only an item this program
//! finds for that app by itself is removed. Every removal goes through the
//! same deletion policy as the app. A network request names what to change,
//! never a command: each change is one of three fixed forms, and a network
//! service is accepted only when macOS lists it.

#![forbid(unsafe_code)]

mod unused_ports;

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use diskmarshal_application::ports::WallClock;
use diskmarshal_application::use_cases::{
    ChangeNetworkSettings, CleanRules, DeleteItems, DeletionPorts, ExecutionMode,
    ListInstalledApps, PlanCleanup, PlanUninstall, RemoveAppItems,
};
use diskmarshal_domain::cleanup::PrivilegeScope;
use diskmarshal_domain::deletion::ProtectedPaths;
use diskmarshal_domain::network::NetworkChange;
use diskmarshal_infrastructure::bundles::{PlistAppBundleReader, application_folders};
use diskmarshal_infrastructure::catalog::TomlRuleCatalog;
use diskmarshal_infrastructure::filesystem::{
    LstatFileInspector, ReaddirFolderLister, ReaddirWalker, SafeFileRemover,
    disable_dataless_materialization,
};
use diskmarshal_infrastructure::network::NetworksetupNetworkSettings;
use diskmarshal_infrastructure::privilege::{HelperRequest, HelperTask, encode_outcomes};
use diskmarshal_infrastructure::system::{LocaltimeWallClock, SysinfoRunningAppProbe};

use crate::unused_ports::{NoHistory, NoTrashBin, UnchangedSettings};

const PROGRAM: &str = env!("CARGO_PKG_NAME");
/// Home folder of the administrator account. Administrator rules and
/// folders are absolute, so this only anchors the deletion policy and the
/// library that is never touched from here.
const ADMINISTRATOR_HOME: &str = "/var/root";
/// What is printed when every network change was made.
const NETWORK_CHANGED: &str = "ok";

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
    match &request.task {
        HelperTask::ChangeNetwork(changes) => change_network(changes, request.is_dry_run),
        HelperTask::CleanRules(_) | HelperTask::Uninstall { .. } => remove(&request),
    }
}

/// Changes network settings. A dry run checks the request against this Mac
/// and changes nothing.
fn change_network(changes: &[NetworkChange], is_dry_run: bool) -> Result<String, String> {
    let settings = NetworksetupNetworkSettings;
    let outcome = if is_dry_run {
        ChangeNetworkSettings::new(&UnchangedSettings(&settings)).execute(changes)
    } else {
        ChangeNetworkSettings::new(&settings).execute(changes)
    };
    outcome
        .map(|()| NETWORK_CHANGED.to_owned())
        .map_err(|error| error.to_string())
}

/// Cleans administrator rules or removes the system items of an app.
fn remove(request: &HelperRequest) -> Result<String, String> {
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
        // `run` sends network changes elsewhere; nothing is removed for one.
        HelperTask::ChangeNetwork(_) => return Err("not a removal".to_owned()),
    }
    .map_err(|error| error.to_string())?;
    Ok(encode_outcomes(&report.outcomes))
}

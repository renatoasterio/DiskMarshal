//! `diskmarshal-cli apps`: lists the installed apps, or shows what
//! uninstalling one of them would remove. Nothing is removed.

use std::path::Path;

use anyhow::Context as _;
use clap::Args;
use diskmarshal_application::progress::{CancellationToken, NoProgress};
use diskmarshal_application::use_cases::{ListInstalledApps, PlanUninstall};
use diskmarshal_domain::applications::{InstalledApp, LeftoverKind, UninstallPlan};
use diskmarshal_domain::cleanup::{PrivilegeScope, RiskLevel};
use diskmarshal_infrastructure::bundles::{PlistAppBundleReader, application_folders};
use diskmarshal_infrastructure::catalog::TomlRuleCatalog;
use diskmarshal_infrastructure::filesystem::{
    LstatFileInspector, ReaddirFolderLister, ReaddirWalker, icloud_sync_roots,
};
use diskmarshal_infrastructure::system::SysinfoRunningAppProbe;

use super::home_folder;

/// Arguments of the `apps` subcommand.
#[derive(Args)]
pub struct AppsArguments {
    /// Nome do app a examinar; sem ele, todos os apps são listados
    name: Option<String>,
}

/// Lists the apps, or previews the uninstall of the one named.
pub fn run(arguments: &AppsArguments) -> anyhow::Result<()> {
    let home = home_folder()?;
    let walker = ReaddirWalker::new(icloud_sync_roots());
    let inspector = LstatFileInspector;
    let installed = ListInstalledApps::new(
        &walker,
        &inspector,
        &PlistAppBundleReader::for_home(&home),
        &SysinfoRunningAppProbe,
    )
    .execute(
        &application_folders(&home),
        &NoProgress,
        &CancellationToken::new(),
    )
    .context("não foi possível listar os apps")?;

    let Some(name) = &arguments.name else {
        print_apps(&installed);
        return Ok(());
    };
    let app = installed
        .iter()
        .find(|app| app.name.eq_ignore_ascii_case(name))
        .with_context(|| format!("nenhum app instalado se chama {name}"))?;
    let catalog = TomlRuleCatalog::new(home.clone(), None);
    let plan = PlanUninstall::new(&catalog, &ReaddirFolderLister, &walker, &inspector)
        .execute(app, &installed, &NoProgress, &CancellationToken::new())
        .with_context(|| format!("não foi possível examinar {}", app.name))?;
    print_plan(&plan, &home);
    Ok(())
}

fn print_apps(installed: &[InstalledApp]) {
    println!("{:>10}  {:<14}  {:<8}  App", "Tamanho", "Versão", "Remover");
    for app in installed {
        let version = app.version.as_deref().unwrap_or("-");
        let open = if app.is_running { "  (aberto)" } else { "" };
        println!(
            "{:>10}  {:<14}  {:<8}  {}{open}",
            app.size.to_string(),
            version,
            scope_label(app.scope),
            app.name
        );
    }
    let total = installed
        .iter()
        .map(|app| app.size)
        .sum::<diskmarshal_domain::shared::ByteSize>();
    println!("\n{} apps · {total}", installed.len());
}

fn print_plan(plan: &UninstallPlan, home: &Path) {
    let app = &plan.app;
    println!("App            {}", app.name);
    println!("Identificador  {}", app.bundle_id);
    println!("Versão         {}", app.version.as_deref().unwrap_or("-"));
    if app.is_running {
        println!("Situação       aberto agora: feche antes de desinstalar");
    }
    if plan.has_other_copy {
        println!("Outra cópia    há outra cópia instalada: os dados ficam com ela");
    }
    println!(
        "\n{:>10}  {:<13}  {:<7}  {:<8}  Item",
        "Tamanho", "Tipo", "Risco", "Remover"
    );
    for item in &plan.items {
        let target = &item.request.target;
        let shown = target.strip_prefix(home).map_or_else(
            |_| target.display().to_string(),
            |inside| format!("~/{}", inside.display()),
        );
        println!(
            "{:>10}  {:<13}  {:<7}  {:<8}  {shown}",
            item.request.reclaimable.to_string(),
            kind_label(item.kind),
            risk_label(item.risk),
            scope_label(item.scope),
        );
    }
    println!("\n{} itens · {}", plan.items.len(), plan.reclaimable());
    println!("Nada foi removido: este comando só mostra a prévia.");
}

const fn scope_label(scope: PrivilegeScope) -> &'static str {
    match scope {
        PrivilegeScope::User => "usuário",
        PrivilegeScope::Administrator => "admin",
    }
}

const fn risk_label(risk: RiskLevel) -> &'static str {
    match risk {
        RiskLevel::Safe => "seguro",
        RiskLevel::Review => "revisar",
    }
}

const fn kind_label(kind: LeftoverKind) -> &'static str {
    match kind {
        LeftoverKind::Application => "aplicativo",
        LeftoverKind::Support => "dados do app",
        LeftoverKind::Caches => "caches",
        LeftoverKind::Preferences => "preferências",
        LeftoverKind::Container => "contêiner",
        LeftoverKind::SavedState => "estado salvo",
        LeftoverKind::WebData => "dados web",
        LeftoverKind::Logs => "logs",
        LeftoverKind::LaunchItem => "serviço",
        LeftoverKind::Helper => "auxiliar",
    }
}

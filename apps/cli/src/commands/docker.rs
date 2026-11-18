//! `diskmarshal-cli docker`: shows what Docker holds and what of it nothing
//! uses. Nothing is removed.

use anyhow::anyhow;
use diskmarshal_application::ports::EngineError;
use diskmarshal_application::use_cases::InspectDocker;
use diskmarshal_domain::cleanup::RiskLevel;
use diskmarshal_domain::docker::{DockerInventory, ResourceKind};
use diskmarshal_domain::shared::ByteSize;
use diskmarshal_infrastructure::process::DockerCliContainerEngine;

use super::home_folder;

/// Asks Docker and prints a summary.
pub fn run() -> anyhow::Result<()> {
    let engine = DockerCliContainerEngine::for_home(&home_folder()?);
    match InspectDocker::new(&engine).execute() {
        Ok(inventory) => print_inventory(&inventory),
        Err(EngineError::NotInstalled) => {
            println!("Docker não encontrado neste Mac. Não há o que limpar.");
        }
        Err(EngineError::NotRunning) => {
            println!("O Docker está instalado, mas não está em execução.");
            println!("Abra o Docker, espere ele iniciar e rode o comando de novo.");
        }
        Err(error) => return Err(anyhow!("o Docker não respondeu como esperado: {error}")),
    }
    Ok(())
}

fn print_inventory(inventory: &DockerInventory) {
    println!("Docker {} · em execução\n", inventory.engine_version);
    println!(
        "{:<16}  {:>10}  {:>10}  {:>7}  Em uso",
        "Tipo", "Ocupado", "Sem uso", "Total"
    );
    for usage in &inventory.usage {
        println!(
            "{:<16}  {:>10}  {:>10}  {:>7}  {}",
            kind_label(usage.kind),
            usage.size.to_string(),
            usage.reclaimable.to_string(),
            usage.total_count,
            usage.active_count
        );
    }
    println!("{:<16}  {:>10}", "Tudo", inventory.size().to_string());

    println!("\n{:>10}  {:<16}  {:<7}  Nome", "Tamanho", "Tipo", "Risco");
    for resource in &inventory.unused {
        let name = match resource.kind {
            ResourceKind::BuildCache => "tudo o que os builds guardaram",
            _ if resource.name.is_empty() => "(sem nome)",
            _ => resource.name.as_str(),
        };
        let risk = match resource.risk() {
            RiskLevel::Safe => "seguro",
            RiskLevel::Review => "revisar",
        };
        println!(
            "{:>10}  {:<16}  {:<7}  {name}",
            resource.size.to_string(),
            kind_label(resource.kind),
            risk
        );
    }
    let listed: ByteSize = inventory.unused.iter().map(|resource| resource.size).sum();
    println!("\n{} itens sem uso · até {listed}", inventory.unused.len());
    println!("Nada foi removido: este comando só mostra a prévia.");
}

const fn kind_label(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::BuildCache => "cache de build",
        ResourceKind::Image => "imagem",
        ResourceKind::Container => "container",
        ResourceKind::Volume => "volume",
    }
}

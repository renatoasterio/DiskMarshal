//! `maccleaner-cli scan`: measures a folder and lists its largest items.

use std::path::{self, PathBuf};

use anyhow::Context as _;
use clap::Args;
use maccleaner_application::progress::{CancellationToken, NoProgress};
use maccleaner_application::use_cases::{ScanDisk, ScanReport};
use maccleaner_domain::scan::{EntryKind, FileTree};
use maccleaner_infrastructure::filesystem::{ReaddirWalker, icloud_sync_roots};

/// Arguments of the `scan` subcommand.
#[derive(Args)]
pub struct ScanArguments {
    /// Pasta a medir
    path: PathBuf,

    /// Quantos itens listar
    #[arg(long, default_value_t = 15)]
    top: usize,
}

/// Scans the folder and prints a summary.
pub fn run(arguments: &ScanArguments) -> anyhow::Result<()> {
    let root = path::absolute(&arguments.path)
        .with_context(|| format!("caminho inválido: {}", arguments.path.display()))?;
    let walker = ReaddirWalker::new(icloud_sync_roots());
    let report = ScanDisk::new(&walker)
        .execute(&root, &NoProgress, &CancellationToken::new())
        .with_context(|| format!("não foi possível varrer {}", root.display()))?;
    print_report(&report, arguments.top);
    Ok(())
}

fn print_report(report: &ScanReport, top: usize) {
    let Some(root) = report.tree.root() else {
        return;
    };
    println!("Pasta              {}", report.root.display());
    println!("Itens              {}", group_digits(root.item_count()));
    println!("Ocupado em disco   {}", root.allocated_size());
    println!("Tamanho lógico     {}", root.logical_size());
    println!("Somente na nuvem   {}", root.cloud_only_size());
    println!(
        "Pastas sem acesso  {}",
        group_digits(report.inaccessible_paths.len() as u64)
    );
    let seconds = format!("{:.1}", report.elapsed.as_secs_f64()).replace('.', ",");
    println!("Tempo              {seconds} s");

    println!("\nMaiores itens");
    for id in report.tree.largest_children(FileTree::ROOT, top) {
        let Some(node) = report.tree.node(id) else {
            continue;
        };
        let marker = match node.kind() {
            EntryKind::Directory => "/",
            EntryKind::Package => " (pacote)",
            EntryKind::File | EntryKind::Symlink | EntryKind::Other => "",
        };
        println!(
            "{:>12}  {}{marker}",
            node.allocated_size().to_string(),
            node.name()
        );
    }
}

/// Formats an integer with the pt-BR thousands separator: `4.712.330`.
fn group_digits(value: u64) -> String {
    let digits = value.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push('.');
        }
        grouped.push(digit);
    }
    grouped
}

#[cfg(test)]
mod tests {
    use super::group_digits;

    #[test]
    fn groups_digits_in_threes() {
        assert_eq!(group_digits(0), "0");
        assert_eq!(group_digits(999), "999");
        assert_eq!(group_digits(1_000), "1.000");
        assert_eq!(group_digits(4_712_330), "4.712.330");
    }
}

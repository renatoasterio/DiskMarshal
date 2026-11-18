//! `diskmarshal-cli network`: shows which programs are using the network and
//! how fast. Nothing is captured and nothing is changed.

use std::thread;
use std::time::Duration;

use anyhow::Context as _;
use clap::Args;
use diskmarshal_application::use_cases::MeasureTraffic;
use diskmarshal_domain::network::{Connection, ProgramTraffic, TrafficMeter, TrafficSnapshot};
use diskmarshal_infrastructure::network::NettopTrafficProbe;

const UNREADABLE: &str = "não foi possível ler o tráfego de rede";
/// How wide a program name may be in the table.
const NAME_WIDTH: usize = 26;

/// Arguments of the `network` subcommand.
#[derive(Args)]
pub struct NetworkArguments {
    /// Segundos entre as duas leituras que medem a velocidade
    #[arg(long, default_value_t = 2, value_parser = clap::value_parser!(u64).range(1..=60))]
    seconds: u64,

    /// Quantos programas listar
    #[arg(long, default_value_t = 15)]
    top: usize,

    /// Lista também as conexões de cada programa
    #[arg(long)]
    connections: bool,
}

/// Reads the traffic twice and prints who is using the network.
pub fn run(arguments: &NetworkArguments) -> anyhow::Result<()> {
    let probe = NettopTrafficProbe::new();
    let measure = MeasureTraffic::new(&probe);
    let mut meter = TrafficMeter::default();
    // The first reading only sets the starting point of the counters.
    measure.execute(&mut meter).context(UNREADABLE)?;
    thread::sleep(Duration::from_secs(arguments.seconds));
    let snapshot = measure.execute(&mut meter).context(UNREADABLE)?;
    print_snapshot(&snapshot, arguments);
    Ok(())
}

fn print_snapshot(snapshot: &TrafficSnapshot, arguments: &NetworkArguments) {
    println!("Recebendo agora    {}", snapshot.download);
    println!("Enviando agora     {}", snapshot.upload);
    println!("Programas na rede  {}", snapshot.programs.len());
    println!("Conexões com dados {}", snapshot.connection_count());
    // One decimal place, with the pt-BR decimal comma.
    let resent = format!("{:.1}", snapshot.retransmitted_ratio() * 100.0).replace('.', ",");
    println!("Reenviado          {resent}% do que foi enviado");

    println!(
        "\n{:<NAME_WIDTH$}  {:>11}  {:>11}  {:>10}  {:>10}  Conexões",
        "Programa", "Recebendo", "Enviando", "Recebido", "Enviado"
    );
    for program in snapshot.programs.iter().take(arguments.top) {
        print_program(program);
        if arguments.connections {
            program.connections.iter().for_each(print_connection);
        }
    }
    let hidden = snapshot.programs.len().saturating_sub(arguments.top);
    if hidden > 0 {
        println!("… e mais {hidden} com menos tráfego");
    }
    println!(
        "\nVelocidades medidas em {} s. Os totais contam desde que cada conexão foi aberta.",
        arguments.seconds
    );
    println!("Só os contadores do macOS foram lidos: nenhum pacote foi capturado.");
}

fn print_program(program: &ProgramTraffic) {
    let name: String = program.name.chars().take(NAME_WIDTH).collect();
    println!(
        "{name:<NAME_WIDTH$}  {:>11}  {:>11}  {:>10}  {:>10}  {}",
        program.download.to_string(),
        program.upload.to_string(),
        program.received.to_string(),
        program.sent.to_string(),
        program.connections.len()
    );
}

fn print_connection(connection: &Connection) {
    let service = connection.service().unwrap_or("-");
    let answer = connection
        .round_trip
        .map_or_else(|| "-".to_owned(), |time| format!("{} ms", time.as_millis()));
    println!(
        "    {} {:<24} {:<6} {:>7}  {:>10}  {:>10}",
        connection.transport.label(),
        connection.remote.to_string(),
        service,
        answer,
        connection.received.to_string(),
        connection.sent.to_string()
    );
}

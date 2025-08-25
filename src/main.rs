use clap::Parser;
use nozy::cli::Cli;

fn main() {
    let args = Cli::parse();
    let mut cli = nozy::cli::CliHandler::new();
    cli.handle(&args).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });
}
mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "clippers")]
#[command(about = "A CLI clipboard manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Watch,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Watch => {
            commands::watch::execute()?;
        }
    }

    Ok(())
}


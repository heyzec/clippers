mod commands;
mod r#impl;

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
    List,
    Pick,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Watch => {
            commands::watch::execute()?;
        }
        Commands::List => {
            commands::list::execute()?;
        }
        Commands::Pick => {
            commands::pick::execute()?;
        }
    }

    Ok(())
}

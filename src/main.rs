use clap::{Parser, Subcommand};
use rs_cli_tmpl::commands;
use rs_cli_tmpl::error::AppError;

#[derive(Parser)]
#[command(name = "rs-cli-tmpl")]
#[command(
    about = "Reference architecture for building Rust CLI tools",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new item to the template storage backend
    #[clap(alias = "a")]
    Add {
        /// Identifier for the item
        id: String,
        /// Content to persist with the item
        #[clap(short, long)]
        content: String,
    },
    /// List all stored item identifiers
    #[clap(alias = "ls")]
    List,
    /// Delete an item from storage
    #[clap(alias = "rm")]
    Delete {
        /// Identifier for the item to delete
        id: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result: Result<(), AppError> = match cli.command {
        Commands::Add { id, content } => commands::add(&id, &content),
        Commands::List => commands::list().map(|_| ()),
        Commands::Delete { id } => commands::delete(&id),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

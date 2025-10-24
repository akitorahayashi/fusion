use clap::{Parser, Subcommand};
use fusion::cli;
use fusion::error::AppError;

#[derive(Parser)]
#[command(name = "fusion")]
#[command(about = "Fusion CLI for managing local LLM runtimes", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage local LLM runtimes
    #[command(subcommand)]
    Llm(LlmCommands),
}

#[derive(Subcommand)]
enum LlmCommands {
    /// Start the configured LLM runtimes
    Up,
    /// Stop the configured LLM runtimes
    Down {
        /// Force-stop services using SIGKILL
        #[arg(short, long, default_value_t = false)]
        force: bool,
    },
    /// Display runtime status information
    Ps,
    /// Show log file locations for managed runtimes
    Logs,
}

fn main() {
    let cli = Cli::parse();

    let result: Result<(), AppError> = match cli.command {
        Commands::Llm(subcommand) => match subcommand {
            LlmCommands::Up => cli::handle_up(),
            LlmCommands::Down { force } => cli::handle_down(force),
            LlmCommands::Ps => cli::handle_ps(),
            LlmCommands::Logs => cli::handle_logs(),
        },
    };

    if let Err(err) = result {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

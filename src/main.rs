use clap::{Parser, Subcommand};
use fusion::cli::{self, StartOptions};
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
    /// Manage the Ollama runtime
    #[command(subcommand)]
    #[clap(visible_alias = "ol")]
    Ollama(ServiceCommands),
    /// Manage the MLX runtime
    #[command(subcommand)]
    #[clap(visible_alias = "mx")]
    Mlx(ServiceCommands),
    /// Display runtime status information for all services
    #[clap(visible_alias = "p")]
    Ps,
    /// Show log file locations for managed runtimes
    #[clap(visible_alias = "l")]
    Logs,
}

#[derive(Subcommand)]
enum ServiceCommands {
    /// Start the service
    Up(StartOptions),
    /// Stop the service
    Down {
        /// Force-stop services using SIGKILL
        #[arg(short, long, default_value_t = false)]
        force: bool,
    },
    /// Display runtime status for this service
    Ps,
    /// Show log file locations for this service
    Logs,
}

fn main() {
    let cli = Cli::parse();

    let result: Result<(), AppError> = match cli.command {
        Commands::Ollama(service_command) => match service_command {
            ServiceCommands::Up(options) => cli::handle_ollama_up(options),
            ServiceCommands::Down { force } => cli::handle_ollama_down(force),
            ServiceCommands::Ps => cli::handle_ollama_ps(),
            ServiceCommands::Logs => cli::handle_ollama_logs(),
        },
        Commands::Mlx(service_command) => match service_command {
            ServiceCommands::Up(options) => cli::handle_mlx_up(options),
            ServiceCommands::Down { force } => cli::handle_mlx_down(force),
            ServiceCommands::Ps => cli::handle_mlx_ps(),
            ServiceCommands::Logs => cli::handle_mlx_logs(),
        },
        Commands::Ps => cli::handle_ps(),
        Commands::Logs => cli::handle_logs(),
    };

    if let Err(err) = result {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

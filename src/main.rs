use clap::{Parser, Subcommand};
use fusion::cli::{self, ServiceType, StartOptions};
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
        Commands::Ollama(service_command) => {
            handle_service_command(ServiceType::Ollama, service_command)
        }
        Commands::Mlx(service_command) => handle_service_command(ServiceType::Mlx, service_command),
        Commands::Ps => cli::handle_ps(),
        Commands::Logs => cli::handle_logs(),
    };

    if let Err(err) = result {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn handle_service_command(
    service_type: ServiceType,
    command: ServiceCommands,
) -> Result<(), AppError> {
    match command {
        ServiceCommands::Up(options) => cli::handle_up(service_type, options),
        ServiceCommands::Down { force } => cli::handle_down(service_type, force),
        ServiceCommands::Ps => cli::handle_ps_single(service_type),
        ServiceCommands::Logs => cli::handle_logs_single(service_type),
    }
}

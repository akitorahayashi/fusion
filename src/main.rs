use clap::{Parser, Subcommand};
use fusion::cli::{self, ServiceConfigCommand, ServiceType};
use fusion::error::AppError;

#[derive(Parser)]
#[command(name = "fusion")]
#[command(version)]
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
    /// Manage global configuration
    #[clap(visible_alias = "cf")]
    #[command(subcommand)]
    Config(ConfigCommands),
}

#[derive(Subcommand)]
enum ServiceCommands {
    /// Start the service using configuration defaults
    Up,
    /// Stop the service
    #[clap(visible_alias = "d")]
    Down {
        /// Force-stop services using SIGKILL
        #[arg(short, long, default_value_t = false)]
        force: bool,
    },
    /// Display runtime status for this service
    Ps,
    /// Show log file locations for this service
    #[clap(visible_alias = "lg")]
    Log,
    /// Check health by running a minimal inference request
    #[clap(visible_alias = "hl")]
    Health,
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show the current configuration file contents
    Show,
    /// Create a symlink to the configuration file in the current directory
    Edit,
    /// Print the configuration file path
    Path,
    /// Reset configuration file to default values
    Reset,
}

fn main() {
    let cli = Cli::parse();

    let result: Result<(), AppError> = match cli.command {
        Commands::Ollama(service_command) => {
            handle_service_command(ServiceType::Ollama, service_command)
        }
        Commands::Mlx(service_command) => handle_service_command(ServiceType::Mlx, service_command),
        Commands::Ps => cli::handle_ps(),
        Commands::Config(config_command) => cli::handle_config(map_config_command(config_command)),
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
        ServiceCommands::Up => cli::handle_up(service_type),
        ServiceCommands::Down { force } => cli::handle_down(service_type, force),
        ServiceCommands::Ps => cli::handle_ps_single(service_type),
        ServiceCommands::Log => cli::handle_logs_single(service_type),
        ServiceCommands::Health => cli::handle_health_single(service_type),
    }
}

fn map_config_command(cmd: ConfigCommands) -> ServiceConfigCommand {
    match cmd {
        ConfigCommands::Show => ServiceConfigCommand::Show,
        ConfigCommands::Edit => ServiceConfigCommand::Edit,
        ConfigCommands::Path => ServiceConfigCommand::Path,
        ConfigCommands::Reset => ServiceConfigCommand::Reset,
    }
}

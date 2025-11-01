use clap::{Parser, Subcommand};
use fusion::cli::{self, RunOverrides, ServiceConfigCommand, ServiceType};
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
    /// Start the service using configuration defaults
    Up,
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
    /// Run a prompt against the running service
    #[clap(visible_alias = "r")]
    Run {
        /// The prompt to send to the model
        prompt: String,
        /// Override the configured model name
        #[arg(short, long)]
        model: Option<String>,
        /// Override the configured sampling temperature
        #[arg(short, long)]
        temperature: Option<f32>,
        /// Override the configured system prompt
        #[arg(short, long)]
        system: Option<String>,
    },
    /// Manage configuration for this service
    #[clap(visible_alias = "cf")]
    #[command(subcommand)]
    Config(ConfigCommands),
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show the current configuration file contents
    Show,
    /// Open the configuration file in $EDITOR
    Edit,
    /// Print the configuration file path
    Path,
    /// Set a configuration value (e.g. run.model llama3)
    Set {
        /// Dot-separated key to update (e.g. 'ollama_run.model')
        key: String,
        /// Value to write to the key
        value: String,
    },
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
        ServiceCommands::Up => cli::handle_up(service_type),
        ServiceCommands::Down { force } => cli::handle_down(service_type, force),
        ServiceCommands::Ps => cli::handle_ps_single(service_type),
        ServiceCommands::Logs => cli::handle_logs_single(service_type),
        ServiceCommands::Run { prompt, model, temperature, system } => {
            let overrides = RunOverrides { model, temperature, system };
            cli::handle_run(service_type, prompt, overrides)
        }
        ServiceCommands::Config(subcommand) => {
            cli::handle_config(service_type, map_config_command(subcommand))
        }
    }
}

fn map_config_command(cmd: ConfigCommands) -> ServiceConfigCommand {
    match cmd {
        ConfigCommands::Show => ServiceConfigCommand::Show,
        ConfigCommands::Edit => ServiceConfigCommand::Edit,
        ConfigCommands::Path => ServiceConfigCommand::Path,
        ConfigCommands::Set { key, value } => ServiceConfigCommand::Set { key, value },
    }
}

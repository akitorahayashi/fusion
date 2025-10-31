use crate::core::paths;
use crate::core::process::{self, StartOutcome, StatusOutcome, StopOutcome};
use crate::core::services::{self, ManagedService};
use crate::error::AppError;
use clap::Args;

#[derive(Debug, Clone, Default, Args)]
pub struct StartOptions {
    /// Specify the host (IP) to bind to
    #[arg(long)]
    pub host: Option<String>,
    /// Specify the port to bind to
    #[arg(long)]
    pub port: Option<u16>,
}

fn handle_service_up(service: ManagedService) -> Result<(), AppError> {
    match process::start_service(&service)? {
        StartOutcome::Started { .. } => {
            println!("• {} started on {}:{}", service.name, service.host, service.port);
        }
        StartOutcome::AlreadyRunning { .. } => {
            println!("• {} already running on {}:{}", service.name, service.host, service.port);
        }
    }
    Ok(())
}

fn handle_service_down(service: ManagedService, force: bool) -> Result<(), AppError> {
    match process::stop_service(&service, force)? {
        StopOutcome::Stopped { forced, .. } => {
            if forced {
                println!("• {} force-stopped on {}:{}", service.name, service.host, service.port);
            } else {
                println!("• {} stopped on {}:{}", service.name, service.host, service.port);
            }
        }
        StopOutcome::TerminatedByName { count, forced } => {
            let action = if forced { "killed with SIGKILL" } else { "signaled" };
            println!(
                "• {} stopped by signature on {}:{} ({} process{} {action})",
                service.name,
                service.host,
                service.port,
                count,
                if count == 1 { "" } else { "es" }
            );
        }
        StopOutcome::NotRunning => {
            println!("• {} is not running on {}:{}", service.name, service.host, service.port);
        }
    }
    Ok(())
}

fn handle_service_ps(service: ManagedService) -> Result<(), AppError> {
    match process::status_service(&service)? {
        StatusOutcome::Running { pid } => {
            println!(
                "• {}: running on {}:{} (pid {pid})",
                service.name, service.host, service.port
            );
        }
        StatusOutcome::NotRunning => {
            println!("• {}: not running on {}:{}", service.name, service.host, service.port);
        }
    }
    Ok(())
}

fn handle_service_logs(service: ManagedService) -> Result<(), AppError> {
    paths::ensure_pid_dir()?;
    println!("• {}: {}", service.name, service.log_path().display());
    Ok(())
}

pub fn handle_ollama_up(options: StartOptions) -> Result<(), AppError> {
    println!("🚀 Starting Ollama...");
    let StartOptions { host, port } = options;
    let service = services::create_ollama_service(host, port);
    handle_service_up(service)
}

pub fn handle_ollama_down(force: bool) -> Result<(), AppError> {
    println!("🛑 Stopping Ollama...");
    let dummy = services::create_ollama_service(None, None);
    let (host_override, port_override) =
        process::read_config(&dummy)?.map(|(h, p)| (Some(h), Some(p))).unwrap_or((None, None));
    let service = services::create_ollama_service(host_override, port_override);
    handle_service_down(service, force)
}

pub fn handle_ollama_ps() -> Result<(), AppError> {
    println!("ℹ️  Ollama status:");
    let dummy = services::create_ollama_service(None, None);
    let (host_override, port_override) =
        process::read_config(&dummy)?.map(|(h, p)| (Some(h), Some(p))).unwrap_or((None, None));
    let service = services::create_ollama_service(host_override, port_override);
    handle_service_ps(service)
}

pub fn handle_ollama_logs() -> Result<(), AppError> {
    println!("📜 Ollama log location:");
    let dummy = services::create_ollama_service(None, None);
    let (host_override, port_override) =
        process::read_config(&dummy)?.map(|(h, p)| (Some(h), Some(p))).unwrap_or((None, None));
    let service = services::create_ollama_service(host_override, port_override);
    handle_service_logs(service)
}

pub fn handle_mlx_up(options: StartOptions) -> Result<(), AppError> {
    println!("🚀 Starting MLX...");
    let StartOptions { host, port } = options;
    let service = services::create_mlx_service(host, port)?;
    handle_service_up(service)
}

pub fn handle_mlx_down(force: bool) -> Result<(), AppError> {
    println!("🛑 Stopping MLX...");
    let dummy = services::create_mlx_service(None, Some(8080))?;
    let (host_override, port_override) =
        process::read_config(&dummy)?.map(|(h, p)| (Some(h), Some(p))).unwrap_or((None, Some(8080)));
    let service = services::create_mlx_service(host_override, port_override)?;
    handle_service_down(service, force)
}

pub fn handle_mlx_ps() -> Result<(), AppError> {
    println!("ℹ️  MLX status:");
    let dummy = services::create_mlx_service(None, Some(8080))?;
    let (host_override, port_override) =
        process::read_config(&dummy)?.map(|(h, p)| (Some(h), Some(p))).unwrap_or((None, Some(8080)));
    let service = services::create_mlx_service(host_override, port_override)?;
    handle_service_ps(service)
}

pub fn handle_mlx_logs() -> Result<(), AppError> {
    println!("📜 MLX log location:");
    let dummy = services::create_mlx_service(None, Some(8080))?;
    let (host_override, port_override) =
        process::read_config(&dummy)?.map(|(h, p)| (Some(h), Some(p))).unwrap_or((None, Some(8080)));
    let service = services::create_mlx_service(host_override, port_override)?;
    handle_service_logs(service)
}

pub fn handle_ps() -> Result<(), AppError> {
    println!("ℹ️  Status for LLM runtimes:");
    for service in services::default_services()? {
        handle_service_ps(service)?;
    }
    Ok(())
}

pub fn handle_logs() -> Result<(), AppError> {
    println!("Log files:");
    for service in services::default_services()? {
        handle_service_logs(service)?;
    }
    println!("Use 'tail -f <log>' to follow output.");
    Ok(())
}

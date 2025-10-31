use super::ServiceType;
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

fn service_label(service_type: ServiceType) -> &'static str {
    match service_type {
        ServiceType::Ollama => "Ollama",
        ServiceType::Mlx => "MLX",
    }
}

fn create_service(
    service_type: ServiceType,
    host: Option<String>,
    port: Option<u16>,
) -> Result<ManagedService, AppError> {
    match service_type {
        ServiceType::Ollama => Ok(services::create_ollama_service(host, port)),
        ServiceType::Mlx => services::create_mlx_service(host, port),
    }
}

fn load_service(service_type: ServiceType) -> Result<ManagedService, AppError> {
    match service_type {
        ServiceType::Ollama => services::load_ollama_service(),
        ServiceType::Mlx => services::load_mlx_service(),
    }
}

pub fn handle_up(service_type: ServiceType, options: StartOptions) -> Result<(), AppError> {
    let StartOptions { host, port } = options;
    println!("🚀 Starting {}...", service_label(service_type));
    let service = create_service(service_type, host, port)?;
    handle_service_up(service)
}

pub fn handle_down(service_type: ServiceType, force: bool) -> Result<(), AppError> {
    println!("🛑 Stopping {}...", service_label(service_type));
    let service = load_service(service_type)?;
    handle_service_down(service, force)
}

pub fn handle_ps_single(service_type: ServiceType) -> Result<(), AppError> {
    println!("ℹ️  {} status:", service_label(service_type));
    let service = load_service(service_type)?;
    handle_service_ps(service)
}

pub fn handle_logs_single(service_type: ServiceType) -> Result<(), AppError> {
    println!("📜 {} log location:", service_label(service_type));
    let service = load_service(service_type)?;
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

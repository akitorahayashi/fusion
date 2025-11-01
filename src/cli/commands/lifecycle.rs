use super::shared::{load_config, service_for_runtime, service_for_up};
use crate::cli::{ServiceType, service_label};
use crate::core::paths;
use crate::core::process::{self, StartOutcome, StatusOutcome, StopOutcome};
use crate::core::services::{self, ManagedService};
use crate::error::AppError;
use std::fs;
use std::io;

const LOG_TAIL_LINES: usize = 15;

pub fn handle_up(service_type: ServiceType) -> Result<(), AppError> {
    println!("🚀 Starting {}...", service_label(service_type));
    let cfg = load_config()?;
    let service = service_for_up(&cfg, service_type);
    handle_service_up(service)
}

pub fn handle_down(service_type: ServiceType, force: bool) -> Result<(), AppError> {
    println!("🛑 Stopping {}...", service_label(service_type));
    let cfg = load_config()?;
    let service = service_for_runtime(&cfg, service_type)?;
    handle_service_down(service, force)
}

pub fn handle_ps_single(service_type: ServiceType) -> Result<(), AppError> {
    println!("ℹ️  {} status:", service_label(service_type));
    let cfg = load_config()?;
    let service = service_for_runtime(&cfg, service_type)?;
    handle_service_ps(service)
}

pub fn handle_logs_single(service_type: ServiceType) -> Result<(), AppError> {
    println!("📜 {} log location:", service_label(service_type));
    let cfg = load_config()?;
    let service = service_for_runtime(&cfg, service_type)?;
    handle_service_logs(service)
}

pub fn handle_ps() -> Result<(), AppError> {
    println!("ℹ️  Status for LLM runtimes:");
    let cfg = load_config()?;
    for service in services::default_services(&cfg)? {
        handle_service_ps(service)?;
    }
    Ok(())
}

pub fn handle_logs() -> Result<(), AppError> {
    println!("Log files:");
    let cfg = load_config()?;
    for service in services::default_services(&cfg)? {
        handle_service_logs(service)?;
    }
    println!("Use 'tail -f <log>' to follow output.");
    Ok(())
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
    let log_path = service.log_path();
    println!("• {}: {}", service.name, log_path.display());
    match fs::read_to_string(&log_path) {
        Ok(contents) => {
            for line in tail_lines(&contents, LOG_TAIL_LINES) {
                println!("    {line}");
            }
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            println!("    (log file not found)");
        }
        Err(err) => return Err(err.into()),
    }
    Ok(())
}

fn tail_lines(contents: &str, count: usize) -> Vec<String> {
    let lines: Vec<_> = contents.lines().map(|line| line.to_string()).collect();
    let start = lines.len().saturating_sub(count);
    lines[start..].to_vec()
}

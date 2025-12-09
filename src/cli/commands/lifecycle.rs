use super::shared::{load_config, service_for_runtime, service_for_up};
use crate::cli::{ServiceType, service_label};
use crate::core::config::Config;
use crate::core::health;
use crate::core::paths;
use crate::core::process::{self, StartOutcome, StatusOutcome, StopOutcome};
use crate::core::services::{self, ManagedService};
use crate::error::AppError;
use std::collections::VecDeque;
use std::fs;
use std::io;
use std::thread;
use std::time::{Duration, Instant};

const LOG_TAIL_LINES: usize = 15;
const DEFAULT_STARTUP_TIMEOUT_SECS: u64 = 300;
const POLLING_INTERVAL_MS: u64 = 1000;

pub fn handle_up(service_type: ServiceType) -> Result<(), AppError> {
    println!("🚀 Starting {}...", service_label(service_type));
    let cfg = load_config()?;
    let service = service_for_up(&cfg, service_type);
    handle_service_up(service, &cfg)
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

fn model_name_for_service<'a>(service: &ManagedService, cfg: &'a Config) -> &'a str {
    if service.name == "ollama" {
        cfg.ollama_server.model.as_str()
    } else {
        cfg.mlx_server.model.as_str()
    }
}

fn handle_service_up(service: ManagedService, cfg: &Config) -> Result<(), AppError> {
    let model_name = model_name_for_service(&service, cfg);

    match process::start_service(&service)? {
        StartOutcome::Started { pid } => {
            println!("• Process spawned with PID {}. Loading model...", pid);
            wait_until_ready(&service, pid, model_name)?;
            println!("✅ {} is ready on {}:{}", service.name, service.host, service.port);
        }
        StartOutcome::AlreadyRunning { pid } => {
            println!("• {} already running (pid {}). Checking health...", service.name, pid);
            wait_until_ready(&service, pid, model_name)?;
            println!("✅ {} is ready.", service.name);
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
    let log_path = service.log_path()?;
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

fn tail_lines(contents: &str, count: usize) -> impl Iterator<Item = String> {
    let mut lines = VecDeque::with_capacity(count);
    for line in contents.lines() {
        if lines.len() == count {
            lines.pop_front();
        }
        lines.push_back(line.to_string());
    }
    lines.into_iter()
}

fn wait_until_ready(service: &ManagedService, pid: i32, model_name: &str) -> Result<(), AppError> {
    let start = Instant::now();
    let timeout_secs = startup_timeout_secs();
    let timeout = Duration::from_secs(timeout_secs);

    println!("⏳ Waiting for {} to become ready (Timeout: {}s)...", service.name, timeout_secs);

    while start.elapsed() < timeout {
        if !process::is_process_alive(service, pid) {
            let log_tail = process::read_stderr_tail(service, 10).unwrap_or_default();
            return Err(AppError::process_error(
                service.name,
                format!("Process died unexpectedly during startup.\nCheck logs:\n{}", log_tail),
            ));
        }

        match health::check_inference_readiness(service, model_name, 2) {
            Ok(_) => return Ok(()),
            Err(_) => {
                thread::sleep(Duration::from_millis(POLLING_INTERVAL_MS));
            }
        }
    }

    Err(AppError::process_error(service.name, "Timed out waiting for service to be ready."))
}

fn startup_timeout_secs() -> u64 {
    if let Ok(value) = std::env::var("FUSION_STARTUP_TIMEOUT_SECS")
        && let Ok(parsed) = value.parse::<u64>()
    {
        return parsed;
    }
    DEFAULT_STARTUP_TIMEOUT_SECS
}

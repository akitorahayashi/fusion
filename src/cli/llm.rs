use crate::core::paths;
use crate::core::process::{self, StartOutcome, StatusOutcome, StopOutcome};
use crate::core::services;
use crate::error::AppError;

/// Start all managed LLM services.
pub fn handle_up() -> Result<(), AppError> {
    let services = services::default_services()?;
    println!("🚀 Starting LLM runtimes...");

    for service in services {
        match process::start_service(&service)? {
            StartOutcome::Started { pid } => {
                println!("• {} started (pid {pid})", service.name);
            }
            StartOutcome::AlreadyRunning { pid } => {
                println!("• {} already running (pid {pid})", service.name);
            }
        }
    }

    Ok(())
}

/// Stop all managed LLM services.
pub fn handle_down(force: bool) -> Result<(), AppError> {
    let services = services::default_services()?;
    println!("🛑 Stopping LLM runtimes...");

    for service in services {
        match process::stop_service(&service, force)? {
            StopOutcome::Stopped { pid, forced } => {
                if forced {
                    println!("• {} force-stopped (pid {pid})", service.name);
                } else {
                    println!("• {} stopped (pid {pid})", service.name);
                }
            }
            StopOutcome::TerminatedByName { count, forced } => {
                if forced {
                    println!(
                        "• {} stopped by name match ({} process{} killed with SIGKILL)",
                        service.name,
                        count,
                        if count == 1 { "" } else { "es" }
                    );
                } else {
                    println!(
                        "• {} stopped by name match ({} process{} signaled)",
                        service.name,
                        count,
                        if count == 1 { "" } else { "es" }
                    );
                }
            }
            StopOutcome::NotRunning => {
                println!("• {} is not running", service.name);
            }
        }
    }

    Ok(())
}

/// Report the status of managed LLM services.
pub fn handle_ps() -> Result<(), AppError> {
    let services = services::default_services()?;
    println!("ℹ️  Status for LLM runtimes:");

    for service in services {
        match process::status_service(&service)? {
            StatusOutcome::Running { pid } => {
                println!("• {}: running (pid {pid})", service.name);
            }
            StatusOutcome::NotRunning => {
                println!("• {}: not running", service.name);
            }
        }
    }

    Ok(())
}

/// Display log file locations for managed LLM services.
pub fn handle_logs() -> Result<(), AppError> {
    let services = services::default_services()?;
    paths::ensure_pid_dir()?;
    println!("Log files:");
    for service in services {
        println!("• {}: {}", service.name, service.log_path().display());
    }
    println!("Use 'tail -f <log>' to follow output.");
    Ok(())
}

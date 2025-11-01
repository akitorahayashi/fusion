mod common;

use common::CliTestContext;
use fusion::cli::{self, ServiceType};
use fusion::core::process::{DriverGuard, ProcessDriver, install_driver};
use fusion::core::services::ManagedService;
use fusion::error::AppError;
use serial_test::serial;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

struct DriverState {
    next_pid: i32,
    running: HashSet<String>,
    events: Vec<String>,
}

#[derive(Clone)]
struct MockDriver {
    state: Arc<Mutex<DriverState>>,
}

impl MockDriver {
    fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(DriverState {
                next_pid: 10_000,
                running: HashSet::new(),
                events: Vec::new(),
            })),
        }
    }

    fn events(&self) -> Vec<String> {
        let state = self.state.lock().expect("driver state poisoned");
        state.events.clone()
    }

    fn reset_events(&self) {
        let mut state = self.state.lock().expect("driver state poisoned");
        state.events.clear();
    }
}

impl ProcessDriver for MockDriver {
    fn spawn(
        &self,
        service: &ManagedService,
        _log_path: &std::path::Path,
    ) -> Result<i32, AppError> {
        let mut state = self.state.lock().expect("driver state poisoned");
        let pid = state.next_pid;
        state.next_pid += 1;
        state.running.insert(service.name.to_string());
        state.events.push(format!("start:{}", service.name));
        Ok(pid)
    }

    fn is_running(&self, service: &ManagedService, _pid: i32) -> bool {
        let mut state = self.state.lock().expect("driver state poisoned");
        state.events.push(format!("status:{}", service.name));
        state.running.contains(service.name)
    }

    fn is_running_by_signature(&self, service: &ManagedService) -> Option<i32> {
        let mut state = self.state.lock().expect("driver state poisoned");
        state.events.push(format!("status-by-sig:{}", service.name));
        if state.running.contains(service.name) {
            Some(12345) // Mock PID
        } else {
            None
        }
    }

    fn signal(&self, service: &ManagedService, _pid: i32, force: bool) -> Result<bool, AppError> {
        let mut state = self.state.lock().expect("driver state poisoned");
        let removed = state.running.remove(service.name);
        state.events.push(format!("signal:{}:{}", service.name, force));
        Ok(removed)
    }

    fn kill_by_signature(&self, service: &ManagedService, force: bool) -> Result<usize, AppError> {
        let mut state = self.state.lock().expect("driver state poisoned");
        let was_running = state.running.remove(service.name);
        if was_running {
            state.events.push(format!("kill:{}:{}", service.name, force));
            Ok(1)
        } else {
            state.events.push(format!("kill-miss:{}:{}", service.name, force));
            Ok(0)
        }
    }
}

fn install_mock_driver() -> (DriverGuard, MockDriver) {
    let driver = MockDriver::new();
    let guard = install_driver(Box::new(driver.clone()));
    (guard, driver)
}

#[test]
#[serial]
fn llm_ollama_up_starts_service() {
    let _ctx = CliTestContext::new();
    let (_guard, driver) = install_mock_driver();

    cli::handle_up(ServiceType::Ollama).expect("ollama up should succeed");

    let events = driver.events();
    assert!(events.iter().any(|e| e == "start:ollama"));
}

#[test]
#[serial]
fn llm_mlx_up_starts_service() {
    let _ctx = CliTestContext::new();
    let (_guard, driver) = install_mock_driver();

    cli::handle_up(ServiceType::Mlx).expect("mlx up should succeed");

    let events = driver.events();
    assert!(events.iter().any(|e| e == "start:mlx"));
}

#[test]
#[serial]
fn llm_ollama_down_stops_service() {
    let _ctx = CliTestContext::new();
    let (_guard, driver) = install_mock_driver();

    cli::handle_up(ServiceType::Ollama).expect("ollama up should succeed");
    driver.reset_events();
    cli::handle_down(ServiceType::Ollama, false).expect("ollama down should succeed");

    let events = driver.events();
    assert!(events.iter().any(|e| e == "signal:ollama:false"));
}

#[test]
#[serial]
fn llm_mlx_down_stops_service() {
    let _ctx = CliTestContext::new();
    let (_guard, driver) = install_mock_driver();

    cli::handle_up(ServiceType::Mlx).expect("mlx up should succeed");
    driver.reset_events();
    cli::handle_down(ServiceType::Mlx, false).expect("mlx down should succeed");

    let events = driver.events();
    assert!(events.iter().any(|e| e == "signal:mlx:false"));
}

#[test]
#[serial]
fn llm_force_down_kills_when_not_running() {
    let _ctx = CliTestContext::new();
    let (_guard, driver) = install_mock_driver();

    cli::handle_down(ServiceType::Ollama, true).expect("force down for ollama should not error");
    cli::handle_down(ServiceType::Mlx, true).expect("force down for mlx should not error");

    let events = driver.events();
    assert!(events.iter().any(|e| e == "kill-miss:ollama:true"));
    assert!(events.iter().any(|e| e == "kill-miss:mlx:true"));
}

#[test]
#[serial]
fn llm_mlx_ps_queries_one_service() {
    let _ctx = CliTestContext::new();
    let (_guard, driver) = install_mock_driver();

    cli::handle_up(ServiceType::Mlx).expect("mlx up should succeed");
    driver.reset_events();
    cli::handle_ps_single(ServiceType::Mlx).expect("mlx ps should succeed");

    let events = driver.events();
    assert!(events.iter().any(|e| e == "status:mlx"));
    assert!(events.iter().all(|e| !e.contains("status:ollama")));
}

#[test]
#[serial]
fn llm_ollama_ps_queries_one_service() {
    let _ctx = CliTestContext::new();
    let (_guard, driver) = install_mock_driver();

    cli::handle_up(ServiceType::Ollama).expect("ollama up should succeed");
    driver.reset_events();
    cli::handle_ps_single(ServiceType::Ollama).expect("ollama ps should succeed");

    let events = driver.events();
    assert!(events.iter().any(|e| e == "status:ollama"));
    assert!(events.iter().all(|e| !e.contains("status:mlx")));
}

#[test]
#[serial]
fn llm_global_ps_queries_all_services() {
    let _ctx = CliTestContext::new();
    let (_guard, driver) = install_mock_driver();

    cli::handle_up(ServiceType::Ollama).expect("ollama up should succeed");
    cli::handle_up(ServiceType::Mlx).expect("mlx up should succeed");
    driver.reset_events();
    cli::handle_ps().expect("handle_ps should succeed");

    let events = driver.events();
    assert!(events.iter().any(|e| e == "status:ollama"));
    assert!(events.iter().any(|e| e == "status:mlx"));
}

#[test]
#[serial]
fn llm_logs_reports_paths() {
    let ctx = CliTestContext::new();
    cli::handle_logs().expect("handle_logs should succeed");
    assert!(ctx.pid_dir().exists(), "log directory should be created");
}

use crate::core::paths;
use crate::core::services::ManagedService;
use crate::error::AppError;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::mem;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{LazyLock, RwLock};
use sysinfo::{Signal, System};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartOutcome {
    Started { pid: i32 },
    AlreadyRunning { pid: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopOutcome {
    Stopped { pid: i32, forced: bool },
    TerminatedByName { count: usize, forced: bool },
    NotRunning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusOutcome {
    Running { pid: i32 },
    NotRunning,
}

pub trait ProcessDriver: Send + Sync {
    fn spawn(&self, service: &ManagedService, log_path: &Path) -> Result<i32, AppError>;
    fn is_running(&self, pid: i32) -> bool;
    fn signal(&self, service: &ManagedService, pid: i32, force: bool) -> Result<bool, AppError>;
    fn kill_by_signature(&self, service: &ManagedService, force: bool) -> Result<usize, AppError>;
}

struct SystemProcessDriver;

static DRIVER: LazyLock<RwLock<Box<dyn ProcessDriver>>> =
    LazyLock::new(|| RwLock::new(Box::new(SystemProcessDriver)));

fn with_driver<R>(f: impl FnOnce(&dyn ProcessDriver) -> R) -> R {
    let guard = DRIVER.read().expect("process driver lock poisoned");
    f(&**guard)
}

impl ProcessDriver for SystemProcessDriver {
    fn spawn(&self, service: &ManagedService, log_path: &Path) -> Result<i32, AppError> {
        let stdout = OpenOptions::new().create(true).append(true).open(log_path)?;
        let stderr = OpenOptions::new().create(true).append(true).open(log_path)?;

        let mut command =
            Command::new(service.command.first().ok_or_else(|| {
                AppError::process_error(service.name, "service command is empty")
            })?);
        if service.command.len() > 1 {
            command.args(&service.command[1..]);
        }

        if !service.env.is_empty() {
            command.envs(service.env.iter().map(|(key, value)| (key.as_str(), value.as_str())));
        }

        command.stdin(Stdio::null());
        command.stdout(Stdio::from(stdout));
        command.stderr(Stdio::from(stderr));
        let child = command.spawn().map_err(|err| {
            AppError::process_error(service.name, format!("failed to spawn: {err}"))
        })?;
        Ok(child.id() as i32)
    }

    fn is_running(&self, pid: i32) -> bool {
        let mut system = System::new_all();
        system.refresh_processes();
        system.process(sysinfo::Pid::from_u32(pid as u32)).is_some()
    }

    fn signal(&self, _service: &ManagedService, pid: i32, force: bool) -> Result<bool, AppError> {
        let mut system = System::new_all();
        system.refresh_processes();
        let sys_pid = sysinfo::Pid::from_u32(pid as u32);
        if let Some(process) = system.process(sys_pid) {
            let signal = if force { Signal::Kill } else { Signal::Term };
            let result = process
                .kill_with(signal)
                .or_else(|| if force { Some(process.kill()) } else { None })
                .unwrap_or(false);
            Ok(result)
        } else {
            Ok(false)
        }
    }

    fn kill_by_signature(&self, service: &ManagedService, force: bool) -> Result<usize, AppError> {
        let signature = service.command.join(" ");
        let mut system = System::new_all();
        system.refresh_processes();
        let signal = if force { Signal::Kill } else { Signal::Term };

        let mut killed = 0;
        for process in system.processes().values() {
            let command_line = if process.cmd().is_empty() {
                process.name().to_string()
            } else {
                process.cmd().join(" ")
            };

            if command_line.starts_with(&signature) {
                let result = process
                    .kill_with(signal)
                    .or_else(|| if force { Some(process.kill()) } else { None })
                    .unwrap_or(false);
                if result {
                    killed += 1;
                }
            }
        }

        if killed > 0 {
            remove_pid(service)?;
        }

        Ok(killed)
    }
}

pub fn start_service(service: &ManagedService) -> Result<StartOutcome, AppError> {
    ensure_pid_dir()?;

    if let Some(pid) = read_pid(service)? {
        if with_driver(|driver| driver.is_running(pid)) {
            return Ok(StartOutcome::AlreadyRunning { pid });
        }
        remove_pid(service)?;
    }

    let log_path = service.log_path();
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)?;
    } else {
        fs::create_dir_all(paths::pid_dir())?;
    }

    let pid = with_driver(|driver| driver.spawn(service, &log_path))?;
    write_pid(service, pid)?;

    Ok(StartOutcome::Started { pid })
}

pub fn stop_service(service: &ManagedService, force: bool) -> Result<StopOutcome, AppError> {
    if let Some(pid) = read_pid(service)? {
        if with_driver(|driver| driver.is_running(pid)) {
            if with_driver(|driver| driver.signal(service, pid, force))? {
                remove_pid(service)?;
                return Ok(StopOutcome::Stopped { pid, forced: force });
            }
            return Err(AppError::process_error(
                service.name,
                format!("failed to send signal to pid {pid}"),
            ));
        }

        remove_pid(service)?;
    }

    let killed = with_driver(|driver| driver.kill_by_signature(service, force))?;
    if killed > 0 {
        return Ok(StopOutcome::TerminatedByName { count: killed, forced: force });
    }

    Ok(StopOutcome::NotRunning)
}

pub fn status_service(service: &ManagedService) -> Result<StatusOutcome, AppError> {
    if let Some(pid) = read_pid(service)? {
        if with_driver(|driver| driver.is_running(pid)) {
            return Ok(StatusOutcome::Running { pid });
        }
        remove_pid(service)?;
    }

    Ok(StatusOutcome::NotRunning)
}

pub fn read_pid(service: &ManagedService) -> Result<Option<i32>, AppError> {
    let path = service.pid_path();
    match fs::read_to_string(&path) {
        Ok(contents) => {
            let trimmed = contents.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                trimmed.parse::<i32>().map(Some).map_err(|err| {
                    AppError::process_error(
                        service.name,
                        format!("invalid pid value '{trimmed}': {err}"),
                    )
                })
            }
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err.into()),
    }
}

pub fn write_pid(service: &ManagedService, pid: i32) -> Result<(), AppError> {
    ensure_pid_dir()?;
    let path = service.pid_path();
    let mut handle = OpenOptions::new().create(true).write(true).truncate(true).open(path)?;
    writeln!(handle, "{pid}")?;
    Ok(())
}

pub fn remove_pid(service: &ManagedService) -> Result<(), AppError> {
    let path = service.pid_path();
    match fs::remove_file(path) {
        Ok(_) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err.into()),
    }
}

fn ensure_pid_dir() -> Result<(), AppError> {
    paths::ensure_pid_dir().map(|_| ()).map_err(AppError::from)
}

pub struct DriverGuard {
    previous: Option<Box<dyn ProcessDriver>>,
}

pub fn install_driver(driver: Box<dyn ProcessDriver>) -> DriverGuard {
    let mut guard = DRIVER.write().expect("process driver lock poisoned");
    let previous = mem::replace(&mut *guard, driver);
    DriverGuard { previous: Some(previous) }
}

impl Drop for DriverGuard {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.take() {
            let mut guard = DRIVER.write().expect("process driver lock poisoned");
            *guard = previous;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_support::TestProject;
    use std::collections::HashMap;

    fn service(_project: &TestProject) -> ManagedService {
        ManagedService {
            name: "test",
            command: vec!["dummy".into()],
            log_filename: "test.log",
            pid_filename: "test.pid",
            env: HashMap::new(),
        }
    }

    #[test]
    #[serial_test::serial]
    fn write_and_read_pid_round_trip() {
        let project = TestProject::new();
        let svc = service(&project);

        write_pid(&svc, 1234).expect("pid should be written");
        let read = read_pid(&svc).expect("pid should be readable");
        assert_eq!(read, Some(1234));
        assert!(svc.pid_path().exists());
    }

    #[test]
    #[serial_test::serial]
    fn remove_pid_is_idempotent() {
        let project = TestProject::new();
        let svc = service(&project);

        write_pid(&svc, 999).unwrap();
        remove_pid(&svc).expect("pid file should be removed");
        assert!(!svc.pid_path().exists());
        // Removing again should not error.
        remove_pid(&svc).expect("second removal should succeed");
    }

    #[test]
    #[serial_test::serial]
    fn status_service_clears_stale_pid() {
        let project = TestProject::new();
        let svc = service(&project);

        write_pid(&svc, i32::MAX).unwrap();
        let status = status_service(&svc).expect("status check should succeed");
        assert!(matches!(status, StatusOutcome::NotRunning));
        assert!(!svc.pid_path().exists(), "stale pid file should be removed");
    }
}

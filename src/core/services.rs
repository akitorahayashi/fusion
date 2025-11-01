use crate::core::config::{Config, MlxServerConfig, OllamaServerConfig};
use crate::core::{config, paths, process};
use crate::error::AppError;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ManagedService {
    pub name: &'static str,
    pub host: String,
    pub port: u16,
    pub command: Vec<String>,
    pub log_filename: &'static str,
    pub pid_filename: &'static str,
    pub config_filename: &'static str,
    pub env: HashMap<String, String>,
}

impl ManagedService {
    pub fn log_path(&self) -> PathBuf {
        paths::pid_dir().join(self.log_filename)
    }

    pub fn pid_path(&self) -> PathBuf {
        paths::pid_dir().join(self.pid_filename)
    }

    pub fn config_path(&self) -> PathBuf {
        paths::pid_dir().join(self.config_filename)
    }
}

pub fn create_ollama_service(cfg: &OllamaServerConfig) -> ManagedService {
    let mut env_map = config::server_env(&cfg.extra, "OLLAMA_");
    env_map.insert("OLLAMA_HOST".into(), config::format_host_port(&cfg.host, cfg.port));

    ManagedService {
        name: "ollama",
        host: cfg.host.clone(),
        port: cfg.port,
        command: vec!["ollama".into(), "serve".into()],
        log_filename: "ollama.log",
        pid_filename: "ollama.pid",
        config_filename: "ollama.config",
        env: env_map,
    }
}

pub fn create_mlx_service(cfg: &MlxServerConfig) -> ManagedService {
    let env_map = config::server_env(&cfg.extra, "MLX_");

    ManagedService {
        name: "mlx",
        host: cfg.host.clone(),
        port: cfg.port,
        command: vec![
            "mlx_lm.server".into(),
            "--model".into(),
            cfg.model.clone(),
            "--host".into(),
            cfg.host.clone(),
            "--port".into(),
            cfg.port.to_string(),
        ],
        log_filename: "mlx.log",
        pid_filename: "mlx.pid",
        config_filename: "mlx.config",
        env: env_map,
    }
}

pub fn load_ollama_service(cfg: &OllamaServerConfig) -> Result<ManagedService, AppError> {
    let mut service = create_ollama_service(cfg);
    if let Some((host, port)) = process::read_config(&service)? {
        service.host = host.clone();
        service.port = port;
        service.env.insert("OLLAMA_HOST".into(), config::format_host_port(&host, port));
    }
    Ok(service)
}

pub fn load_mlx_service(cfg: &MlxServerConfig) -> Result<ManagedService, AppError> {
    let mut service = create_mlx_service(cfg);
    if let Some((host, port)) = process::read_config(&service)? {
        service.host = host;
        service.port = port;
        // Update command host/port positions
        if let Some(host_arg) = service.command.iter().position(|arg| arg == "--host")
            && let Some(target) = service.command.get_mut(host_arg + 1)
        {
            *target = service.host.clone();
        }
        if let Some(port_arg) = service.command.iter().position(|arg| arg == "--port")
            && let Some(target) = service.command.get_mut(port_arg + 1)
        {
            *target = service.port.to_string();
        }
    }
    Ok(service)
}

pub fn default_services(cfg: &Config) -> Result<Vec<ManagedService>, AppError> {
    Ok(vec![load_ollama_service(&cfg.ollama_server)?, load_mlx_service(&cfg.mlx_server)?])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config;
    use crate::core::test_support::TestProject;

    #[test]
    #[serial_test::serial]
    fn ollama_service_uses_defaults() {
        let _project = TestProject::new();
        let cfg = config::Config::default();
        let service = create_ollama_service(&cfg.ollama_server);
        assert_eq!(service.name, "ollama");
        assert_eq!(service.command, vec!["ollama", "serve"]);
        assert_eq!(service.log_filename, "ollama.log");
        assert_eq!(service.host, "127.0.0.1");
        assert_eq!(service.port, 11434);
        assert_eq!(service.env.get("OLLAMA_HOST").unwrap(), "127.0.0.1:11434");
    }

    #[test]
    #[serial_test::serial]
    fn default_services_includes_mlx() {
        let _project = TestProject::new();
        let mut cfg = config::Config::default();
        cfg.mlx_server.port = 5050;

        let services = default_services(&cfg).expect("services should resolve");
        assert_eq!(services.len(), 2);
        let mlx = services.iter().find(|svc| svc.name == "mlx").unwrap();
        assert!(mlx.command.contains(&"mlx_lm.server".to_string()));
        assert!(mlx.command.contains(&"5050".to_string()));
        assert_eq!(mlx.host, "127.0.0.1");
        assert_eq!(mlx.port, 5050);
    }

    #[test]
    #[serial_test::serial]
    fn load_ollama_service_prefers_config_file() {
        let _project = TestProject::new();
        let mut cfg = config::Config::default();
        cfg.ollama_server.host = "127.0.0.1".into();
        cfg.ollama_server.port = 11434;
        let mut configured = create_ollama_service(&cfg.ollama_server);
        configured.host = "10.0.0.1".into();
        configured.port = 1234;
        // Ensure config file is written with custom values
        process::write_config(&configured).expect("config should be written");

        let loaded = load_ollama_service(&cfg.ollama_server).expect("ollama service should load");
        assert_eq!(loaded.host, configured.host);
        assert_eq!(loaded.port, configured.port);

        // Clean up config file for subsequent tests
        process::remove_config(&configured).expect("config removal should succeed");
    }

    #[test]
    #[serial_test::serial]
    fn load_mlx_service_prefers_runtime_config() {
        let _project = TestProject::new();
        let cfg = config::Config::default();

        let mut configured = create_mlx_service(&cfg.mlx_server);
        configured.host = "10.0.0.5".into();
        configured.port = 5055;
        process::write_config(&configured).expect("config write should succeed");

        let service = load_mlx_service(&cfg.mlx_server).expect("mlx service should load");
        assert_eq!(service.port, 5055);
        assert_eq!(service.host, "10.0.0.5");

        process::remove_config(&configured).expect("config removal should succeed");
    }
}

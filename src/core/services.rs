use crate::core::{env, paths, process};
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

pub fn create_ollama_service(
    host_override: Option<String>,
    port_override: Option<u16>,
) -> ManagedService {
    let (host, port) = env::ollama_host_port(host_override, port_override);
    let mut env_map = env::ollama_environment();
    env_map.insert("OLLAMA_HOST".into(), env::format_host_port(&host, port));

    ManagedService {
        name: "ollama",
        host,
        port,
        command: vec!["ollama".into(), "serve".into()],
        log_filename: "ollama.log",
        pid_filename: "ollama.pid",
        config_filename: "ollama.config",
        env: env_map,
    }
}

pub fn create_mlx_service(
    host_override: Option<String>,
    port_override: Option<u16>,
) -> Result<ManagedService, AppError> {
    let host = env::mlx_host(host_override);
    let port = env::mlx_port(port_override)?;

    Ok(ManagedService {
        name: "mlx",
        host: host.clone(),
        port,
        command: vec![
            "mlx_lm.server".into(),
            "--model".into(),
            env::mlx_model(),
            "--host".into(),
            host,
            "--port".into(),
            port.to_string(),
        ],
        log_filename: "mlx.log",
        pid_filename: "mlx.pid",
        config_filename: "mlx.config",
        env: HashMap::new(),
    })
}

pub fn default_services() -> Result<Vec<ManagedService>, AppError> {
    let ollama_dummy = create_ollama_service(None, None);
    let (ollama_host, ollama_port) = process::read_config(&ollama_dummy)?
        .map(|(h, p)| (Some(h), Some(p)))
        .unwrap_or((None, None));
    let ollama = create_ollama_service(ollama_host, ollama_port);

    let mlx_dummy = create_mlx_service(None, Some(8080))?;
    let (mlx_host, mlx_port) =
        process::read_config(&mlx_dummy)?.map(|(h, p)| (Some(h), Some(p))).unwrap_or((None, None));
    let mlx = create_mlx_service(mlx_host, mlx_port).unwrap_or_else(|_| create_mlx_service(Some("127.0.0.1".to_string()), Some(8080)).unwrap());

    Ok(vec![ollama, mlx])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::env::{OLLAMA_HOST_DEFAULT, reset_cache_for_tests};
    use crate::core::test_support::TestProject;

    fn clear_env_vars() {
        for key in ["OLLAMA_HOST", "FUSION_OLLAMA_HOST", "FUSION_MLX_MODEL", "FUSION_MLX_PORT"] {
            unsafe {
                // SAFETY: tests run serially and take exclusive control of env vars.
                std::env::remove_var(key);
            }
        }
    }

    #[test]
    #[serial_test::serial]
    fn ollama_service_uses_defaults() {
        let _project = TestProject::new();
        clear_env_vars();
        reset_cache_for_tests();

        let service = create_ollama_service(None, None);
        assert_eq!(service.name, "ollama");
        assert_eq!(service.command, vec!["ollama", "serve"]);
        assert_eq!(service.log_filename, "ollama.log");
        assert_eq!(service.host, "127.0.0.1");
        assert_eq!(service.port, 11434);
        assert_eq!(service.env.get("OLLAMA_HOST").unwrap(), OLLAMA_HOST_DEFAULT);
    }

    #[test]
    #[serial_test::serial]
    fn default_services_includes_mlx() {
        let project = TestProject::new();
        clear_env_vars();
        reset_cache_for_tests();
        project.write_env_file("FUSION_MLX_PORT=5050\n");

        let services = default_services().expect("services should resolve");
        assert_eq!(services.len(), 2);
        let mlx = services.iter().find(|svc| svc.name == "mlx").unwrap();
        assert!(mlx.command.contains(&"mlx_lm.server".to_string()));
        assert!(mlx.command.contains(&"5050".to_string()));
        assert_eq!(mlx.host, "127.0.0.1");
        assert_eq!(mlx.port, 5050);
    }
}

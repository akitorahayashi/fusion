use crate::core::{env, paths};
use crate::error::AppError;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ManagedService {
    pub name: &'static str,
    pub command: Vec<String>,
    pub log_filename: &'static str,
    pub pid_filename: &'static str,
    pub env: HashMap<String, String>,
}

impl ManagedService {
    pub fn log_path(&self) -> PathBuf {
        paths::pid_dir().join(self.log_filename)
    }

    pub fn pid_path(&self) -> PathBuf {
        paths::pid_dir().join(self.pid_filename)
    }
}

pub fn create_ollama_service() -> ManagedService {
    ManagedService {
        name: "ollama",
        command: vec!["ollama".into(), "serve".into()],
        log_filename: "ollama.log",
        pid_filename: "ollama.pid",
        env: env::ollama_environment(),
    }
}

pub fn create_mlx_service() -> Result<ManagedService, AppError> {
    let port = env::mlx_port()?;

    Ok(ManagedService {
        name: "mlx",
        command: vec![
            "mlx_lm.server".into(),
            "--model".into(),
            env::mlx_model(),
            "--host".into(),
            "0.0.0.0".into(),
            "--port".into(),
            port.to_string(),
        ],
        log_filename: "mlx.log",
        pid_filename: "mlx.pid",
        env: HashMap::new(),
    })
}

pub fn default_services() -> Result<Vec<ManagedService>, AppError> {
    Ok(vec![create_ollama_service(), create_mlx_service()?])
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

        let service = create_ollama_service();
        assert_eq!(service.name, "ollama");
        assert_eq!(service.command, vec!["ollama", "serve"]);
        assert_eq!(service.log_filename, "ollama.log");
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
    }
}

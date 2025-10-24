use crate::core::paths;
use crate::error::AppError;
use std::collections::HashMap;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};

pub const OLLAMA_HOST_DEFAULT: &str = "0.0.0.0:11434";
pub const MLX_MODEL_DEFAULT: &str = "mlx-community/Llama-3.2-3B-Instruct-4bit";
pub const MLX_PORT_DEFAULT: u16 = 8080;

static ENV_LOADED: AtomicBool = AtomicBool::new(false);

fn load_env_once() {
    if ENV_LOADED.load(Ordering::SeqCst) {
        return;
    }

    if ENV_LOADED.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        return;
    }

    if let Ok(path) = env::var("FUSION_ENV_FILE") {
        let _ = dotenvy::from_filename(path);
        return;
    }

    let candidate = paths::project_root().join(".env");
    if candidate.exists() {
        let _ = dotenvy::from_path(&candidate);
    }
}

#[cfg(test)]
pub(crate) fn reset_cache_for_tests() {
    ENV_LOADED.store(false, Ordering::SeqCst);
}

pub fn ollama_environment() -> HashMap<String, String> {
    load_env_once();

    let mut env_map = HashMap::new();
    env_map.extend(
        [
            ("OLLAMA_CONTEXT_LENGTH", "4096"),
            ("OLLAMA_MAX_LOADED_MODELS", "1"),
            ("OLLAMA_NUM_PARALLEL", "1"),
            ("OLLAMA_MAX_QUEUE", "512"),
            ("OLLAMA_FLASH_ATTENTION", "true"),
            ("OLLAMA_KEEP_ALIVE", "10m"),
            ("OLLAMA_GPU_OVERHEAD", "1024"),
            ("OLLAMA_KV_CACHE_TYPE", "q8_0"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string())),
    );

    let host = env::var("OLLAMA_HOST")
        .or_else(|_| env::var("FUSION_OLLAMA_HOST"))
        .unwrap_or_else(|_| OLLAMA_HOST_DEFAULT.to_string());
    env_map.insert("OLLAMA_HOST".into(), host);

    env_map
}

pub fn mlx_model() -> String {
    load_env_once();
    env::var("FUSION_MLX_MODEL").unwrap_or_else(|_| MLX_MODEL_DEFAULT.to_string())
}

pub fn mlx_port() -> Result<u16, AppError> {
    load_env_once();
    let raw = env::var("FUSION_MLX_PORT").unwrap_or_else(|_| MLX_PORT_DEFAULT.to_string());
    raw.parse::<u16>()
        .map_err(|_| AppError::config_error(format!("Invalid MLX port value '{raw}'")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_support::TestProject;

    fn clear_env_vars() {
        for key in [
            "OLLAMA_HOST",
            "FUSION_OLLAMA_HOST",
            "FUSION_MLX_MODEL",
            "FUSION_MLX_PORT",
            "FUSION_ENV_FILE",
        ] {
            unsafe {
                env::remove_var(key);
            }
        }
    }

    #[test]
    #[serial_test::serial]
    fn ollama_environment_uses_defaults() {
        let _project = TestProject::new();
        clear_env_vars();
        reset_cache_for_tests();

        let env_map = ollama_environment();
        assert_eq!(env_map.get("OLLAMA_CONTEXT_LENGTH").unwrap(), "4096");
        assert_eq!(env_map.get("OLLAMA_HOST").unwrap(), OLLAMA_HOST_DEFAULT);
    }

    #[test]
    #[serial_test::serial]
    fn mlx_overrides_from_env_file() {
        let project = TestProject::new();
        clear_env_vars();
        project.write_env_file("FUSION_MLX_MODEL=custom-model\nFUSION_MLX_PORT=4242\n");
        reset_cache_for_tests();

        assert_eq!(mlx_model(), "custom-model");
        assert_eq!(mlx_port().unwrap(), 4242);
    }

    #[test]
    #[serial_test::serial]
    fn mlx_port_validation_errors() {
        let project = TestProject::new();
        clear_env_vars();
        project.write_env_file("FUSION_MLX_PORT=not-a-number\n");
        reset_cache_for_tests();

        let err = mlx_port().expect_err("port parsing should fail");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }
}

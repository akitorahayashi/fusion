use crate::core::paths;
use crate::error::AppError;
use std::collections::HashMap;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};

pub const OLLAMA_HOST_DEFAULT: &str = "127.0.0.1:11434";
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

fn parse_host_port(value: &str) -> Option<(String, u16)> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.starts_with('[') {
        if let Some(close) = trimmed.find(']') {
            let host = trimmed[1..close].to_string();
            let port_str = trimmed[close + 1..].strip_prefix(':')?;
            let port = port_str.parse::<u16>().ok()?;
            return Some((host, port));
        }
        return None;
    }

    trimmed.rsplit_once(':').and_then(|(host_part, port_part)| {
        if port_part.is_empty() {
            return None;
        }
        port_part.parse::<u16>().ok().map(|port| (host_part.to_string(), port))
    })
}

pub(crate) fn format_host_port(host: &str, port: u16) -> String {
    if host.contains(':') && !host.starts_with('[') && !host.ends_with(']') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    }
}

pub fn ollama_host_port(
    host_override: Option<String>,
    port_override: Option<u16>,
) -> (String, u16) {
    load_env_once();

    let (default_host, default_port) =
        parse_host_port(OLLAMA_HOST_DEFAULT).expect("OLLAMA_HOST_DEFAULT must be valid");

    let base = env::var("OLLAMA_HOST")
        .or_else(|_| env::var("FUSION_OLLAMA_HOST"))
        .unwrap_or_else(|_| OLLAMA_HOST_DEFAULT.to_string());

    let (env_host, env_port) =
        parse_host_port(&base).unwrap_or_else(|| (default_host.clone(), default_port));

    let host = host_override
        .filter(|value| !value.is_empty())
        .map(|value| value.trim_matches(|c| c == '[' || c == ']').to_string())
        .unwrap_or(env_host);
    let port = port_override.unwrap_or(env_port);

    (host, port)
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

    let (host, port) = ollama_host_port(None, None);
    env_map.insert("OLLAMA_HOST".into(), format_host_port(&host, port));

    env_map
}

pub fn mlx_model() -> String {
    load_env_once();
    env::var("FUSION_MLX_MODEL").unwrap_or_else(|_| MLX_MODEL_DEFAULT.to_string())
}

pub fn mlx_host(host_override: Option<String>) -> String {
    load_env_once();
    host_override
        .filter(|value| !value.is_empty())
        .or_else(|| env::var("FUSION_MLX_HOST").ok())
        .unwrap_or_else(|| "127.0.0.1".to_string())
}

pub fn mlx_port(port_override: Option<u16>) -> Result<u16, AppError> {
    if let Some(port) = port_override {
        return Ok(port);
    }

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
            "FUSION_MLX_HOST",
            "FUSION_MLX_PORT",
            "FUSION_ENV_FILE",
        ] {
            unsafe {
                // SAFETY: tests run serially and exclusively own the environment.
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
    fn ollama_host_port_prefers_overrides() {
        let _project = TestProject::new();
        clear_env_vars();
        reset_cache_for_tests();

        let (host, port) = ollama_host_port(Some("127.0.0.1".into()), Some(4242));
        assert_eq!(host, "127.0.0.1");
        assert_eq!(port, 4242);
    }

    #[test]
    #[serial_test::serial]
    fn ollama_host_port_reads_env_values() {
        let project = TestProject::new();
        clear_env_vars();
        project.write_env_file("OLLAMA_HOST=192.168.10.5:2345\n");
        reset_cache_for_tests();

        let (host, port) = ollama_host_port(None, None);
        assert_eq!(host, "192.168.10.5");
        assert_eq!(port, 2345);
    }

    #[test]
    #[serial_test::serial]
    fn ollama_host_port_handles_malformed_env() {
        let project = TestProject::new();
        clear_env_vars();
        project.write_env_file("OLLAMA_HOST=invalid\n");
        reset_cache_for_tests();

        let (host, port) = ollama_host_port(None, None);
        assert_eq!(format!("{host}:{port}"), OLLAMA_HOST_DEFAULT);
    }

    #[test]
    #[serial_test::serial]
    fn mlx_overrides_from_env_file() {
        let project = TestProject::new();
        clear_env_vars();
        project.write_env_file("FUSION_MLX_MODEL=custom-model\nFUSION_MLX_PORT=4242\n");
        reset_cache_for_tests();

        assert_eq!(mlx_model(), "custom-model");
        assert_eq!(mlx_port(None).unwrap(), 4242);
    }

    #[test]
    #[serial_test::serial]
    fn mlx_port_validation_errors() {
        let project = TestProject::new();
        clear_env_vars();
        project.write_env_file("FUSION_MLX_PORT=not-a-number\n");
        reset_cache_for_tests();

        let err = mlx_port(None).expect_err("port parsing should fail");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    #[serial_test::serial]
    fn mlx_host_prefers_override_then_env() {
        let project = TestProject::new();
        clear_env_vars();
        project.write_env_file("FUSION_MLX_HOST=192.168.1.5\n");
        reset_cache_for_tests();

        assert_eq!(mlx_host(None), "192.168.1.5");
        assert_eq!(mlx_host(Some("127.0.0.1".into())), "127.0.0.1");
    }

    #[test]
    #[serial_test::serial]
    fn mlx_host_defaults_to_loopback() {
        let _project = TestProject::new();
        clear_env_vars();
        reset_cache_for_tests();

        assert_eq!(mlx_host(None), "127.0.0.1");
    }
}

use crate::core::paths;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::Write;
use std::path::Path;
use toml::Value as TomlValue;
use toml_edit::{DocumentMut, Item, Table, Value as TomlEditValue};

mod mlx;
mod ollama;

pub use mlx::*;
pub use ollama::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub ollama_server: OllamaServerConfig,
    #[serde(default)]
    pub ollama_run: OllamaRunConfig,
    #[serde(default)]
    pub mlx_server: MlxServerConfig,
    #[serde(default)]
    pub mlx_run: MlxRunConfig,
    #[serde(default)]
    #[serde(flatten)]
    pub extra: BTreeMap<String, TomlValue>,
}

pub fn load_config() -> Result<Config, AppError> {
    ensure_config_exists()?;
    let path = paths::user_config_file()?;
    let contents = fs::read_to_string(&path)?;
    let config: Config = toml::from_str(&contents)
        .map_err(|err| AppError::config_error(format!("Failed to parse config: {err}")))?;
    Ok(config)
}

pub fn save_config(config: &Config) -> Result<(), AppError> {
    let path = paths::user_config_file()?;
    write_config_to_path(&path, config)
}

pub fn load_config_document() -> Result<DocumentMut, AppError> {
    ensure_config_exists()?;
    let path = paths::user_config_file()?;
    let contents = fs::read_to_string(&path)?;
    contents
        .parse::<DocumentMut>()
        .map_err(|err| AppError::config_error(format!("Failed to parse config: {err}")))
}

pub fn save_config_document(document: &DocumentMut) -> Result<(), AppError> {
    let path = paths::user_config_file()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(&path)?;
    file.write_all(document.to_string().as_bytes())?;
    Ok(())
}

fn write_config_to_path(path: &Path, config: &Config) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(path)?;
    let contents = toml::to_string_pretty(config)
        .map_err(|err| AppError::config_error(format!("Failed to serialise config: {err}")))?;
    file.write_all(contents.as_bytes())?;
    Ok(())
}

pub fn ensure_config_exists() -> Result<(), AppError> {
    let path = paths::user_config_file()?;
    if path.exists() {
        return Ok(());
    }

    write_config_to_path(&path, &Config::default())
}
pub fn server_env(extra: &BTreeMap<String, TomlValue>, prefix: &str) -> HashMap<String, String> {
    let mut env = HashMap::new();
    for (key, value) in extra {
        let normalized = normalise_env_key(key, prefix);
        env.insert(normalized, toml_value_to_string(value));
    }
    env
}

fn normalise_env_key(key: &str, prefix: &str) -> String {
    let upper = key.trim().to_uppercase();
    if upper.starts_with(prefix) { upper } else { format!("{prefix}{upper}") }
}

fn toml_value_to_string(value: &TomlValue) -> String {
    match value {
        TomlValue::String(s) => s.clone(),
        TomlValue::Integer(i) => i.to_string(),
        TomlValue::Float(f) => f.to_string(),
        TomlValue::Boolean(b) => b.to_string(),
        TomlValue::Datetime(dt) => dt.to_string(),
        other => other.to_string(),
    }
}

pub fn format_host_port(host: &str, port: u16) -> String {
    if host.contains(':') && !host.starts_with('[') && !host.ends_with(']') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    }
}

pub fn infer_toml_edit_value(raw: &str) -> TomlEditValue {
    let trimmed = raw.trim();
    if let Ok(boolean) = trimmed.parse::<bool>() {
        return TomlEditValue::from(boolean);
    }
    if let Ok(int) = trimmed.parse::<i64>() {
        return TomlEditValue::from(int);
    }
    if let Ok(float) = trimmed.parse::<f64>() {
        return TomlEditValue::from(float);
    }
    TomlEditValue::from(trimmed)
}

pub fn set_document_value(
    document: &mut DocumentMut,
    key_path: &[&str],
    value: TomlEditValue,
) -> Result<(), AppError> {
    if key_path.is_empty() {
        return Err(AppError::config_error("Configuration key must not be empty"));
    }
    let mut current: &mut Table = document.as_table_mut();
    for (index, segment) in key_path.iter().enumerate() {
        if index + 1 == key_path.len() {
            current.insert(segment, Item::Value(value));
            return Ok(());
        }

        let item = current.entry(segment).or_insert(Item::Table(Default::default()));
        current = item.as_table_mut().ok_or_else(|| {
            AppError::config_error(format!(
                "Configuration key '{}' conflicts with existing non-table value",
                key_path[..=index].join(".")
            ))
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::paths;
    use crate::core::test_support::TestProject;

    #[test]
    #[serial_test::serial]
    fn load_config_creates_default_file() {
        let _project = TestProject::new();
        let path = paths::user_config_file().expect("config path should resolve");
        assert!(!path.exists(), "config file should start absent");

        let cfg = load_config().expect("load_config should succeed");
        assert!(path.exists(), "config file should be created");
        assert_eq!(cfg.ollama_server.host, DEFAULT_OLLAMA_HOST);
        assert_eq!(cfg.mlx_server.port, DEFAULT_MLX_PORT);
    }

    #[test]
    #[serial_test::serial]
    fn save_and_reload_persists_changes() {
        let _project = TestProject::new();
        let mut cfg = load_config().expect("load_config should succeed");
        cfg.ollama_server.port = 12000;
        cfg.mlx_run.stream = false;
        save_config(&cfg).expect("save_config should succeed");

        let reloaded = load_config().expect("reload should succeed");
        assert_eq!(reloaded.ollama_server.port, 12000);
        assert!(!reloaded.mlx_run.stream);
    }

    #[test]
    #[serial_test::serial]
    fn set_document_value_updates_nested_key() {
        let _project = TestProject::new();
        let mut document = load_config_document().expect("document should load");
        let key = vec!["ollama_run", "model"];
        set_document_value(&mut document, &key, TomlEditValue::from("custom-model"))
            .expect("set_document_value should succeed");
        save_config_document(&document).expect("save should succeed");

        let cfg = load_config().expect("reload should succeed");
        assert_eq!(cfg.ollama_run.model, "custom-model");
    }

    #[test]
    fn server_env_prefixes_missing_keys() {
        let mut extra = BTreeMap::new();
        extra.insert("keep_alive".into(), TomlValue::String("5m".into()));
        let env = server_env(&extra, "OLLAMA_");
        assert_eq!(env.get("OLLAMA_KEEP_ALIVE"), Some(&"5m".to_string()));
    }

    #[test]
    fn infer_toml_edit_value_detects_types() {
        let bool_value = infer_toml_edit_value("true");
        assert!(bool_value.as_bool().unwrap());
        let int_value = infer_toml_edit_value("42");
        assert_eq!(int_value.as_integer().unwrap(), 42);
        let float_value = infer_toml_edit_value("1.25");
        assert!((float_value.as_float().unwrap() - 1.25).abs() < f64::EPSILON);
        let string_value = infer_toml_edit_value("hello");
        assert_eq!(string_value.as_str().unwrap(), "hello");
    }
}

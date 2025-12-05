use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use toml::Value as TomlValue;

pub const DEFAULT_MLX_HOST: &str = "127.0.0.1";
pub const DEFAULT_MLX_PORT: u16 = 8080;
pub const DEFAULT_MLX_MODEL: &str = "mlx-community/Llama-3.2-3B-Instruct-4bit";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlxServerConfig {
    #[serde(default = "default_mlx_host")]
    pub host: String,
    #[serde(default = "default_mlx_port")]
    pub port: u16,
    #[serde(default = "default_mlx_model")]
    pub model: String,
    #[serde(default)]
    #[serde(flatten)]
    pub extra: BTreeMap<String, TomlValue>,
}

impl Default for MlxServerConfig {
    fn default() -> Self {
        Self {
            host: default_mlx_host(),
            port: default_mlx_port(),
            model: default_mlx_model(),
            extra: BTreeMap::new(),
        }
    }
}

fn default_mlx_host() -> String {
    DEFAULT_MLX_HOST.to_string()
}

fn default_mlx_port() -> u16 {
    DEFAULT_MLX_PORT
}

fn default_mlx_model() -> String {
    DEFAULT_MLX_MODEL.to_string()
}

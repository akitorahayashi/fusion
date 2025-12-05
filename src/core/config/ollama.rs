use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use toml::Value as TomlValue;

pub const DEFAULT_OLLAMA_HOST: &str = "127.0.0.1";
pub const DEFAULT_OLLAMA_PORT: u16 = 11434;
pub const DEFAULT_OLLAMA_MODEL: &str = "llama3.2:3b";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaServerConfig {
    #[serde(default = "default_ollama_host")]
    pub host: String,
    #[serde(default = "default_ollama_port")]
    pub port: u16,
    #[serde(default = "default_ollama_model")]
    pub model: String,
    #[serde(default = "default_ollama_server_extra")]
    #[serde(flatten)]
    pub extra: BTreeMap<String, TomlValue>,
}

impl Default for OllamaServerConfig {
    fn default() -> Self {
        Self {
            host: default_ollama_host(),
            port: default_ollama_port(),
            model: default_ollama_model(),
            extra: default_ollama_server_extra(),
        }
    }
}

fn default_ollama_host() -> String {
    DEFAULT_OLLAMA_HOST.to_string()
}

fn default_ollama_port() -> u16 {
    DEFAULT_OLLAMA_PORT
}

fn default_ollama_model() -> String {
    DEFAULT_OLLAMA_MODEL.to_string()
}

fn default_ollama_server_extra() -> BTreeMap<String, TomlValue> {
    [
        ("OLLAMA_CONTEXT_LENGTH".into(), TomlValue::String("4096".into())),
        ("OLLAMA_MAX_LOADED_MODELS".into(), TomlValue::String("1".into())),
        ("OLLAMA_NUM_PARALLEL".into(), TomlValue::String("1".into())),
        ("OLLAMA_MAX_QUEUE".into(), TomlValue::String("512".into())),
        ("OLLAMA_FLASH_ATTENTION".into(), TomlValue::Boolean(true)),
        ("OLLAMA_KEEP_ALIVE".into(), TomlValue::String("10m".into())),
        ("OLLAMA_GPU_OVERHEAD".into(), TomlValue::String("1024".into())),
        ("OLLAMA_KV_CACHE_TYPE".into(), TomlValue::String("q8_0".into())),
    ]
    .into_iter()
    .collect()
}

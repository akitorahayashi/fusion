use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use toml::Value as TomlValue;

pub const DEFAULT_OLLAMA_HOST: &str = "127.0.0.1";
pub const DEFAULT_OLLAMA_PORT: u16 = 11434;
pub const DEFAULT_OLLAMA_MODEL: &str = "llama3.2:3b";
pub const DEFAULT_OLLAMA_SYSTEM_PROMPT: &str = "You are a helpful assistant.";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaServerConfig {
    #[serde(default = "default_ollama_host")]
    pub host: String,
    #[serde(default = "default_ollama_port")]
    pub port: u16,
    #[serde(default = "default_ollama_server_extra")]
    #[serde(flatten)]
    pub extra: BTreeMap<String, TomlValue>,
}

impl Default for OllamaServerConfig {
    fn default() -> Self {
        Self {
            host: default_ollama_host(),
            port: default_ollama_port(),
            extra: default_ollama_server_extra(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaRunConfig {
    #[serde(default = "default_ollama_model")]
    pub model: String,
    #[serde(default = "default_ollama_system_prompt")]
    pub system_prompt: String,
    #[serde(default = "default_ollama_temperature")]
    pub temperature: f32,
    #[serde(default = "default_stream_false")]
    pub stream: bool,
}

impl Default for OllamaRunConfig {
    fn default() -> Self {
        Self {
            model: default_ollama_model(),
            system_prompt: default_ollama_system_prompt(),
            temperature: default_ollama_temperature(),
            stream: default_stream_false(),
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

fn default_ollama_system_prompt() -> String {
    DEFAULT_OLLAMA_SYSTEM_PROMPT.to_string()
}

fn default_ollama_temperature() -> f32 {
    0.7
}

fn default_stream_false() -> bool {
    false
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

use super::openai::{http_error, run_for_mlx, run_for_ollama};
use crate::cli::{ServiceType, service_machine_name};
use crate::core::config;
use crate::error::AppError;
use reqwest::blocking::Client;
use std::time::Duration;

/// Overrides supplied via the CLI `run` command.
#[derive(Debug, Default, Clone)]
pub struct RunOverrides {
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub system: Option<String>,
}

pub fn handle_run(
    service_type: ServiceType,
    prompt: String,
    overrides: RunOverrides,
) -> Result<(), AppError> {
    let cfg = config::load_config()?;
    let service_name = service_machine_name(service_type);
    let client = Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|err| http_error(service_name, err))?;

    match service_type {
        ServiceType::Ollama => run_for_ollama(&cfg, &client, &prompt, &overrides),
        ServiceType::Mlx => run_for_mlx(&cfg, &client, &prompt, &overrides),
    }
}

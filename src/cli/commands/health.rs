use super::shared::{load_config, service_for_runtime};
use crate::cli::ServiceType;
use crate::core::config;
use crate::error::AppError;
use reqwest::blocking::Client;
use serde_json::json;
use std::time::Duration;

/// Allow a slightly longer timeout for inference (considering model load time)
const HEALTH_TIMEOUT_SECS: u64 = 30;

pub fn handle_health_single(service_type: ServiceType) -> Result<(), AppError> {
    let cfg = load_config()?;

    let service = service_for_runtime(&cfg, service_type)?;
    let model_name = match service_type {
        ServiceType::Ollama => cfg.ollama_server.model.clone(),
        ServiceType::Mlx => cfg.mlx_server.model.clone(),
    };

    let client = Client::builder()
        .timeout(Duration::from_secs(HEALTH_TIMEOUT_SECS))
        .build()
        .map_err(|e| AppError::process_error(service.name, format!("Client error: {e}")))?;

    let url = format!(
        "http://{}/v1/chat/completions",
        config::format_host_port(&service.host, service.port)
    );

    println!("🩺 Checking {} health (inference test) on {}...", service.name, url);
    println!("   Model: {}", model_name);
    println!("   Prompt: \"ping\"");

    let payload = json!({
        "model": model_name,
        "messages": [
            { "role": "user", "content": "ping" }
        ],
        "max_tokens": 10,
        "stream": false
    });

    let response =
        client.post(&url).json(&payload).send().map_err(|e| {
            AppError::process_error(service.name, format!("Connection failed: {e}"))
        })?;

    let status = response.status();
    if status.is_success() {
        println!("✅ {}: Healthy (Inference success)", service.name);
        Ok(())
    } else {
        let error_text =
            response.text().unwrap_or_else(|e| format!("<failed to read error body: {e}>"));
        Err(AppError::process_error(
            service.name,
            format!("Unhealthy. Status: {}, Error: {}", status, error_text),
        ))
    }
}

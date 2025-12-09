use crate::core::config;
use crate::core::services::ManagedService;
use crate::error::AppError;
use reqwest::blocking::Client;
use serde_json::json;
use std::time::Duration;

/// Sends a lightweight inference request to the specified service to check if it is ready.
pub fn check_inference_readiness(
    service: &ManagedService,
    model_name: &str,
    timeout_secs: u64,
) -> Result<(), AppError> {
    let client = Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| AppError::process_error(service.name, format!("Client build error: {e}")))?;

    let url = format!(
        "http://{}/v1/chat/completions",
        config::format_host_port(&service.host, service.port),
    );

    let payload = json!({
        "model": model_name,
        "messages": [
            { "role": "user", "content": "ping" }
        ],
        "max_tokens": 1,
        "stream": false,
    });

    let response =
        client.post(&url).json(&payload).send().map_err(|e| {
            AppError::process_error(service.name, format!("Connection failed: {e}"))
        })?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(AppError::process_error(
            service.name,
            format!("Service responded with status: {}", response.status()),
        ))
    }
}

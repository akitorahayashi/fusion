use crate::core::config;
use crate::core::services::ManagedService;
use crate::error::AppError;
use reqwest::blocking::Client;
use serde_json::json;
use std::time::Duration;

/// Sends an inference request and returns the generated text content.
pub fn query_inference(
    service: &ManagedService,
    model_name: &str,
    prompt: &str,
    timeout_secs: u64,
) -> Result<String, AppError> {
    let client = Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| AppError::process_error(service.name, format!("Client build error: {e}")))?;

    let url = format!(
        "http://{}/v1/chat/completions",
        config::format_host_port(&service.host, service.port),
    );

    // max_tokensを指定せず、モデルのデフォルトまたは十分な長さを許可する
    let payload = json!({
        "model": model_name,
        "messages": [
            { "role": "user", "content": prompt }
        ],
        "stream": false,
    });

    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .map_err(|e| AppError::process_error(service.name, format!("Connection failed: {e}")))?;

    if !response.status().is_success() {
        return Err(AppError::process_error(
            service.name,
            format!("Service responded with status: {}", response.status()),
        ));
    }

    let body: serde_json::Value = response.json().map_err(|e| {
        AppError::process_error(service.name, format!("Failed to parse JSON response: {e}"))
    })?;

    // OpenAI互換レスポンスから content を抽出
    body["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| {
            AppError::process_error(service.name, "Invalid response structure: missing content")
        })
}

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

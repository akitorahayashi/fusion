use super::ServiceType;
use crate::core::config::{
    self, Config, MlxRunConfig, MlxServerConfig, OllamaRunConfig, OllamaServerConfig,
};
use crate::core::paths;
use crate::core::process::{self, StartOutcome, StatusOutcome, StopOutcome};
use crate::core::services::{self, ManagedService};
use crate::error::AppError;
use reqwest::blocking::{Client, Response};
use serde::{Deserialize, Serialize};
use std::env;
use std::io::{self, BufRead, BufReader, Write};
use std::process::Command;
use std::time::Duration;

/// Overrides supplied via the CLI `run` command.
#[derive(Debug, Default, Clone)]
pub struct RunOverrides {
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub system: Option<String>,
}

/// Subcommands supported by `fusion <service> config`.
#[derive(Debug)]
pub enum ServiceConfigCommand {
    Show,
    Edit,
    Path,
    Set { key: String, value: String },
}

fn handle_service_up(service: ManagedService) -> Result<(), AppError> {
    match process::start_service(&service)? {
        StartOutcome::Started { .. } => {
            println!("• {} started on {}:{}", service.name, service.host, service.port);
        }
        StartOutcome::AlreadyRunning { .. } => {
            println!("• {} already running on {}:{}", service.name, service.host, service.port);
        }
    }
    Ok(())
}

fn handle_service_down(service: ManagedService, force: bool) -> Result<(), AppError> {
    match process::stop_service(&service, force)? {
        StopOutcome::Stopped { forced, .. } => {
            if forced {
                println!("• {} force-stopped on {}:{}", service.name, service.host, service.port);
            } else {
                println!("• {} stopped on {}:{}", service.name, service.host, service.port);
            }
        }
        StopOutcome::TerminatedByName { count, forced } => {
            let action = if forced { "killed with SIGKILL" } else { "signaled" };
            println!(
                "• {} stopped by signature on {}:{} ({} process{} {action})",
                service.name,
                service.host,
                service.port,
                count,
                if count == 1 { "" } else { "es" }
            );
        }
        StopOutcome::NotRunning => {
            println!("• {} is not running on {}:{}", service.name, service.host, service.port);
        }
    }
    Ok(())
}

fn handle_service_ps(service: ManagedService) -> Result<(), AppError> {
    match process::status_service(&service)? {
        StatusOutcome::Running { pid } => {
            println!(
                "• {}: running on {}:{} (pid {pid})",
                service.name, service.host, service.port
            );
        }
        StatusOutcome::NotRunning => {
            println!("• {}: not running on {}:{}", service.name, service.host, service.port);
        }
    }
    Ok(())
}

fn handle_service_logs(service: ManagedService) -> Result<(), AppError> {
    paths::ensure_pid_dir()?;
    println!("• {}: {}", service.name, service.log_path().display());
    Ok(())
}

fn service_label(service_type: ServiceType) -> &'static str {
    match service_type {
        ServiceType::Ollama => "Ollama",
        ServiceType::Mlx => "MLX",
    }
}

fn load_config() -> Result<Config, AppError> {
    config::load_config()
}

fn service_for_up(cfg: &Config, service_type: ServiceType) -> ManagedService {
    match service_type {
        ServiceType::Ollama => services::create_ollama_service(&cfg.ollama_server),
        ServiceType::Mlx => services::create_mlx_service(&cfg.mlx_server),
    }
}

fn service_for_runtime(
    cfg: &Config,
    service_type: ServiceType,
) -> Result<ManagedService, AppError> {
    match service_type {
        ServiceType::Ollama => services::load_ollama_service(&cfg.ollama_server),
        ServiceType::Mlx => services::load_mlx_service(&cfg.mlx_server),
    }
}

pub fn handle_up(service_type: ServiceType) -> Result<(), AppError> {
    println!("🚀 Starting {}...", service_label(service_type));
    let cfg = load_config()?;
    let service = service_for_up(&cfg, service_type);
    handle_service_up(service)
}

pub fn handle_down(service_type: ServiceType, force: bool) -> Result<(), AppError> {
    println!("🛑 Stopping {}...", service_label(service_type));
    let cfg = load_config()?;
    let service = service_for_runtime(&cfg, service_type)?;
    handle_service_down(service, force)
}

pub fn handle_ps_single(service_type: ServiceType) -> Result<(), AppError> {
    println!("ℹ️  {} status:", service_label(service_type));
    let cfg = load_config()?;
    let service = service_for_runtime(&cfg, service_type)?;
    handle_service_ps(service)
}

pub fn handle_logs_single(service_type: ServiceType) -> Result<(), AppError> {
    println!("📜 {} log location:", service_label(service_type));
    let cfg = load_config()?;
    let service = service_for_runtime(&cfg, service_type)?;
    handle_service_logs(service)
}

pub fn handle_ps() -> Result<(), AppError> {
    println!("ℹ️  Status for LLM runtimes:");
    let cfg = load_config()?;
    for service in services::default_services(&cfg)? {
        handle_service_ps(service)?;
    }
    Ok(())
}

pub fn handle_logs() -> Result<(), AppError> {
    println!("Log files:");
    let cfg = load_config()?;
    for service in services::default_services(&cfg)? {
        handle_service_logs(service)?;
    }
    println!("Use 'tail -f <log>' to follow output.");
    Ok(())
}

pub fn handle_run(
    service_type: ServiceType,
    prompt: String,
    overrides: RunOverrides,
) -> Result<(), AppError> {
    let cfg = load_config()?;
    let client = Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|err| http_error(service_label(service_type), err))?;

    match service_type {
        ServiceType::Ollama => {
            run_ollama(&client, &cfg.ollama_server, &cfg.ollama_run, &prompt, &overrides)
        }
        ServiceType::Mlx => run_mlx(&client, &cfg.mlx_server, &cfg.mlx_run, &prompt, &overrides),
    }
}

pub fn handle_config(
    service_type: ServiceType,
    command: ServiceConfigCommand,
) -> Result<(), AppError> {
    match command {
        ServiceConfigCommand::Show => show_config(),
        ServiceConfigCommand::Edit => edit_config(),
        ServiceConfigCommand::Path => print_config_path(),
        ServiceConfigCommand::Set { key, value } => set_config_value(service_type, key, value),
    }
}

fn show_config() -> Result<(), AppError> {
    let _ = config::load_config_document()?;
    let path = paths::user_config_file()?;
    let contents = std::fs::read_to_string(&path)?;
    print!("{}", contents);
    Ok(())
}

fn edit_config() -> Result<(), AppError> {
    let _ = config::load_config_document()?;
    let path = paths::user_config_file()?;
    let editor = env::var("EDITOR")
        .map_err(|_| AppError::config_error("$EDITOR is not set; cannot edit configuration"))?;
    let status = Command::new(editor)
        .arg(&path)
        .status()
        .map_err(|err| AppError::config_error(format!("Failed to launch editor: {err}")))?;
    if !status.success() {
        return Err(AppError::config_error("Editor exited with a non-zero status"));
    }
    Ok(())
}

fn print_config_path() -> Result<(), AppError> {
    let path = paths::user_config_file()?;
    println!("{}", path.display());
    Ok(())
}

fn set_config_value(service_type: ServiceType, key: String, value: String) -> Result<(), AppError> {
    let mut document = config::load_config_document()?;
    let segments: Vec<String> = key
        .split('.')
        .map(|segment| segment.trim().to_string())
        .filter(|segment| !segment.is_empty())
        .collect();
    if segments.is_empty() {
        return Err(AppError::config_error("Configuration key must not be empty"));
    }
    let refs: Vec<&str> = segments.iter().map(|segment| segment.as_str()).collect();
    let inferred = config::infer_toml_edit_value(&value);
    config::set_document_value(&mut document, &refs, inferred)?;
    config::save_config_document(&document)?;

    println!(
        "Updated {} configuration key '{}'",
        service_label(service_type).to_lowercase(),
        segments.join(".")
    );
    Ok(())
}

fn run_ollama(
    client: &Client,
    server_cfg: &OllamaServerConfig,
    run_cfg: &OllamaRunConfig,
    prompt: &str,
    overrides: &RunOverrides,
) -> Result<(), AppError> {
    let url = format!(
        "http://{}/api/generate",
        config::format_host_port(&server_cfg.host, server_cfg.port)
    );
    let model = overrides.model.as_deref().unwrap_or(run_cfg.model.as_str());
    let temperature = overrides.temperature.unwrap_or(run_cfg.temperature);
    let system = overrides.system.as_deref().unwrap_or(run_cfg.system_prompt.as_str());

    let payload = OllamaGenerateRequest {
        model: model.to_string(),
        prompt: prompt.to_string(),
        system: Some(system.to_string()),
        stream: run_cfg.stream,
        options: OllamaOptions { temperature },
    };

    let response =
        client.post(url).json(&payload).send().map_err(|err| http_error("ollama", err))?;
    let response = ensure_success("ollama", response)?;

    if run_cfg.stream {
        stream_ollama_response(response)?;
    } else {
        let completion: OllamaCompletion =
            response.json().map_err(|err| http_error("ollama", err))?;
        println!("{}", completion.response.trim_end());
    }
    Ok(())
}

fn stream_ollama_response(mut response: Response) -> Result<(), AppError> {
    let mut reader = BufReader::new(&mut response);
    let mut buffer = String::new();
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    loop {
        buffer.clear();
        if reader.read_line(&mut buffer)? == 0 {
            break;
        }
        let trimmed = buffer.trim();
        if trimmed.is_empty() {
            continue;
        }
        let chunk: OllamaStreamChunk = serde_json::from_str(trimmed).map_err(|err| {
            AppError::process_error("ollama", format!("Invalid stream chunk: {err}"))
        })?;
        if let Some(text) = chunk.response {
            handle.write_all(text.as_bytes())?;
            handle.flush()?;
        }
        if chunk.done.unwrap_or(false) {
            break;
        }
    }
    handle.write_all(b"\n")?;
    handle.flush()?;
    Ok(())
}

fn run_mlx(
    client: &Client,
    server_cfg: &MlxServerConfig,
    run_cfg: &MlxRunConfig,
    prompt: &str,
    overrides: &RunOverrides,
) -> Result<(), AppError> {
    let url = format!(
        "http://{}/v1/chat/completions",
        config::format_host_port(&server_cfg.host, server_cfg.port)
    );
    let model = overrides.model.as_deref().unwrap_or(run_cfg.model.as_str());
    let temperature = overrides.temperature.unwrap_or(run_cfg.temperature);
    let system = overrides.system.as_deref().unwrap_or(run_cfg.system_prompt.as_str());

    let payload = ChatCompletionRequest {
        model: model.to_string(),
        messages: vec![
            ChatMessage { role: "system".into(), content: system.to_string() },
            ChatMessage { role: "user".into(), content: prompt.to_string() },
        ],
        temperature,
        stream: run_cfg.stream,
    };

    let response = client.post(url).json(&payload).send().map_err(|err| http_error("mlx", err))?;
    let response = ensure_success("mlx", response)?;

    if run_cfg.stream {
        stream_mlx_response(response)?;
    } else {
        let completion: ChatCompletionResponse =
            response.json().map_err(|err| http_error("mlx", err))?;
        if let Some(choice) = completion.choices.into_iter().next()
            && let Some(message) = choice.message
        {
            println!("{}", message.content.trim_end());
        }
    }

    Ok(())
}

fn stream_mlx_response(mut response: Response) -> Result<(), AppError> {
    let mut reader = BufReader::new(&mut response);
    let mut buffer = String::new();
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    loop {
        buffer.clear();
        if reader.read_line(&mut buffer)? == 0 {
            break;
        }
        let trimmed = buffer.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "data: [DONE]" {
            break;
        }
        let json = trimmed.strip_prefix("data:").map(str::trim).unwrap_or(trimmed);
        let chunk: ChatStreamChunk = serde_json::from_str(json).map_err(|err| {
            AppError::process_error("mlx", format!("Invalid stream chunk: {err}"))
        })?;
        for choice in chunk.choices {
            if let Some(delta) = choice.delta
                && let Some(content) = delta.content
            {
                handle.write_all(content.as_bytes())?;
                handle.flush()?;
            }
            if let Some(message) = choice.message {
                handle.write_all(message.content.as_bytes())?;
                handle.flush()?;
            }
        }
    }

    handle.write_all(b"\n")?;
    handle.flush()?;
    Ok(())
}

fn ensure_success(service: &str, response: Response) -> Result<Response, AppError> {
    if response.status().is_success() {
        return Ok(response);
    }

    let status = response.status();
    let body = response.text().unwrap_or_else(|_| "<failed to read error body>".to_string());
    Err(AppError::process_error(
        service,
        format!("HTTP request failed with status {status}: {body}"),
    ))
}

fn http_error(service: &str, err: reqwest::Error) -> AppError {
    AppError::process_error(service, format!("HTTP error: {err}"))
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Debug, Deserialize)]
struct OllamaCompletion {
    response: String,
}

#[derive(Debug, Deserialize)]
struct OllamaStreamChunk {
    #[serde(default)]
    response: Option<String>,
    #[serde(default)]
    done: Option<bool>,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    #[serde(default)]
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    #[serde(default)]
    message: Option<ChatMessage>,
    #[serde(default)]
    delta: Option<ChatDelta>,
}

#[derive(Debug, Deserialize)]
struct ChatDelta {
    #[serde(default)]
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatStreamChunk {
    #[serde(default)]
    choices: Vec<ChatChoice>,
}

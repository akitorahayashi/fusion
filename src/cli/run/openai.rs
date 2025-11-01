use super::command::RunOverrides;
use crate::core::config::{self, Config};
use crate::error::AppError;
use reqwest::blocking::{Client, Response};
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, BufReader, Write};

pub(super) fn run_for_ollama(
    cfg: &Config,
    client: &Client,
    prompt: &str,
    overrides: &RunOverrides,
) -> Result<(), AppError> {
    let server_cfg = &cfg.ollama_server;
    let run_cfg = &cfg.ollama_run;
    let model = overrides.model.as_deref().unwrap_or(run_cfg.model.as_str());
    let temperature = overrides.temperature.unwrap_or(run_cfg.temperature);
    let system_prompt = overrides.system.as_deref().unwrap_or(run_cfg.system_prompt.as_str());

    run_openai_compatible(
        client,
        OpenAiRunArgs {
            host: &server_cfg.host,
            port: server_cfg.port,
            model,
            system_prompt,
            temperature,
            stream: run_cfg.stream,
            prompt,
            service_name: "ollama",
        },
    )
}

pub(super) fn run_for_mlx(
    cfg: &Config,
    client: &Client,
    prompt: &str,
    overrides: &RunOverrides,
) -> Result<(), AppError> {
    let server_cfg = &cfg.mlx_server;
    let run_cfg = &cfg.mlx_run;
    let model = overrides.model.as_deref().unwrap_or(run_cfg.model.as_str());
    let temperature = overrides.temperature.unwrap_or(run_cfg.temperature);
    let system_prompt = overrides.system.as_deref().unwrap_or(run_cfg.system_prompt.as_str());

    run_openai_compatible(
        client,
        OpenAiRunArgs {
            host: &server_cfg.host,
            port: server_cfg.port,
            model,
            system_prompt,
            temperature,
            stream: run_cfg.stream,
            prompt,
            service_name: "mlx",
        },
    )
}

pub(super) fn http_error(service: &str, err: reqwest::Error) -> AppError {
    AppError::process_error(service, format!("HTTP error: {err}"))
}

fn run_openai_compatible(client: &Client, args: OpenAiRunArgs<'_>) -> Result<(), AppError> {
    let url =
        format!("http://{}/v1/chat/completions", config::format_host_port(args.host, args.port));

    let payload = ChatCompletionRequest {
        model: args.model.to_string(),
        messages: vec![
            ChatMessage { role: "system".into(), content: args.system_prompt.to_string() },
            ChatMessage { role: "user".into(), content: args.prompt.to_string() },
        ],
        temperature: args.temperature,
        stream: args.stream,
    };

    let response =
        client.post(url).json(&payload).send().map_err(|err| http_error(args.service_name, err))?;
    let response = ensure_success(args.service_name, response)?;

    if args.stream {
        stream_openai_response(response, args.service_name)
    } else {
        let completion: ChatCompletionResponse =
            response.json().map_err(|err| http_error(args.service_name, err))?;
        if let Some(choice) = completion.choices.into_iter().next()
            && let Some(message) = choice.message
        {
            println!("{}", message.content.trim_end());
        }
        Ok(())
    }
}

fn stream_openai_response(mut response: Response, service_name: &str) -> Result<(), AppError> {
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
            AppError::process_error(service_name, format!("Invalid stream chunk: {err}"))
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

struct OpenAiRunArgs<'a> {
    host: &'a str,
    port: u16,
    model: &'a str,
    system_prompt: &'a str,
    temperature: f32,
    stream: bool,
    prompt: &'a str,
    service_name: &'a str,
}

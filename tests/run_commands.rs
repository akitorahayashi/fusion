mod common;

use common::CliTestContext;
use fusion::cli::{self, RunOverrides, ServiceType};
use fusion::core::config::{load_config, save_config};
use serde_json::Value;
use serial_test::serial;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone)]
struct CapturedRequest {
    line: String,
    body: Value,
}

#[test]
#[serial]
fn llm_ollama_run_uses_openai_payload() {
    let _ctx = CliTestContext::new();
    let listener = TcpListener::bind("127.0.0.1:0").expect("stub listener should bind");
    let port = listener.local_addr().unwrap().port();
    let capture: Arc<Mutex<Option<CapturedRequest>>> = Arc::new(Mutex::new(None));
    let capture_thread = {
        let capture = Arc::clone(&capture);
        thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept should succeed");
            let mut reader = BufReader::new(stream);
            let mut request_line = String::new();
            reader.read_line(&mut request_line).expect("read request line");

            let mut content_length = 0usize;
            loop {
                let mut header = String::new();
                reader.read_line(&mut header).expect("read header");
                if header.trim().is_empty() {
                    break;
                }
                let lower = header.to_ascii_lowercase();
                if let Some(value) = header.split(':').nth(1)
                    && lower.starts_with("content-length")
                {
                    content_length = value.trim().parse::<usize>().expect("parse content length");
                }
            }

            let mut body = vec![0u8; content_length];
            reader.read_exact(&mut body).expect("read body");
            let json: Value = serde_json::from_slice(&body).expect("valid JSON payload");
            *capture.lock().expect("capture lock poisoned") =
                Some(CapturedRequest { line: request_line.trim().to_string(), body: json });

            let response_body =
                br#"{"choices":[{"message":{"role":"assistant","content":"stubbed"}}]}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                response_body.len(),
                String::from_utf8_lossy(response_body)
            );
            reader.get_mut().write_all(response.as_bytes()).expect("write response");
            reader.get_mut().flush().ok();
        })
    };

    let mut cfg = load_config().expect("load_config should succeed");
    cfg.ollama_server.port = port;
    cfg.ollama_run.stream = false;
    save_config(&cfg).expect("save_config should succeed");

    let overrides = RunOverrides {
        model: Some("custom-model".into()),
        temperature: Some(0.5),
        system: Some("system message".into()),
    };

    cli::handle_run(ServiceType::Ollama, "Hello AI".into(), overrides).expect("run should succeed");

    capture_thread.join().expect("stub thread should join");
    let captured =
        capture.lock().expect("capture lock poisoned").clone().expect("request should be captured");

    assert_eq!(captured.line, "POST /v1/chat/completions HTTP/1.1");
    assert_eq!(captured.body["model"], "custom-model");
    assert_eq!(captured.body["temperature"], 0.5);
    assert_eq!(captured.body["stream"], false);
    let messages = captured.body["messages"].as_array().expect("messages should be an array");
    assert_eq!(messages.len(), 2, "system and user messages should be present");
    assert_eq!(messages[0]["role"], "system");
    assert_eq!(messages[0]["content"], "system message");
    assert_eq!(messages[1]["role"], "user");
    assert_eq!(messages[1]["content"], "Hello AI");
}

#[test]
#[serial]
fn llm_mlx_run_uses_openai_payload() {
    let _ctx = CliTestContext::new();
    let listener = TcpListener::bind("127.0.0.1:0").expect("stub listener should bind");
    let port = listener.local_addr().unwrap().port();
    let capture: Arc<Mutex<Option<CapturedRequest>>> = Arc::new(Mutex::new(None));
    let capture_thread = {
        let capture = Arc::clone(&capture);
        thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept should succeed");
            let mut reader = BufReader::new(stream);
            let mut request_line = String::new();
            reader.read_line(&mut request_line).expect("read request line");

            let mut content_length = 0usize;
            loop {
                let mut header = String::new();
                reader.read_line(&mut header).expect("read header");
                if header.trim().is_empty() {
                    break;
                }
                let lower = header.to_ascii_lowercase();
                if let Some(value) = header.split(':').nth(1)
                    && lower.starts_with("content-length")
                {
                    content_length = value.trim().parse::<usize>().expect("parse content length");
                }
            }

            let mut body = vec![0u8; content_length];
            reader.read_exact(&mut body).expect("read body");
            let json: Value = serde_json::from_slice(&body).expect("valid JSON payload");
            *capture.lock().expect("capture lock poisoned") =
                Some(CapturedRequest { line: request_line.trim().to_string(), body: json });

            let response_body =
                br#"{"choices":[{"message":{"role":"assistant","content":"mlx response"}}]}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                response_body.len(),
                String::from_utf8_lossy(response_body)
            );
            reader.get_mut().write_all(response.as_bytes()).expect("write response");
            reader.get_mut().flush().ok();
        })
    };

    let mut cfg = load_config().expect("load_config should succeed");
    cfg.mlx_server.port = port;
    cfg.mlx_run.stream = false;
    save_config(&cfg).expect("save_config should succeed");

    let overrides = RunOverrides {
        model: Some("mlx-custom-model".into()),
        temperature: Some(0.3),
        system: Some("mlx system".into()),
    };

    cli::handle_run(ServiceType::Mlx, "Hi MLX".into(), overrides).expect("run should succeed");

    capture_thread.join().expect("stub thread should join");
    let captured =
        capture.lock().expect("capture lock poisoned").clone().expect("request should be captured");

    assert_eq!(captured.line, "POST /v1/chat/completions HTTP/1.1");
    assert_eq!(captured.body["model"], "mlx-custom-model");
    assert_eq!(captured.body["temperature"], 0.3);
    assert_eq!(captured.body["stream"], false);
    let messages = captured.body["messages"].as_array().expect("messages should be an array");
    assert_eq!(messages.len(), 2, "system and user messages should be present");
    assert_eq!(messages[0]["content"], "mlx system");
    assert_eq!(messages[1]["content"], "Hi MLX");
}

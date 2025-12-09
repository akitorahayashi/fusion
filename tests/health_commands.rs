mod common;

use common::CliTestContext;
use fusion::cli::{self, ServiceType};
use fusion::core::config::{load_config, save_config};
use serial_test::serial;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::thread;

/// Helper function to test health check for a given service type
fn test_health_inference_request(service_type: ServiceType, expected_model: &str) {
    let _ctx = CliTestContext::new();
    let listener = TcpListener::bind("127.0.0.1:0").expect("stub listener should bind");
    let port = listener.local_addr().unwrap().port();
    let expected_model = expected_model.to_string();

    let stub_thread = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("accept should succeed");
        let mut reader = BufReader::new(stream);
        let mut request_line = String::new();
        reader.read_line(&mut request_line).expect("read request line");

        // Read headers to get content length
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

        // Read and validate body
        let mut body = vec![0u8; content_length];
        reader.read_exact(&mut body).expect("read body");
        let json: serde_json::Value = serde_json::from_slice(&body).expect("valid JSON payload");

        // Verify minimal inference payload
        assert_eq!(json["model"], expected_model.as_str());
        assert_eq!(json["max_tokens"], 1);
        assert_eq!(json["stream"], false);
        let messages = json["messages"].as_array().expect("messages should be an array");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[0]["content"], "ping");

        // Send success response
        let response_body = br#"{"choices":[{"message":{"role":"assistant","content":"pong"}}]}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            response_body.len(),
            String::from_utf8_lossy(response_body)
        );
        reader.get_mut().write_all(response.as_bytes()).expect("write response");
        reader.get_mut().flush().ok();
    });

    let mut cfg = load_config().expect("load_config should succeed");
    match service_type {
        ServiceType::Ollama => cfg.ollama_server.port = port,
        ServiceType::Mlx => cfg.mlx_server.port = port,
    }
    save_config(&cfg).expect("save_config should succeed");

    cli::handle_health_single(service_type).expect("health should succeed");

    stub_thread.join().expect("stub thread should join");
}

#[test]
#[serial]
fn llm_ollama_health_sends_inference_request() {
    test_health_inference_request(ServiceType::Ollama, "llama3.2:3b");
}

#[test]
#[serial]
fn llm_mlx_health_sends_inference_request() {
    test_health_inference_request(ServiceType::Mlx, "mlx-community/Llama-3.2-3B-Instruct-4bit");
}

#[test]
#[serial]
fn llm_health_returns_error_on_failure() {
    let _ctx = CliTestContext::new();
    let listener = TcpListener::bind("127.0.0.1:0").expect("stub listener should bind");
    let port = listener.local_addr().unwrap().port();

    let stub_thread = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("accept should succeed");
        let mut reader = BufReader::new(stream);

        // Read request
        let mut request_line = String::new();
        reader.read_line(&mut request_line).expect("read request line");
        loop {
            let mut header = String::new();
            reader.read_line(&mut header).expect("read header");
            if header.trim().is_empty() {
                break;
            }
        }

        // Send error response
        let response_body = br#"{"error": "model not found"}"#;
        let response = format!(
            "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            response_body.len(),
            String::from_utf8_lossy(response_body)
        );
        reader.get_mut().write_all(response.as_bytes()).expect("write response");
        reader.get_mut().flush().ok();
    });

    let mut cfg = load_config().expect("load_config should succeed");
    cfg.ollama_server.port = port;
    save_config(&cfg).expect("save_config should succeed");

    let result = cli::handle_health_single(ServiceType::Ollama);
    assert!(result.is_err(), "health should fail on HTTP error");

    stub_thread.join().expect("stub thread should join");
}

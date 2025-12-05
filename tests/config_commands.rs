mod common;

use common::CliTestContext;
use fusion::cli::{self, ServiceConfigCommand};
use fusion::core::config::load_config;

#[test]
#[serial_test::serial]
fn llm_config_show_works() {
    let _ctx = CliTestContext::new();
    // Ensure the config file exists before running the command.
    let _ = load_config().expect("load_config should succeed");

    cli::handle_config(ServiceConfigCommand::Show).expect("config show should succeed");
}

#[test]
#[serial_test::serial]
fn llm_config_reset_restores_defaults() {
    let _ctx = CliTestContext::new();

    // Modify the config
    let mut cfg = load_config().expect("load_config should succeed");
    cfg.ollama_server.port = 9999;
    cfg.mlx_server.model = "custom-model".to_string();
    fusion::core::config::save_config(&cfg).expect("save_config should succeed");

    // Verify the changes were saved
    let modified = load_config().expect("reload should succeed");
    assert_eq!(modified.ollama_server.port, 9999);
    assert_eq!(modified.mlx_server.model, "custom-model");

    // Reset the config
    cli::handle_config(ServiceConfigCommand::Reset).expect("config reset should succeed");

    // Verify it was reset to defaults
    let reset = load_config().expect("reload after reset should succeed");
    assert_eq!(reset.ollama_server.port, 11434); // default port
    assert_eq!(reset.mlx_server.model, "mlx-community/Llama-3.2-3B-Instruct-4bit"); // default model
}

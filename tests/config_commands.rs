mod common;

use common::CliTestContext;
use fusion::cli::{self, ServiceConfigCommand, ServiceType};
use fusion::core::config::load_config;

#[test]
#[serial_test::serial]
fn llm_config_set_updates_file() {
    let _ctx = CliTestContext::new();
    // Ensure the config file exists before running the command.
    let _ = load_config().expect("load_config should succeed");

    cli::handle_config(
        ServiceType::Ollama,
        ServiceConfigCommand::Set { key: "ollama_server.port".into(), value: "22222".into() },
    )
    .expect("config set should succeed");

    let updated = load_config().expect("reload should succeed");
    assert_eq!(updated.ollama_server.port, 22222);
}

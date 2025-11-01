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

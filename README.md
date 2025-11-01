# Fusion

A Rust CLI tool for managing local Ollama and MLX runtimes on **macOS**.

## Overview

Fusion is a Rust CLI that manages local Ollama and MLX runtimes for development. It handles service
startup, shutdown, status reporting, and now includes a first-class prompt runner that talks to the
managed HTTP APIs directly. All behaviour is driven by a persistent TOML configuration file rather
than ad-hoc environment variables, making the tool predictable across shells and sessions.

## Getting Started

```bash
# Install dependencies and build
cargo build

# Print CLI help
cargo run -- --help

# Start the managed runtimes
cargo run -- ollama up
cargo run -- mlx up
```

### Binary installation

```bash
cargo install --path .
fusion ollama up
fusion mlx up
```

## Configuration

Fusion stores all runtime settings in `~/.config/fusion/config.toml` (or the platform-equivalent using `dirs::config_dir()`). The file is created on first use with sensible defaults and can be managed via the CLI:

```bash
fusion config show             # dump the current file
fusion config path             # print the path to config.toml
fusion config edit             # create symlink to edit
```

The configuration file contains sections for both services:

```toml
[ollama_server]
host = "127.0.0.1"
port = 11434

[ollama_run]
model = "llama3.2:3b"
system_prompt = "You are a helpful assistant."
temperature = 0.8
stream = false

[mlx_server]
host = "127.0.0.1"
port = 8080
model = "mlx-community/Llama-3.2-3B-Instruct-4bit"

[mlx_run]
model = "mlx-community/Llama-3.2-3B-Instruct-4bit"
system_prompt = "You are a helpful assistant."
temperature = 0.7
stream = false
```

Logs, PID files, and runtime state are stored under each service's directory in `~/.config/fusion/<service>/`.
Override the project root for tests by setting `FUSION_PROJECT_ROOT`; the config location can be redirected
with `FUSION_CONFIG_DIR`.

## CLI Usage

```text
fusion ollama up
fusion ollama down [--force]
fusion ollama ps
fusion ollama log
fusion ollama run <prompt> [--model <name>] [--temperature <value>] [--system <prompt>]

fusion mlx up
fusion mlx down [--force]
fusion mlx ps
fusion mlx log
fusion mlx run <prompt> [--model <name>] [--temperature <value>] [--system <prompt>]

# global commands
fusion ps
fusion config <show|edit|path>
```

The `run` subcommand issues an HTTP request to the managed runtime using the defaults from
`config.toml`, merging any CLI overrides for the model, system prompt, or temperature. Both
services speak the OpenAI-compatible `/v1/chat/completions` API, so the CLI sends identical payloads
and reuses the same streaming logic regardless of backend. The `config` family offers read/write
access without leaving the terminal.

## Testing

The project mirrors the original testing culture:

- **Core unit tests** live next to modules in `src/core/`, covering configuration persistence,
  service construction, and process lifecycle helpers.
- **Integration tests** in `tests/llm_commands.rs` drive the CLI entry points with a mock process
  driver and lightweight HTTP stubs, ensuring command wiring, configuration updates, and prompt
  execution stay consistent without launching real runtimes.

Run the full suite with:

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

## Project Structure

- `src/core/paths.rs` – project root, PID directory, and config file resolution
- `src/core/config.rs` – strongly-typed TOML configuration management
- `src/core/services.rs` – `ManagedService` definitions plus config-driven loaders
- `src/core/process.rs` – PID/log helpers and pluggable process driver
- `src/cli/commands/` – lifecycle and configuration command handlers for managed runtimes
- `src/cli/run/` – shared OpenAI-compatible run pipeline reused by each managed service
- `tests/service_lifecycle.rs` – integration tests for service up/down/ps/log operations
- `tests/run_commands.rs` – integration tests for run command execution and payload validation
- `tests/config_commands.rs` – integration tests for configuration management

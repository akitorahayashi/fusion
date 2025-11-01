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

Fusion stores service-specific configuration in separate TOML files under `~/.config/fusion/`:
- `~/.config/fusion/ollama/config.toml` - Ollama configuration
- `~/.config/fusion/mlx/config.toml` - MLX configuration

Each service maintains its own logs, PID files, and runtime state in its respective directory. The files
are created on first use with sensible defaults and can be managed via the CLI:

```bash
fusion ollama config show             # dump the current file
fusion ollama config path             # print the path to config.toml
fusion ollama config edit             # create symlink to edit
fusion ollama config set ollama_run.temperature 0.6
```

Each service has separate configuration:

```toml
# ~/.config/fusion/ollama/config.toml
[ollama_server]
host = "127.0.0.1"
port = 11434

# ~/.config/fusion/mlx/config.toml
[mlx_server]
host = "127.0.0.1"
port = 8080
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
fusion ollama config <show|edit|path|set>

fusion mlx up
fusion mlx down [--force]
fusion mlx ps
fusion mlx log
fusion mlx run <prompt> [--model <name>] [--temperature <value>] [--system <prompt>]
fusion mlx config <show|edit|path|set>

# global helpers across all services
fusion ps
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

## Overview

The Fusion CLI is a Rust reimplementation of the original Typer-based tool that orchestrates local
LLM runtimes for `menv` development. It manages the lifecycle of two services:

- **Ollama** – configured through `OLLAMA_*` and `FUSION_OLLAMA_HOST` variables.
- **MLX** – configured through `FUSION_MLX_MODEL` and `FUSION_MLX_PORT` (defaults match the Python CLI).

Managed processes log to the project `.tmp` directory and expose the same `.env` knobs as the legacy
tool, letting existing workflows carry over unchanged.

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

Fusion automatically loads a `.env` file located at the project root or pointed to by
`FUSION_ENV_FILE`. The following variables mirror the Python implementation:

| Variable | Purpose | Default |
| --- | --- | --- |
| `FUSION_OLLAMA_HOST` / `OLLAMA_HOST` | Bind address for `ollama serve` | `127.0.0.1:11434` |
| `OLLAMA_*` keys | Additional Ollama tuning parameters | See `src/core/env.rs` |
| `FUSION_MLX_HOST` | Bind address for `mlx_lm.server` | `127.0.0.1` |
| `FUSION_MLX_MODEL` | MLX model identifier | `mlx-community/Llama-3.2-3B-Instruct-4bit` |
| `FUSION_MLX_PORT` | MLX server port | `8080` |

Logs and PID files are written to `<project-root>/.tmp`. You can override the target location for
tests or tooling by setting `FUSION_PROJECT_ROOT`.

## CLI Usage

```text
fusion ollama up [--host <IP>] [--port <PORT>]
fusion ollama down [--force]
fusion ollama ps
fusion ollama logs

# (aliases)
fusion ol up

fusion mlx up [--host <IP>] [--port <PORT>]
fusion mlx down [--force]
fusion mlx ps
fusion mlx logs

# global helpers across all services
fusion ps
fusion logs
```

The `--host` and `--port` flags override the environment-driven defaults for each service, making it easy to bind Ollama or MLX to a different interface temporarily. When no overrides are provided, both runtimes listen on loopback (`127.0.0.1`).

All commands surface human-friendly console output and reuse the same messaging as the Python CLI.

## Testing

The project mirrors the original testing culture:

- **Core unit tests** live next to modules in `src/core/`, covering environment parsing, service
  definitions, and PID/file management.
- **Integration tests** in `tests/llm_commands.rs` drive the CLI entry points with a mock process
  driver, ensuring command wiring and messaging stay consistent without launching real runtimes.

Run the full suite with:

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

## Project Structure

- `src/core/paths.rs` – project root and `.tmp` resolution
- `src/core/env.rs` – `.env` loading and configuration defaults
- `src/core/services.rs` – `ManagedService` definitions for Ollama and MLX plus config-driven loaders
- `src/core/process.rs` – PID/log helpers and pluggable process driver
- `src/cli/llm.rs` – shared `ServiceType`-driven handlers consumed by `src/main.rs`
- `tests/llm_commands.rs` – integration coverage using the mock driver

Refer to `fusion-prev/` for the original Python implementation when verifying feature parity. Remove
that directory only after the Rust port has been fully validated in your environment.

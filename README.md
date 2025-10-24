# Fusion CLI (Rust)

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
cargo run -- llm --help

# Start the default runtimes
cargo run -- llm up
```

### Binary installation

```bash
cargo install --path .
fusion llm up
```

## Configuration

Fusion automatically loads a `.env` file located at the project root or pointed to by
`FUSION_ENV_FILE`. The following variables mirror the Python implementation:

| Variable | Purpose | Default |
| --- | --- | --- |
| `FUSION_OLLAMA_HOST` / `OLLAMA_HOST` | Bind address for `ollama serve` | `0.0.0.0:11434` |
| `OLLAMA_*` keys | Additional Ollama tuning parameters | See `src/core/env.rs` |
| `FUSION_MLX_MODEL` | MLX model identifier | `mlx-community/Llama-3.2-3B-Instruct-4bit` |
| `FUSION_MLX_PORT` | MLX server port | `8080` |

Logs and PID files are written to `<project-root>/.tmp`. You can override the target location for
tests or tooling by setting `FUSION_PROJECT_ROOT`.

## CLI Usage

```text
fusion llm up            # start Ollama and MLX
fusion llm down          # stop both services (graceful SIGTERM)
fusion llm down --force  # force stop using SIGKILL
fusion llm ps            # report whether services are running
fusion llm logs          # print log file locations
```

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
- `src/core/services.rs` – `ManagedService` definitions for Ollama and MLX
- `src/core/process.rs` – PID/log helpers and pluggable process driver
- `src/cli/llm.rs` – user-facing command handlers used by `src/main.rs`
- `tests/llm_commands.rs` – integration coverage using the mock driver

Refer to `fusion-prev/` for the original Python implementation when verifying feature parity. Remove
that directory only after the Rust port has been fully validated in your environment.

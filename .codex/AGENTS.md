# Fusion Development Overview

## Project Name
fusion

## Project Summary
Fusion is a Rust CLI that manages local Ollama and MLX runtimes for development. It handles service startup, shutdown, status reporting, and now includes a first-class prompt runner that talks to the managed HTTP APIs directly. All behaviour is driven by a persistent TOML configuration file rather than ad-hoc environment variables, making the tool predictable across shells and sessions.

## Platform Support
**macOS-only** - This project is designed exclusively for macOS systems and does not support Windows or Linux.

## Tech Stack
- **Language**: Rust (Edition 2024)
- **Key Libraries**:
  - clap (4.5) - Command line argument parser
  - dirs (5.0) - Standard directories
  - reqwest (0.12) - HTTP client
  - serde (1.0) - Serialization
  - serde_json (1.0) - JSON handling
  - sysinfo (0.30) - System information
  - toml (0.8) - TOML parser
  - toml_edit (0.22) - TOML editing
- **Dev Dependencies**:
  - assert_cmd (2.0) - Command assertion testing
  - assert_fs (1.1) - Filesystem assertions
  - predicates (3.1) - Value predicates
  - serial_test (3.1) - Serial test execution
  - tempfile (3.10) - Temporary files

## Coding Standards
- **Formatter**: rustfmt with max_width = 100, use_small_heuristics = "Max", use_field_init_shorthand = true
- **Linter**: clippy with msrv = "1.90.0", cognitive-complexity-threshold = 25, too-many-arguments-threshold = 7, type-complexity-threshold = 250, single-char-binding-names-threshold = 4

## Naming Conventions
- **Functions and Variables**: snake_case (e.g., `handle_service_command`, `service_type`)
- **Structs, Enums, Traits**: PascalCase (e.g., `Cli`, `Commands`, `ServiceType`)
- **Constants**: SCREAMING_SNAKE_CASE (not extensively used in visible code)
- **Modules**: snake_case (e.g., `cli`, `core`)

## Key Commands
- **Build**: `cargo build`
- **Run**: `cargo run -- <args>`
- **Test**: `cargo test`
- **Format**: `cargo fmt --all`
- **Lint**: `cargo clippy --all-targets -- -D warnings`
- **Install**: `cargo install --path .`

## Testing Strategy
- **Framework**: Built-in Rust testing with cargo test
- **Unit Tests**: Located next to modules in `src/core/`
- **Integration Tests**: In `tests/` directory, using assert_cmd for CLI testing
- **CI**: GitHub Actions workflows run tests on Ubuntu with `cargo test --all-targets --all-features`
- **Coverage**: Not specified, but full suite run with RUST_TEST_THREADS=1
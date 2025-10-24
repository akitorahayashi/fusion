# rs-cli-tmpl

`rs-cli-tmpl` is a reference template for building Rust-based command line tools with a clean,
layered architecture. It demonstrates how to separate concerns across the CLI interface,
application commands, pure core logic, and I/O abstractions so new projects can start from a
well-tested foundation.

## Architectural Highlights

- **Three-tier structure** &mdash; `src/main.rs` handles CLI parsing, `src/commands.rs` wires
  dependencies and user messaging, and `src/core/` keeps business rules testable via the
  `Execute` trait.
- **I/O abstraction** &mdash; `src/storage.rs` defines a `Storage` trait and a `FilesystemStorage`
  implementation rooted at `~/.config/rs-cli-tmpl`, making it easy to swap storage backends.
- **Robust testing strategy** &mdash; unit tests live next to their modules, `src/core/test_support.rs`
  offers a `MockStorage` for core logic tests, and the `tests/` directory provides integration
  suites for both the library API and the CLI binary.

The template ships with minimal sample commands (`add`, `list`, and `delete`) that show how to
thread dependencies through each layer. Replace or extend them with your own domain logic while
reusing the same structure.

## Storage Layout

The template stores items under `~/.config/rs-cli-tmpl/<id>/item.txt`. For example, after running `rs-cli-tmpl add my-item --content '...'`:

```text
~/.config/rs-cli-tmpl/
  my-item/
    item.txt
```

## Quick Start

```bash
cargo install --path .
# or
cargo build --release
```

The optimized binary will be created at `target/release/rs-cli-tmpl`.

## Development Commands

- `cargo build` &mdash; build a debug binary.
- `cargo build --release` &mdash; build the optimized release binary.
- `cargo fmt` &mdash; format code using rustfmt.
- `cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings` &mdash; format check and lint with clippy.
- `RUST_TEST_THREADS=1 cargo test --all-targets --all-features` &mdash; run all tests.
- `cargo fetch --locked` &mdash; pre-fetch dependencies.

## Testing Culture

- **Unit Tests**: Live alongside their modules inside `src/`, covering helper utilities and
  filesystem boundaries.
- **Core Logic Tests**: Use the mock storage in `src/core/test_support.rs` to exercise the
  command implementations without touching the real filesystem.
- **Integration Tests**: Located in the `tests/` directory. Separate crates cover the public
  library API (`tests/commands_api.rs`) and CLI workflows (`tests/cli_commands.rs`,
  `tests/cli_flow.rs`). Shared fixtures live in `tests/common/mod.rs`.

Run `cargo test` regularly&mdash;filesystem-heavy tests rely on the `serial_test` crate to avoid race
conditions.

## Adapting the Template

1. Replace the sample commands in `src/core/` with your own business logic.
2. Extend `src/commands.rs` to wire new dependencies and expose public APIs.
3. Update the CLI definitions in `src/main.rs` to match your command surface.
4. Refresh the integration tests and documentation to describe the new behavior.

Happy hacking!

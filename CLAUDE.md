# Starship Development Guide

## Build & Test Commands
- Build: `cargo check --workspace --locked`
- Build (all features): `cargo check --workspace --locked --all-features`
- Lint: `cargo clippy --all-targets --all-features`
- Format: `cargo fmt --all` and `dprint fmt` (for non-Rust files)
- Run all tests: `cargo test --all-features --locked --workspace`
- Run single test: `cargo test test_function_name`
- Run module tests: `cargo test --package starship --test module_name`
- Test with ignored: `cargo test -- --include-ignored`

## Code Style Guidelines
- Follow standard Rust naming conventions (snake_case for variables/functions, CamelCase for types)
- Use `crate::utils::create_command` instead of `std::process::Command::new` directly
- For file paths, use `crate::utils::context_path()` to create absolute paths
- Return `Option<Module>` from module functions
- Use `?` operator for error propagation
- Create module tests using `ModuleRenderer` structure
- Ensure proper mocking of commands, environment variables, and file systems in tests
- Format code with rustfmt (default settings)

Run `cargo test --doc` to verify documentation examples work correctly.
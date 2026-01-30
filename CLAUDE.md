# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

lazypueue is a lazygit-style terminal UI for pueue task management, written in Rust. It provides an interactive TUI for managing and monitoring pueue tasks with keyboard-driven workflows.

## Development Environment

This project uses **Nix flakes** for reproducible development environments. All development should be done within the Nix development shell.

### Essential Commands

```bash
# Enter development environment
nix develop

# Build the project
cargo build

# Build with Nix
nix build

# Run the application
cargo run

# Run with Nix
nix run

# Run tests
cargo test

# Run linter (clippy)
cargo clippy

# Check code without building
cargo check

# Format code
cargo fmt

# Format check (CI)
cargo fmt --check
```

## Architecture

### Technology Stack

- **TUI Framework**: ratatui (v0.28) - Modern terminal UI framework
- **Terminal Backend**: crossterm (v0.28) - Cross-platform terminal manipulation
- **Async Runtime**: tokio - For async communication with pueue daemon
- **Pueue Integration**: pueue-lib - Client library for pueue daemon communication
- **CLI Parsing**: clap with derive macros
- **Error Handling**: anyhow for application errors, thiserror for library errors

### Code Structure

The application follows a typical Rust TUI architecture:

1. **main.rs**: Entry point, terminal setup/teardown, main event loop
2. **Terminal Setup**: Uses crossterm for raw mode, alternate screen, and mouse capture
3. **Event Loop**: Processes keyboard/mouse events and redraws UI
4. **Rendering**: ratatui's declarative widget system for UI composition

### TUI Pattern

The application uses ratatui's immediate-mode rendering pattern:
- State is maintained in application structs
- Each frame, the entire UI is redrawn based on current state
- Events trigger state updates, which are reflected in the next frame

### Pueue Integration

- Uses `pueue-lib` for communication with the pueue daemon
- Communicates via the pueue daemon URI (configurable via `--uri` flag)
- Async operations handled by tokio runtime

## Nix-Specific Notes

### Flake Structure

- **flake.nix**: Defines development shell and build outputs
- **rust-overlay**: Provides stable Rust toolchain with rust-analyzer and rust-src
- **devShell**: Includes all build dependencies and development tools

### Updating Dependencies

When adding new Rust dependencies:
1. Add to Cargo.toml
2. Run `cargo build` to update Cargo.lock
3. Ensure Cargo.lock is committed (required for nix build)

When adding new system dependencies:
1. Add to `buildInputs` in flake.nix for runtime dependencies
2. Add to `nativeBuildInputs` for build-time tools

## Code Conventions

### Rust Style

- Follow standard Rust formatting (`cargo fmt`)
- Address all clippy warnings before committing
- Use `anyhow::Result` for main application code
- Use `thiserror` for defining custom error types in library code

### Module Organization

As the project grows, organize code into modules:
- `app/` - Application state and main logic
- `ui/` - UI components and rendering
- `events/` - Event handling
- `pueue/` - Pueue client wrapper and communication

### Error Handling

- Use `anyhow::Result` with context for better error messages
- Provide helpful error messages for common issues (daemon not running, connection failures)
- Gracefully handle terminal errors during cleanup

## Testing

- Unit tests should be colocated with code (`#[cfg(test)] mod tests`)
- Integration tests go in `tests/` directory
- Use `pretty_assertions` for better test output
- Mock pueue daemon responses for testing UI logic

## Build Optimization

The release profile in Cargo.toml is optimized for binary size and performance:
- LTO enabled for smaller binaries
- Single codegen unit for better optimization
- Symbols stripped for smaller size
- Opt-level 3 for maximum performance

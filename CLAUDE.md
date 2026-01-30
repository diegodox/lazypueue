# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

lazypueue is a lazygit-style terminal UI for pueue task management, written in Rust. It provides an interactive TUI for managing and monitoring pueue tasks with keyboard-driven workflows.

## Development Environment

This project uses **Nix flakes** for reproducible development environments. All development should be done within the Nix development shell.

**Important:** The development environment automatically includes `pueue` and starts the `pueued` daemon when you enter the shell. This means you don't need to install pueue separately for development.

### Essential Commands

```bash
# Enter development environment (automatically starts pueued if not running)
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

### Pueue Development Commands

The dev environment includes pueue for testing:

```bash
# Add test tasks
pueue add -- sleep 10
pueue add -- echo "Hello"
pueue add -- python -c "for i in range(100): print(i)"

# Check pueue status
pueue status

# View task logs
pueue log <task_id>

# Stop project daemon
pueue shutdown
```

### Project-Local Pueue Daemon

This project uses a **project-specific pueue daemon** that is completely isolated from any system-wide pueue installation.

**Directory Structure:**
```
.pueue/
├── pueue.yml         # Project-local configuration
├── data/             # Task state and metadata
├── logs/             # Task output logs
└── runtime/          # Unix socket for daemon communication
```

**Key Points:**
- The daemon is automatically started when you enter `nix develop`
- All task data is stored in `.pueue/` directory
- Socket file is at `.pueue/runtime/pueue.socket`
- No conflicts with system-wide or other project daemons
- Configuration can be customized in `.pueue/pueue.yml`

**Environment Variable:**
- `PUEUE_CONFIG_PATH` is automatically set to `$(pwd)/.pueue/pueue.yml`
- This tells pueue/pueued to use the project-local config

**Daemon Lifecycle:**
- Daemon starts automatically in `nix develop` shell
- Daemon persists after exiting shell (reconnectable)
- Stop with: `pueue shutdown`
- Daemon state persists across restarts (stored in `.pueue/data/`)

**Troubleshooting:**
```bash
# Check if daemon is running
pueue status

# View daemon socket
ls -la .pueue/runtime/pueue.socket

# Manually start daemon (if stopped)
pueued -c .pueue/pueue.yml -d

# Stop daemon
pueue shutdown

# Clean all task data (nuclear option)
rm -rf .pueue/{data,logs,runtime}
```

**Customization:**
Edit `.pueue/pueue.yml` to customize:
- Parallel task limits
- Shell command
- Environment variables for all tasks
- Callback hooks
- Time display formats

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

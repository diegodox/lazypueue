# lazypueue

A lazygit-style terminal UI for [pueue](https://github.com/Nukesor/pueue) task management.

## About

lazypueue provides an interactive terminal interface for managing pueue tasks, inspired by lazygit and lazydocker. Navigate and control your task queue with intuitive keyboard shortcuts.

## Features

- 🚀 Interactive TUI for pueue task management
- ⌨️  Keyboard-driven workflow
- 📊 Real-time task status updates
- 🎨 Clean, intuitive interface

## Prerequisites

- Nix with flakes enabled
- [pueue](https://github.com/Nukesor/pueue) daemon running (automatically installed in dev environment)

## Development

This project uses Nix flakes for reproducible development environments. The development shell automatically includes `pueue` and starts a **project-local pueued daemon** that is isolated from any system-wide pueue installation.

### Enter development shell

```bash
nix develop
```

This will:
- Set up the Rust toolchain
- Install pueue for testing
- Automatically start a **project-specific pueue daemon**
- Configure daemon to use `.pueue/` directory for all data

**Note:** The daemon is project-local, meaning:
- Task queue is isolated to this project
- No conflicts with other pueue instances
- All data stored in `.pueue/` directory
- Can be customized via `.pueue/pueue.yml`

### Build

```bash
# In nix develop shell
cargo build

# Or directly with nix
nix build
```

### Run

```bash
# In nix develop shell
cargo run

# Or directly with nix
nix run
```

### Testing

```bash
# Add some test tasks
pueue add -- sleep 10
pueue add -- echo "Hello from pueue"

# Run lazypueue
cargo run

# Run unit tests
cargo test
```

### Linting and Formatting

```bash
# Run clippy
cargo clippy

# Format code
cargo fmt
```

## Usage

```bash
lazypueue [OPTIONS]

Options:
  -u, --uri <URI>  Pueue daemon URI
  -h, --help       Print help
```

## Keyboard Shortcuts

(To be implemented)

- `q` - Quit
- `?` - Show help

## License

MIT OR Apache-2.0

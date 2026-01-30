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

- [pueue](https://github.com/Nukesor/pueue) daemon running
- Nix with flakes enabled

## Development

This project uses Nix flakes for reproducible development environments.

### Enter development shell

```bash
nix develop
```

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

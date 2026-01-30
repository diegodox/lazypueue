{
  description = "A lazygit-style TUI for pueue task management";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
        ];

        buildInputs = with pkgs; [
          # Add any system libraries needed here
        ];

        devInputs = with pkgs; [
          pueue  # Task queue manager for development/testing
        ];

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = buildInputs ++ devInputs;
          inherit nativeBuildInputs;

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

          shellHook = ''
            echo "🚀 lazypueue development environment"
            echo "Rust version: $(rustc --version)"
            echo ""

            # Set up project-local pueue configuration
            export PUEUE_CONFIG_PATH="$(pwd)/.pueue/pueue.yml"

            # Create pueue directories if they don't exist
            mkdir -p .pueue/{data,logs,runtime}

            # Check if project-specific daemon is already running
            PROJECT_SOCKET=".pueue/runtime/pueue.socket"

            # Quick check: daemon process is running for this project
            if pgrep -f "pueued.*$(pwd)/.pueue" > /dev/null 2>&1; then
              echo "✓ Project-local pueue daemon already running"
            else
              # Clean up stale socket if it exists
              [ -S "$PROJECT_SOCKET" ] && rm -f "$PROJECT_SOCKET" 2>/dev/null || true

              echo "📋 Starting project-local pueue daemon..."
              # Start daemon with project-specific config (daemonize and detach)
              nohup pueued -c "$PUEUE_CONFIG_PATH" > /dev/null 2>&1 &
              disown

              # Brief wait for socket creation
              for i in 1 2 3 4 5; do
                if [ -S "$PROJECT_SOCKET" ]; then
                  echo "✓ Project-local pueue daemon started"
                  echo "   Config: .pueue/pueue.yml"
                  echo "   Socket: $PROJECT_SOCKET"
                  break
                fi
                sleep 0.1
              done

              if [ ! -S "$PROJECT_SOCKET" ]; then
                echo "⚠ Daemon starting... (run 'pueue status' to verify)"
              fi
            fi

            echo ""
            echo "Available commands:"
            echo "  cargo build       - Build the project"
            echo "  cargo run         - Run the TUI"
            echo "  cargo test        - Run tests"
            echo "  cargo clippy      - Run linter"
            echo "  cargo fmt         - Format code"
            echo ""
            echo "Pueue commands (project-local):"
            echo "  pueue add -- <cmd> - Add a task to test with"
            echo "  pueue status       - Check pueue status"
            echo "  pueue log <id>     - View task logs"
            echo "  pueue shutdown     - Stop the project daemon"
          '';
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "lazypueue";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          inherit nativeBuildInputs buildInputs;
        };
      }
    );
}

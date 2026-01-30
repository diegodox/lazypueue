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
            echo "Pueue version: $(pueue --version 2>/dev/null | head -1 || echo 'not found')"
            echo ""

            # Set up project-local pueue configuration
            export PUEUE_CONFIG_PATH="$(pwd)/.pueue/pueue.yml"

            # Create pueue directories if they don't exist
            mkdir -p .pueue/{data,logs,runtime}

            # Check if project-specific daemon is already running
            PROJECT_SOCKET=".pueue/runtime/pueue.socket"

            # Quick check: socket exists and daemon process is running
            if [ -S "$PROJECT_SOCKET" ] && pgrep -f "pueued.*$(pwd)/.pueue" > /dev/null 2>&1; then
              echo "✓ Project-local pueue daemon already running"
            else
              # Clean up stale socket if it exists
              [ -S "$PROJECT_SOCKET" ] && rm -f "$PROJECT_SOCKET"

              echo "📋 Starting project-local pueue daemon..."
              # Start daemon with project-specific config
              if pueued -c "$PUEUE_CONFIG_PATH" -d > /dev/null 2>&1; then
                # Give daemon a moment to start
                sleep 0.5

                # Verify it started
                if [ -S "$PROJECT_SOCKET" ]; then
                  echo "✓ Project-local pueue daemon started"
                  echo "   Config: .pueue/pueue.yml"
                  echo "   Socket: $PROJECT_SOCKET"
                else
                  echo "⚠ Daemon may not have started properly"
                  echo "   Run 'pueue status' to check"
                fi
              else
                echo "⚠ Failed to start pueue daemon"
                echo "   Try manually: pueued -c .pueue/pueue.yml -d"
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

#!/usr/bin/env just --justfile
set quiet := true
set unstable := true

default:
    just --list

[doc('Build dailies')]
build:
    #!/usr/bin/env bash

    if command -v nix >/dev/null 2>&1; then
      echo "🔧 Building with nix..."
      nix build
      mkdir -p target/release
      cp -v result/bin/dailies target/release/dailies
    else
      echo "🔧 Building with cargo..."

      # Check if rustc and just are available
      for cmd in rustc cargo; do
        if ! command -v $cmd >/dev/null 2>&1; then
          echo "❌ Missing required command: $cmd" >&2
          exit 1
        fi
      done

      cargo build --release
    fi

[doc('Linting')]
lint:
    cargo clippy --all -- -D warnings
    nixfmt flake.nix

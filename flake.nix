{
  inputs = {
    oxalica.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs";
  };

  outputs =
    {
      self,
      flake-utils,
      nixpkgs,
      oxalica,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import oxalica) ];
        };

        rustPackage = pkgs.rustPlatform.buildRustPackage {
          pname = "dailies";
          version = "0.1.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
        };
      in
      {
        devShell = pkgs.mkShell {
          RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
          shellHook = ''
            export CARGO_TARGET_DIR="$(git rev-parse --show-toplevel)/target_dirs/nix_rustc";
          '';
          nativeBuildInputs = with pkgs; [
            alejandra
            pkg-config
            (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
            just
            nil
            nixfmt-rfc-style
          ];
        };

        packages.dailies = rustPackage;
        defaultPackage = rustPackage;
      }
    );
}

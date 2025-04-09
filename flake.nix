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
          CMAKE_LLVM_DIR = "${pkgs.llvmPackages.libllvm.dev}/lib/cmake/llvm";
          CMAKE_CLANG_DIR = "${pkgs.llvmPackages.libclang.dev}/lib/cmake/clang";
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          LD_LIBRARY_PATH = "${pkgs.stdenv.cc.cc.lib}/lib";
          RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
          nativeBuildInputs = with pkgs; [
            alejandra
            pkg-config
            rust-analyzer
            rustc
            clippy
            rustfmt
            cargo
            just
          ];
        };

        packages.dailies = rustPackage;
        defaultPackage = rustPackage;
      }
    );
}

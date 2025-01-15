{
  description = "Modified Wasmtime with state transition chaining support";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" ];
        };

        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
          cmake
          python3
          ninja
        ];

        buildInputs = with pkgs; [
          openssl
          curl
        ] ++ lib.optionals stdenv.isDarwin [
          darwin.apple_sdk.frameworks.Security
          darwin.apple_sdk.frameworks.SystemConfiguration
        ];

      in
      {
        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs buildInputs;

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          
          # Ensure the development shell has the right environment variables
          shellHook = ''
            export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
            export RUST_BACKTRACE=1
          '';
        };

        # You can add more outputs here for distributable builds
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "wasmtime-chaining";
          version = "0.1.0";
          src = ./.;

          inherit nativeBuildInputs buildInputs;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          # Enable all features by default
          buildFeatures = [ "all" ];

          meta = with pkgs.lib; {
            description = "Modified Wasmtime runtime with state transition chaining support";
            homepage = "https://github.com/colinrozzi/wasmtime-chaining";
            license = licenses.asl20;
            maintainers = [ "Colin Rozzi" ];
          };
        };
      });
}

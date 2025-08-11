{
  description = "DWN-RS: Decentralized Web Node Implementation in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };


        # Rust toolchain with WebAssembly support
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        # Native dependencies
        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
          openssl
          protobuf
        ];

        # Runtime dependencies
        buildInputs = with pkgs; [
          openssl
          zlib
          libiconv
        ] ++ lib.optionals stdenv.isDarwin [
          darwin.apple_sdk.frameworks.Security
          darwin.apple_sdk.frameworks.CoreFoundation
          darwin.apple_sdk.frameworks.SystemConfiguration
        ];

        # WebAssembly-specific dependencies
        wasmBuildInputs = with pkgs; [
          wasm-pack
          nodejs_20
          nodePackages.npm
          binaryen
        ];

        # Common environment variables
        commonEnv = {
          RUST_BACKTRACE = "1";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        };

        # WebAssembly-specific environment
        wasmEnv = commonEnv // {
          RUSTFLAGS = "--cfg=web_sys_unstable_apis";
        };

      in
      {
        packages = {
          # Default package - native build
          default = self.packages.${system}.dwn-rs;

          # Native build of the entire workspace
          dwn-rs = pkgs.rustPlatform.buildRustPackage {
            pname = "dwn-rs";
            version = "0.1.0";

            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
              allowBuiltinFetchGit = true;
            };

            inherit nativeBuildInputs buildInputs;

            # Skip tests that require network or external services
            checkFlags = [
              "--skip=test_remote"
              "--skip=surrealdb"
            ];

            # Build all workspace members
            cargoBuildFlags = [ "--workspace" ];

            # Set environment variables
            RUST_BACKTRACE = "1";
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

            meta = with pkgs.lib; {
              description = "Decentralized Web Node implementation in Rust";
              homepage = "https://github.com/enmand/dwn-rs";
              license = licenses.asl20;
              maintainers = [ ];
            };
          };

          # Core library only
          dwn-rs-core = pkgs.rustPlatform.buildRustPackage {
            pname = "dwn-rs-core";
            version = "0.1.0";

            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
              allowBuiltinFetchGit = true;
            };

            inherit nativeBuildInputs buildInputs;

            cargoBuildFlags = [ "-p" "dwn-rs-core" ];

            RUST_BACKTRACE = "1";
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          };

          # WebAssembly build
          dwn-rs-wasm = pkgs.stdenv.mkDerivation {
            pname = "dwn-rs-wasm";
            version = "0.1.0";

            src = ./.;

            nativeBuildInputs = nativeBuildInputs ++ wasmBuildInputs;

            buildPhase = ''
              export HOME=$TMPDIR
          
              # Build the WebAssembly package
              cd crates/dwn-rs-wasm
              wasm-pack build --target web --out-dir ../../pkg --scope dwn-rs
          
              # Optimize with binaryen
              wasm-opt -Oz -o ../../pkg/dwn_rs_wasm_bg.wasm ../../pkg/dwn_rs_wasm_bg.wasm
            '';

            installPhase = ''
              mkdir -p $out
              cp -r pkg/* $out/
            '';

            inherit wasmEnv;
          };

          # Browser-ready WebAssembly package
          dwn-rs-wasm-browser = pkgs.stdenv.mkDerivation {
            pname = "dwn-rs-wasm-browser";
            version = "0.1.0";

            src = ./.;

            nativeBuildInputs = nativeBuildInputs ++ wasmBuildInputs;

            buildPhase = ''
              export HOME=$TMPDIR
          
              # Build for browsers specifically
              cd crates/dwn-rs-wasm
              wasm-pack build --target web --out-dir browsers --features "browser"
          
              # Copy pre-built browsers output if it exists
              if [ -d "browsers" ]; then
                cp -r browsers ../../
              fi
            '';

            installPhase = ''
              mkdir -p $out
              if [ -d "browsers" ]; then
                cp -r browsers/* $out/
              fi
            '';

            inherit wasmEnv;
          };
        };

        # Development shell
        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.packages.${system}.dwn-rs ];

          buildInputs = nativeBuildInputs ++ buildInputs ++ wasmBuildInputs ++ (with pkgs; [
            # Development tools
            cargo-watch
            cargo-edit
            cargo-audit
            cargo-deny
            cargo-outdated

            # Documentation tools
            mdbook

            # Database tools (for SurrealDB integration)
            surrealdb

            # Additional utilities
            jq
            curl

            # WebAssembly testing tools
            chromedriver
            geckodriver
          ]);

          shellHook = ''
            echo "🦀 DWN-RS Development Environment"
            echo "Rust version: $(rustc --version)"
            echo "Cargo version: $(cargo --version)"
            echo "wasm-pack version: $(wasm-pack --version)"
            echo ""
            echo "Available commands:"
            echo "  cargo build                    # Build native version"
            echo "  cargo test                     # Run tests"
            echo "  cargo build --target wasm32-unknown-unknown  # Build for WebAssembly"
            echo "  wasm-pack build crates/dwn-rs-wasm --target web  # Build WASM package"
            echo ""
            echo "Workspace members:"
            echo "  - dwn-rs-core"
            echo "  - dwn-rs-stores"
            echo "  - dwn-rs-remote" 
            echo "  - dwn-rs-wasm"
            echo "  - dwn-rs-message-derive"
        
            # Set up environment variables
            export RUST_BACKTRACE=1
            export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig"
            export RUSTFLAGS="--cfg=web_sys_unstable_apis"
          '';

          inherit (commonEnv) RUST_BACKTRACE PKG_CONFIG_PATH;
          RUSTFLAGS = "--cfg=web_sys_unstable_apis";
        };

        # Formatter
        formatter = pkgs.nixpkgs-fmt;

        # Apps for easy execution
        apps = {
          default = {
            type = "app";
            program = "${self.packages.${system}.dwn-rs}/bin/dwn-rs";
          };
        };

        # Checks for CI/CD
        checks = {
          dwn-rs-clippy = pkgs.rustPlatform.buildRustPackage {
            pname = "dwn-rs-clippy";
            version = "0.1.0";

            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
              allowBuiltinFetchGit = true;
            };

            inherit nativeBuildInputs buildInputs;

            buildPhase = ''
              cargo clippy --workspace --all-targets -- -D warnings
            '';

            installPhase = ''
              mkdir -p $out
              echo "Clippy check passed" > $out/clippy-result
            '';

            RUST_BACKTRACE = "1";
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          };

          dwn-rs-fmt = pkgs.rustPlatform.buildRustPackage {
            pname = "dwn-rs-fmt";
            version = "0.1.0";

            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
              allowBuiltinFetchGit = true;
            };

            inherit nativeBuildInputs;

            buildPhase = ''
              cargo fmt --all -- --check
            '';

            installPhase = ''
              mkdir -p $out
              echo "Format check passed" > $out/fmt-result
            '';
          };
        };
      });
}

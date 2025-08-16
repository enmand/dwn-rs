{
  description = "DWN-RS: Decentralized Web Node Implementation in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, fenix, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };

        # Rust toolchain with WebAssembly support using fenix
        rustToolchain = with fenix.packages.${system}; combine [
          stable.rustc
          stable.cargo
          stable.rustfmt
          stable.clippy
          stable.rust-src
          targets.wasm32-unknown-unknown.stable.rust-std
        ];

        nativeBuildInputs = with pkgs; [
          rustToolchain
          cargo-make
          pkg-config
          openssl
          protobuf
        ];

        buildInputs = with pkgs; [
          openssl
          zlib
          libiconv
        ];

        wasmBuildInputs = with pkgs; [
          wasm-pack
          nodejs_20
          nodePackages.npm
          binaryen
        ];

        commonVersion = "0.1.0";
        commonCargoLock = {
          lockFile = ./Cargo.lock;
          allowBuiltinFetchGit = true;
        };

        commonEnv = {
          RUST_BACKTRACE = "1";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        };

        # WebAssembly-specific environment
        wasmEnv = commonEnv // {
          RUSTFLAGS = "--cfg=web_sys_unstable_apis";
          CC_wasm32_unknown_unknown = "${pkgs.llvmPackages.clang-unwrapped}/bin/clang";
          CXX_wasm32_unknown_unknown = "${pkgs.llvmPackages.clang-unwrapped}/bin/clang++";
          AR_wasm32_unknown_unknown = "${pkgs.llvmPackages.bintools-unwrapped}/bin/llvm-ar";
          CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "${pkgs.lld}/bin/lld";
          CFLAGS_wasm32_unknown_unknown = "--target=wasm32-unknown-unknown -isystem ${pkgs.llvmPackages.clang}/resource-root/include";
          CXXFLAGS_wasm32_unknown_unknown = "--target=wasm32-unknown-unknown -isystem ${pkgs.llvmPackages.clang}/resource-root/include";
          LDFLAGS_wasm32_unknown_unknown = "-L${pkgs.llvmPackages.libunwind}/lib -lunwind";
        };

        # Common package metadata
        commonMeta = with pkgs.lib; {
          homepage = "https://github.com/enmand/dwn-rs";
          license = licenses.asl20;
          maintainers = [ ];
        };

        # LLVM packages for WASM builds
        llvmPackages = with pkgs.llvmPackages; [
          bintools-unwrapped
          clang-unwrapped
          lld
          libllvm
        ];

        # Helper function to create Rust packages
        mkRustPackage = { pname, description, cargoBuildFlags ? [ ], checkFlags ? [ ] }:
          pkgs.rustPlatform.buildRustPackage {
            inherit pname cargoBuildFlags checkFlags;
            version = commonVersion;
            src = self;
            cargoLock = commonCargoLock;
            inherit nativeBuildInputs buildInputs;
            inherit (commonEnv) RUST_BACKTRACE PKG_CONFIG_PATH;
            meta = commonMeta // { inherit description; };
          };

        # Helper function to create WebAssembly packages
        mkWasmPackage = { pname, description, target ? "web", outDir ? "../../pkg", extraBuildCommands ? "" }:
          pkgs.stdenv.mkDerivation {
            inherit pname;
            version = commonVersion;
            src = self;

            nativeBuildInputs = nativeBuildInputs ++ wasmBuildInputs ++ llvmPackages;

            buildPhase = ''
              export HOME=$TMPDIR
              ${pkgs.lib.concatStringsSep "\n" (pkgs.lib.mapAttrsToList (name: value: "export ${name}=\"${toString value}\"") wasmEnv)}
              
              cargo make build-wasm -- --out-dir ${outDir}
              ${extraBuildCommands}
            '';

            installPhase = ''
              mkdir -p $out
              cp -r ${if outDir == "../../pkg" then "pkg" else builtins.baseNameOf outDir}/* $out/
            '';

            meta = commonMeta // { inherit description; };
          };

        # Helper function to create check packages  
        mkCheck = { pname, description, command, installResult }:
          pkgs.rustPlatform.buildRustPackage {
            inherit pname;
            version = commonVersion;
            src = self;
            cargoLock = commonCargoLock;
            inherit nativeBuildInputs buildInputs;
            inherit (commonEnv) RUST_BACKTRACE PKG_CONFIG_PATH;

            buildPhase = command;
            installPhase = ''
              mkdir -p $out
              echo "${installResult}" > $out/result
            '';

            meta = commonMeta // { inherit description; };
          };

      in
      {
        packages = {
          # Default package - native build
          default = self.packages.${system}.dwn-rs;

          # Native build of the entire workspace
          dwn-rs = mkRustPackage {
            pname = "dwn-rs";
            description = "Decentralized Web Node implementation in Rust";
            cargoBuildFlags = [ "--workspace" ];
            checkFlags = [
              "--skip=test_remote"
              "--skip=surrealdb"
            ];
          };

          # Core library only
          dwn-rs-core = mkRustPackage {
            pname = "dwn-rs-core";
            description = "DWN-RS core library";
            cargoBuildFlags = [ "-p" "dwn-rs-core" ];
          };

          # WebAssembly build
          dwn-rs-wasm = mkWasmPackage {
            pname = "dwn-rs-wasm";
            description = "DWN-RS WebAssembly package";
            extraBuildCommands = ''
              # Optimize with binaryen
              wasm-opt -Oz -o ../../pkg/dwn_rs_wasm_bg.wasm ../../pkg/dwn_rs_wasm_bg.wasm
            '';
          };

          # Browser-ready WebAssembly package
          dwn-rs-wasm-browser = mkWasmPackage {
            pname = "dwn-rs-wasm-browser";
            description = "DWN-RS WebAssembly package for browsers";
            target = "web";
            outDir = "browsers";
            extraBuildCommands = ''
              # Build with browser features
              wasm-pack build --target web --out-dir browsers --features "browser"
              
              # Copy pre-built browsers output if it exists
              if [ -d "browsers" ]; then
                cp -r browsers ../../
              fi
            '';
          };
        };

        # Development shells
        devShells = {
          # Default development shell (native development)
          default = pkgs.mkShell {
            inputsFrom = [ self.packages.${system}.dwn-rs ];

            buildInputs = nativeBuildInputs ++ buildInputs ++ wasmBuildInputs ++ llvmPackages ++ (with pkgs; [
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

              chromedriver
              geckodriver
            ]);

            shellHook = ''
              echo "🦀 DWN-RS Development Environment"
              echo "Rust version: $(rustc --version)"
              echo "Cargo version: $(cargo --version)"
              echo ""
              echo "Available commands:"
              echo "  cargo build                    # Build native version"
              echo "  cargo test                     # Run tests"
              echo ""
              echo "Workspace members:"
              echo "  - dwn-rs-core"
              echo "  - dwn-rs-stores"
              echo "  - dwn-rs-remote" 
              echo "  - dwn-rs-wasm"
              echo "  - dwn-rs-message-derive"
          
              # Set up environment variables
              ${pkgs.lib.concatStringsSep "\n" (pkgs.lib.mapAttrsToList (name: value: "export ${name}=\"${toString value}\"") wasmEnv)}
            '';

            # Additional shell-specific environment variables
          };
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
          dwn-rs-clippy = mkCheck {
            pname = "dwn-rs-clippy";
            description = "Clippy linting check for DWN-RS";
            command = "cargo clippy --workspace --all-targets -- -D warnings";
            installResult = "Clippy check passed";
          };

          dwn-rs-fmt = mkCheck {
            pname = "dwn-rs-fmt";
            description = "Format check for DWN-RS";
            command = "cargo fmt --all -- --check";
            installResult = "Format check passed";
          };
        };
      });
}

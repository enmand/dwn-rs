# vim: set ft=make :
set shell := ["sh", "-c"]
set allow-duplicate-recipes
set positional-arguments
set export

export cc := env_var_or_default("CC", "/opt/homebrew/opt/llvm/bin/clang")
export ar := env_var_or_default("AR", "/opt/homebrew/opt/llvm/bin/llvm-ar")
macos_compile := if os() == "macos" { if arch() == "aarch64" { "CC={{ $cc }} AR={{ ar }}" } else { "" } } else { "" }

build:
	{{macos_compile}} wasm-pack build --release --target nodejs --out-name index --out-dir out -- --no-default-features -F wasm

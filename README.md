dwn-store-surrealdb-rs
====

dwn-store-surrealdb-rs is a Rust-based DWN store implementation that can be
used with [dwn-sdk-js](https://github.com/TBD54566975/dwn-sdk-js).


# Compiling

## Web Assembly

_Note:_ On an M1-based Mac you may need to install LLVM from Homebrew, and use that
Clang compiler, instead of the built-in XCode Clang compiler.

`wasm-pack build --release --target bunlder --out-name index --out-dir dist`

For M1-based Macs run with the Homebrew-based LLVM:

`CC=/opt/homebrew/opt/llvm/bin/clang AR=/opt/homebrew/opt/llvm/bin/llvm-ar wasm-pack build --release --target bundler --out-name index --out-dir dist`

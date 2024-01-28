dwn-rs
====

dwn-rs is a Rust-based DWN implementation that can be
used with [dwn-sdk-js](https://github.com/TBD54566975/dwn-sdk-js).

# Compiling

This project uses [cargo-make](https://sagiegurari.github.io/cargo-make/). To
install it, run:

```bash
$ cargo install cargo-make
```

## Web Assembly

To compile to Web Assembly, run:

```bash
$ cargo make build-wasm
```

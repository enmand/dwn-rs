# dwn-rs-remote

A Rust library for interacting with remote Decentralized Web Node (DWN) instances using JSON APIs.

## Overview

`dwn-rs-remote` is a new crate designed to simplify the process of interacting with remote DWN instances. This crate provides a single,
unified API for sending messages to remote DWN instances and receiving responses. It supports the `processMessage` function,
which will interact with a JSON API offered by e.g. `dwn-server`.

## Features

-   **Simple API**: `dwn-rs-remote` provides a straightforward API for interacting with remote DWN instances.
-   **JSON API support**: The crate supports the JSON API offered by `dwn-server`, allowing developers to interact with remote DWN instances.

## Requirements

-   **dwn-rs** (or `dwn-rs-core`): This crate depends on the `dwn-rs` crate, which provides the core data structures and functions for interacting with DWN services.

## Usage

To use `dwn-rs-remote`, add the following dependency to your `Cargo.toml`:

```toml
[dependencies]
dwn-rs-remote = "0.1.0"
```

Then, you can use the `processMessage` function to interact with remote DWN instances. Here's an example:

```rust
use dwn_rss_remote::{process_message, RemoteDWNInstance};

let instance = RemoteDWNInstance::new("https://example.com/dwn");
let message = serde_json::json!({"type": "message", "data": "Hello, world!"});
let tenant = "did:persona"
let response = process_message(&tenant, &instance, message);

if let Some(response) = response {
    println!("Received response: {}", response);
} else {
    println!("Error processing message");
}
```

## Contributing

Contributions to `dwn-rs-remote` are welcome. Please see the [README file](https://github.com/enmand/dwn-rs) for more information on how to contribute to the project.

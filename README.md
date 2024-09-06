# dwn-rs 🌐🚀

dwn-rs is a Rust-based implementation of the Decentralized Web Node (DWN) specification. This library provides the core traits and implementations necessary to interact with DWN services, designed to be used by other libraries and applications. It can be compiled for WebAssembly for use in web browsers or run natively.

## Project Description 📝

dwn-rs aims to facilitate the development of decentralized web applications and services by providing a robust and easy-to-use DWN core library. Key features include:

- Core DWN types and traits
- Compatibility with WebAssembly and native environments
- Integration with the [dwn-sdk-js](https://github.com/TBD54566975/dwn-sdk-js) for JavaScript interoperability

## Requirements 🛠️

- Rust 1.75.0+
- [cargo-make](https://sagiegurari.github.io/cargo-make/)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

To install `cargo-make`, run:

```bash
$ cargo install cargo-make
```

To install `wasm-pack`, run:

```bash
$ cargo install wasm-pack
```

## Building and Running 🚧

### Building for WebAssembly 🌐

To compile the project for WebAssembly, execute the following command:

```bash
$ cargo make build-wasm
```

### Running in Pure Rust 🦀 (WIP)

To build and run the project in a native Rust environment, use:

```bash
$ cargo build
$ cargo run
```

## Integration with dwn-sdk-js 🌐🗄️

Once you have compiled the project to WebAssembly, you can use the generated `.wasm` file in your `dwn-sdk-js` project to implement a store. Here's how to do it:

1. **Compile the Rust project to WASM**:

Make sure you have built the WASM file using the following command:

```bash
$ cargo make build-wasm
```

2. **Copy the WASM file to your `dwn-sdk-js` project**:

The compiled WASM file is usually located in the `crates/dwn-rs-wasm/pks/` directory. Copy this file to an appropriate location in your `dwn-sdk-js` project.

3. **Initialize the WASM stores in your JavaScript code**:

Use the following code to load and utilize the WASM file in your JavaScript project:

```typescript
import {
  SurrealDataStore,
  SurrealMessageStore,
  SurrealEventLog,
  SurrealResumableTaskStore,
  EventStream,
  init_tracing,
  TracingLevel,
} from "../pkg/index.js";

// only needed in Node environments
import WebSocket from "isomorphic-ws";
global.WebSocket = WebSocket;

// only required if you want logging/tracing to the console
init_tracing(TracingLevel.Error);

let messageStore = new SurrealMessageStore();
await messageStore.connect("mem://");

let dataStore = new SurrealDataStore();
await dataStore.connect("mem://");

let eventLog = new SurrealEventLog();
await eventLog.connect("mem://");

let resumableTaskStore = new SurrealResumableTaskStore();
await t.connect("mem://");

let eventStream = new EventStream();

const dwn = await Dwn.create({
  messageStore,
  dataStore,
  eventLog,
  resumableTaskStore,
  eventStream,
});
```

To connect to a remove SurrealDB server, use a connection string in the form: `ws://<username>:<password>@<host>:<port>/<namespace>`.
The auth type (`root` or `namespace`) can be given with `?auth=<type>`. `dwn-rs` requires the full namespace for tenanted operations.

4. **Run your project**:

Ensure that your project is set up to serve the WASM file correctly and run your project as usual.

## Roadmap 🗺️

Here's a simple roadmap with checkmarks tracking our progress against the DWN specification and companion guide:

### Core Features

- [x] Implement core DWN types and traits
- [x] Compatibility with WebAssembly
- [x] Integration with dwn-sdk-js
  - [x] Message store
  - [x] Data store
  - [x] Event Log
  - [x] Event Stream
- [ ] DWN Message processing
- [ ] JSONAPI service
- [ ] Light "DWN" proxy

### Functional Requirements

- [ ] Data sharing protocols

## Contributions 🤝

We welcome contributions from the community! Please check out our contribution guidelines in the `CONTRIBUTING.md` file and feel free to open issues or submit pull requests.

## License 📜

This project is licensed under the Apache License. See the `LICENSE` file for more details.

## Acknowledgements 🙏

Thanks to the Decentralized Identity Foundation (DIF) and the contributors to the DWN specification and companion guide for their invaluable resources and guidance.

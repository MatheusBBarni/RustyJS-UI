# RustyJS-UI

RustyJS-UI is an experimental desktop UI runtime that lets JavaScript describe a native interface while Rust handles rendering and event dispatch.

The goal is to give developers a simple, declarative JavaScript API for building native desktop apps, while keeping the rendering layer fast and memory-safe in Rust.

## Purpose

This project explores a split architecture:

- JavaScript owns application state and produces a virtual UI tree.
- Rust receives that tree, converts it into typed data, and renders it with `iced`.
- Native UI events are sent back to JavaScript through callback IDs, which trigger state updates and another render pass.

Instead of crossing the JS/Rust boundary for every widget call, the runtime serializes the UI tree into a single payload and sends it across the bridge.

## Current MVP

The current implementation includes:

- An embedded JavaScript runtime powered by `boa_engine`
- A VDOM-style bridge between JavaScript and Rust
- Native rendering through `iced`
- Basic components:
  - `View`
  - `Text`
  - `Button`
  - `TextInput`
- Style support for common layout and text fields
- Callback-based event dispatch from Rust back into JavaScript

## How It Works

At a high level, the flow looks like this:

1. JavaScript calls `App.run(...)`.
2. The runtime sends an `INIT_WINDOW` payload to Rust.
3. The JavaScript `render` function returns a tree of nodes such as `View`, `Text`, and `Button`.
4. That tree is serialized into an `UPDATE_VDOM` payload.
5. Rust deserializes the payload into typed UI nodes and renders them with `iced`.
6. When the user clicks a button or changes an input, Rust sends the callback ID back into the JS runtime.
7. JavaScript updates state and calls `App.requestRender()`.

## Running The Example

There is a simple example in [examples/hello_world_counter.js](examples/hello_world_counter.js) that renders `Hello world` and an increment button.

Run it directly:

```powershell
cargo run -- examples/hello_world_counter.js
```

Or use the helper script:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run-example.ps1
```

## Running Tests

```powershell
cargo test
```

## Project Layout

- [src/runtime](src/runtime) contains the embedded JS runtime and bootstrap script
- [src/bridge.rs](src/bridge.rs) defines the bridge payloads
- [src/vdom.rs](src/vdom.rs) defines the wire format and typed UI nodes
- [src/ui.rs](src/ui.rs) renders typed nodes into `iced` widgets
- [examples](examples) contains runnable JavaScript examples
- [tests](tests) contains integration tests for the bridge

## Status

This is still an MVP. The bridge, rendering flow, example loading, and integration tests are in place, but the project is still early and intended for experimentation rather than production use.

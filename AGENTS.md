# AGENTS.md

## Role

Act as a senior software engineer. Make precise, minimal changes that preserve the current architecture unless the task explicitly calls for a deeper refactor.

## Project Summary

RustyJS-UI is an experimental desktop UI runtime where JavaScript describes the UI tree and Rust renders it natively with `iced`.

Core flow:

1. JavaScript bootstraps through the embedded Boa runtime.
2. JS emits bridge payloads such as `INIT_WINDOW` and `UPDATE_VDOM`.
3. Rust deserializes those payloads into typed structures.
4. `iced` renders the native UI.
5. Native events are sent back into JS by callback ID.
6. JS updates state and requests another render.

## Repository Map

- `src/main.rs`: desktop app bootstrap, `iced` application loop, event processing, async polling.
- `src/runtime/`: embedded Boa runtime, module loading, async fetch transport, JS bootstrap integration.
- `src/bridge.rs`: bridge payloads, event payloads, window config parsing.
- `src/vdom.rs`: wire-node and typed UI tree definitions/conversions.
- `src/ui.rs`: native rendering of typed nodes with `iced`.
- `src/style.rs`: translation layer for style/layout values.
- `src/modal.rs`: modal host and overlay behavior.
- `tests/`: integration tests around runtime behavior, bridge payloads, routing, fetch, modules, and widgets.
- `tests/fixtures/`: JS fixture apps used by module-loading and bridge tests.
- `examples/`: runnable example apps for manual validation.
- `scripts/`: helper scripts for running examples and the login app.
- `docs/`: PRDs and product notes. Treat as design context, not source of truth over code/tests.

## Default Working Style

- Read the touched area before editing.
- Prefer focused fixes over broad cleanup.
- Keep public behavior stable unless the task explicitly changes it.
- Preserve existing naming and file organization.
- Add or update tests when runtime behavior, bridge payloads, module loading, or rendering logic changes.
- Use `anyhow` context on fallible IO/runtime paths so failures remain debuggable.

## Architecture Rules

When changing the JS-to-Rust contract, keep the full pipeline in sync:

- JS bootstrap/runtime surface
- bridge payload parsing in `src/bridge.rs`
- wire/tree types in `src/vdom.rs`
- native rendering in `src/ui.rs`
- tests and examples that exercise the changed behavior

When adding a new component, prop, or style capability, update all relevant layers instead of patching only one side.

Important invariants:

- The bridge is payload-driven. Avoid ad hoc host-side state that bypasses `BridgePayload`.
- Typed conversion should fail clearly on invalid wire data instead of silently coercing unexpected input.
- Module loading must stay inside the app entry root and preserve current ESM restrictions.
- Async work must continue to integrate through the runtime queue and `poll_async()` path.
- Modal behavior should preserve overlay semantics and not break normal root rendering.

## Testing Expectations

Use the smallest command that proves the change, then broaden if needed:

- `cargo test`
- `cargo test <test_name>`
- `cargo run -- examples/hello_world_counter.js`
- `cargo run -- examples/router_demo/main.js`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run-example.ps1`

Testing guidance:

- For bridge/runtime regressions, prefer or extend `tests/js_bridge_*.rs`.
- For ESM/module resolution work, use `tests/fixtures/esm`.
- For UI primitives, validate both typed tree behavior and rendered event flow where practical.
- If behavior changes user-facing examples, update the closest example and README references.

## Change Heuristics

- For runtime/bootstrap work, inspect `src/runtime/mod.rs` and the embedded JS bootstrap together.
- For event/callback issues, trace from `src/ui.rs` to `EventPayload` handling in `src/main.rs` and back into `JsRuntime::trigger_callback`.
- For layout/styling issues, inspect `src/style.rs`, `src/vdom.rs`, and `src/ui.rs` as a group.
- For route or module issues, review both the runtime loader and the dedicated tests before changing behavior.

## Avoid

- Do not introduce broad refactors without a clear need.
- Do not weaken path-safety or module-boundary checks.
- Do not edit unrelated docs/examples as incidental cleanup.
- Do not rely on manual testing alone for bridge/runtime changes when an automated test can be added.

## Completion Checklist

Before finishing:

1. Confirm the affected architectural layers are consistent.
2. Run the relevant tests or examples when the change affects behavior.
3. Note any tests not run and any remaining risk.

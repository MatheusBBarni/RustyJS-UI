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
- Native ESM support for local multi-file `.js` apps
- A VDOM-style bridge between JavaScript and Rust
- Native rendering through `iced`
- JS-first path routing with in-memory history via `App.createRouter`
- Built-in DX helpers via `Navigation`, `Storage`, `Timer`, and runtime diagnostics
- Basic components:
  - `View`
  - `Text`
  - `Button`
  - `TextInput`
  - `SelectInput` / `NativeSelect`
  - `FlatList` / `NativeList` (native scrollable list with JS-side item expansion)
  - `NativeCombobox` (higher-level helper built from `TextInput` + `SelectInput`)
  - `Tabs` (controlled tab helper)
  - `Modal` (native overlay rendered above the current tree)
  - `Alert` (imperative helper that shows a modal alert with title, description, and two action buttons)
  - `Toast` (timer-backed notification helper)
- Style support for common layout and text fields
- Callback-based event dispatch from Rust back into JavaScript

## How It Works

At a high level, the flow looks like this:

1. JavaScript calls `App.run(...)`.
2. The runtime sends an `INIT_WINDOW` payload to Rust.
3. The JavaScript `render` function returns a tree of nodes such as `View`, `Text`, `Button`, `TextInput`, `SelectInput`, and `Modal`.
4. That tree is serialized into an `UPDATE_VDOM` payload.
5. Rust deserializes the payload into typed UI nodes and renders them with `iced`.
6. When the user clicks a button or changes an input, Rust sends the callback ID back into the JS runtime.
7. JavaScript updates state and calls `App.requestRender()`.

## Running Examples

Available examples:

- [examples/hello_world_counter.js](examples/hello_world_counter.js): renders `Hello world` and an increment button
- [examples/text_input_echo.js](examples/text_input_echo.js): controlled text input with live echo
- [examples/select_input_echo.js](examples/select_input_echo.js): controlled native select input backed by labeled options
- [examples/flex_form.js](examples/flex_form.js): centered form layout using web-style flex props
- [examples/flat_list.js](examples/flat_list.js): renders repeated rows from array data with item-specific callbacks through `NativeList`
- [examples/task_form_flat_list.js](examples/task_form_flat_list.js): simple task form that adds, completes, and deletes items inside a FlatList
- [examples/modal.js](examples/modal.js): opens a native modal overlay and dismisses it with buttons or `Escape`
- [examples/storage_bridge.js](examples/storage_bridge.js): persists and clears preferences through the async `Storage` bridge
- [examples/tabs.js](examples/tabs.js): controlled tab helper with tab-strip styling
- [examples/toast.js](examples/toast.js): shows a timer-backed toast notification
- [examples/multi_file_save_button/main.js](examples/multi_file_save_button/main.js): imports a reusable `SaveButton` component from a sibling module
- [examples/router_demo/main.js](examples/router_demo/main.js): multi-file route demo with path params, query parsing, and programmatic navigation
- [examples/login-app/app/main.js](examples/login-app/app/main.js): routed login app with auth modals, protected task and user screens, fetch, `FlatList`, `SelectInput`, and imperative `Alert` confirmations for destructive actions
- [examples/pokemon_fetch.js](examples/pokemon_fetch.js): fetches Pokemon details from PokeAPI using the new async `fetch` bridge

Run any example directly:

```sh
cargo run -- examples/hello_world_counter.js
```

For example:

```sh
cargo run -- examples/select_input_echo.js
```

Or try the centered flex form example:

```sh
cargo run -- examples/flex_form.js
```

Or try the FlatList example:

```sh
cargo run -- examples/flat_list.js
```

Or use the helper script:

```sh
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run-example.ps1
```

Or try the fetch example:

```sh
cargo run -- examples/pokemon_fetch.js
```

Or try the modal example:

```sh
cargo run -- examples/modal.js
```

Or run the multi-file example:

```sh
cargo run -- examples/multi_file_save_button/main.js
```

Or try the router example:

```sh
cargo run -- examples/router_demo/main.js
```

Or run the login app example:

```sh
cargo run -- examples/login-app/app/main.js
```

Or run the perf harness:

```sh
cargo run --bin perf_harness
```

Or use the helper scripts:

```sh
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run-perf.ps1
```

```sh
bash scripts/run-perf.sh
```

The current dev-mode baseline snapshot lives at [docs/perf-baseline.json](docs/perf-baseline.json).

## Multi-file ESM Apps

When you pass a file path to `cargo run -- <entry-file>`, RustyJS-UI now treats that file as an ECMAScript module entrypoint. Local component files can use standard static imports and exports without a bundler.

```js
// save_button.js
export function SaveButton(props = {}) {
  return Button({
    text: props.text ?? 'Save',
    onClick: props.onClick
  });
}

// main.js
import { SaveButton } from './save_button.js';
```

Current module-loading rules:

- Relative `./` and `../` imports are supported for local modules
- The built-in package specifier `RustyJS-UI` is supported for component imports (`import { Button } from 'RustyJS-UI'`)
- Import specifiers must include the `.js` extension
- Imports are resolved relative to the importing file and must stay inside the entry file's root directory
- Existing single-file apps still work unchanged

Imperative alert example:

```js
import { Alert } from 'RustyJS-UI';

function calcHeight() {
  Alert({
    title: 'Recalculate layout?',
    description: 'This action may reset your current draft.',
    primaryButtonText: 'Continue',
    primaryButtonOnClick: () => {},
    secondaryButtonText: 'Cancel',
    secondaryButtonOnClick: () => {}
  });
}
```

Type declarations for the built-in package live in [types/rustyjs-ui.d.ts](types/rustyjs-ui.d.ts).

## Routing API

RustyJS-UI includes a JS-side router helper for path-based navigation inside a native app. The router is created in JavaScript and drives the normal `App.requestRender()` loop, so it fits the current runtime without changing the Rust bridge.

Supported behavior:

- `initialPath` sets the first active route
- `routes` defines the route table in first-match order
- `:param` segments are extracted into `route.params`
- query strings are exposed as `route.query`
- `navigate`, `replace`, `back`, and `forward` update the in-memory history stack and re-render
- `router.getPath()` returns the current normalized path

Example:

```js
function HomeScreen(route) {
  return View({
    children: [
      Text({ text: `Path: ${route.path}` }),
      Button({ text: 'Open project', onClick: () => route.navigate('/projects/alpha?tab=overview') })
    ]
  });
}

function ProjectScreen(route) {
  return View({
    children: [
      Text({ text: `Project: ${route.params.projectId}` }),
      Text({ text: `Tab: ${route.query.tab || 'none'}` }),
      Button({ text: 'Back home', onClick: () => route.navigate('/') })
    ]
  });
}

const router = App.createRouter({
  initialPath: '/',
  routes: [
    { path: '/', render: HomeScreen },
    { path: '/projects/:projectId', render: ProjectScreen }
  ],
  notFound: (route) =>
    View({
      children: [
        Text({ text: `No route matched ${route.path}` }),
        Button({ text: 'Home', onClick: () => route.navigate('/') })
      ]
    })
});

App.run({
  title: 'Router Demo',
  render: () =>
    View({
      children: [
        Text({ text: `Current path: ${router.getPath()}` }),
        router.render()
      ]
    })
});
```

## DX APIs

### Storage

`Storage` is an async key/value bridge backed by a host-side JSON store.

```js
async function saveTheme() {
  await Storage.set('theme', 'forest');
  const theme = await Storage.get('theme');
  App.requestRender();
}
```

### Timer

`Timer.after(ms)` resolves after the requested delay and can be used for non-blocking UI flows such as toast dismissal.

```js
Timer.after(1000).then(() => {
  App.requestRender();
});
```

### Navigation

`Navigation` exposes the router factory and path helpers as a stable public surface.

```js
const router = Navigation.createRouter({
  routes: [{ path: '/', render: () => Text({ text: 'Home' }) }]
});
```

## Running Tests

```sh
cargo test
```

## SelectInput API

`SelectInput` is backed by Iced's `PickList`, which is the closest native equivalent to a web-style single-select dropdown.

`NativeSelect` is the preferred name for new code and is currently an alias of `SelectInput`.

Supported props:

- `value: string`
- `placeholder?: string`
- `options: Array<string | { label: string, value: string }>`
- `onChange?: (nextValue: string) => void`
- `style?: { width, padding, borderWidth, borderRadius, borderColor, backgroundColor, color, fontSize }`

Example:

```js
const frameworks = [
  { label: 'Rust', value: 'rust' },
  { label: 'TypeScript', value: 'typescript' }
];

let value = '';

function handleChange(nextValue) {
  value = nextValue;
  App.requestRender();
}

NativeSelect({
  value,
  placeholder: 'Choose a language',
  options: frameworks,
  onChange: handleChange,
  style: {
    width: 320,
    padding: 10,
    borderWidth: 1,
    borderRadius: 8,
    borderColor: '#C7CDD4'
  }
});
```

## Modal API

`Modal` renders above the current tree using an `iced` overlay, similar to React Native's modal presentation model.

Supported props:

- `visible?: boolean` defaults to `true`
- `transparent?: boolean` defaults to `false`
- `closeOnEscape?: boolean` defaults to `true`
- `closeOnBackdrop?: boolean` defaults to `false`
- `onRequestClose?: () => void` fires when `Escape` is pressed while the modal is open
- `backdropColor?: string | { red, green, blue, alpha }`
- `style?: object` applied to the full-window modal container

Behavior notes:

- Modal content is rendered in a full-window container with `width: 'fill'` and `height: 'fill'`
- When `transparent` is `false`, the modal uses `backdropColor` if provided, otherwise a white backdrop
- Background content is blocked while a modal is visible

Example:

```js
let visible = false;

function openModal() {
  visible = true;
  App.requestRender();
}

function closeModal() {
  visible = false;
  App.requestRender();
}

Modal({
  visible,
  onRequestClose: closeModal,
  backdropColor: '#00000059',
  style: {
    justifyContent: 'center',
    alignItems: 'center',
    padding: 24
  },
  children: [
    View({
      style: {
        width: 360,
        padding: 20,
        gap: 12,
        backgroundColor: '#FFFFFF',
        borderRadius: 18
      },
      children: [
        Text({ text: 'Confirm action' }),
        Button({ text: 'Close', onClick: closeModal })
      ]
    })
  ]
});
```

## Flex Layout API

`View` behaves like a flex container and now accepts more web-style layout props in addition to the existing aliases.

Supported layout props include:

- `flexDirection` or `direction`: `'row' | 'column'`
- `gap` or `spacing`: number
- `justifyContent`: `'start' | 'center' | 'end' | 'flex-start' | 'flex-end' | 'space-between' | 'space-around' | 'space-evenly'`
- `alignItems`: `'start' | 'center' | 'end' | 'stretch' | 'flex-start' | 'flex-end'`
- `width` / `height`: number, `'fill'`, `'auto'`, or `'shrink'`

Example:

```js
View({
  style: {
    width: 'fill',
    height: 'fill',
    flexDirection: 'column',
    justifyContent: 'center',
    alignItems: 'center',
    gap: 14
  },
  children: [
    Text({ text: 'Centered content' }),
    Button({ text: 'Save' })
  ]
});
```

## FlatList API

`FlatList` accepts React Native-style `data` and `renderItem` props, expands them into child nodes in JavaScript, and renders them through a native `iced::scrollable` container in Rust. It supports scrolling, but it does not provide virtualization yet.

`NativeList` is the preferred name for new code and currently delegates to the same bridge path while accepting forward-looking virtualization props for compatibility.

Supported props:

- `data: Array<any>`
- `renderItem: ({ item, index }) => Node`
- `horizontal?: boolean`
- `style?: object` for the outer scroll container
- `contentContainerStyle?: object` for the inner content layout
- `ListEmptyComponent?: (() => Node) | Node`
- `ListHeaderComponent?: (() => Node) | Node`
- `ListFooterComponent?: (() => Node) | Node`
- `ItemSeparatorComponent?: ((context) => Node) | Node`
- `keyExtractor?: (item, index) => string` (accepted for API parity, currently ignored by the bridge)

Default behavior:

- Vertical lists default to `width: 'fill'` and `height: 'fill'`
- Horizontal lists default to a fill-sized scroll container with row layout

Example:

```js
const todos = [
  { id: '1', title: 'Buy milk', done: false },
  { id: '2', title: 'Call Ada', done: true }
];

NativeList({
  data: todos,
  style: {
    gap: 10
  },
  renderItem: ({ item, index }) =>
    View({
      style: {
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center'
      },
      children: [
        Text({ text: `${index + 1}. ${item.title}` }),
        Button({
          text: item.done ? 'Done' : 'Toggle',
          onClick: () => {
            item.done = !item.done;
            App.requestRender();
          }
        })
      ]
    })
});
```

## Project Layout

- [src/runtime](src/runtime) contains the embedded JS runtime and bootstrap script
- [src/bridge.rs](src/bridge.rs) defines the bridge payloads
- [src/vdom.rs](src/vdom.rs) defines the wire format and typed UI nodes
- [src/ui.rs](src/ui.rs) renders typed nodes into `iced` widgets
- [examples](examples) contains runnable JavaScript examples
- [tests](tests) contains integration tests for the bridge
- [types/rustyjs-ui.d.ts](types/rustyjs-ui.d.ts) defines the typed built-in package surface
- [src/bin/perf_harness.rs](src/bin/perf_harness.rs) provides a repeatable headless perf harness

## Status

This is still an MVP. The bridge, rendering flow, example loading, and integration tests are in place, but the project is still early and intended for experimentation rather than production use.

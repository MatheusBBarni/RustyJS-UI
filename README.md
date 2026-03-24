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
  - `SelectInput`
  - `FlatList` (native scrollable list with JS-side item expansion)
- Style support for common layout and text fields
- Callback-based event dispatch from Rust back into JavaScript

## How It Works

At a high level, the flow looks like this:

1. JavaScript calls `App.run(...)`.
2. The runtime sends an `INIT_WINDOW` payload to Rust.
3. The JavaScript `render` function returns a tree of nodes such as `View`, `Text`, `Button`, `TextInput`, and `SelectInput`.
4. That tree is serialized into an `UPDATE_VDOM` payload.
5. Rust deserializes the payload into typed UI nodes and renders them with `iced`.
6. When the user clicks a button or changes an input, Rust sends the callback ID back into the JS runtime.
7. JavaScript updates state and calls `App.requestRender()`.

## Running Examples

Available examples:

- [examples/hello_world_counter.js](examples/hello_world_counter.js): renders `Hello world` and an increment button
- [examples/text_input_echo.js](examples/text_input_echo.js): controlled text input with live echo
- [examples/select_input_echo.js](examples/select_input_echo.js): controlled select input backed by labeled options
- [examples/flex_form.js](examples/flex_form.js): centered form layout using web-style flex props
- [examples/flat_list.js](examples/flat_list.js): renders repeated rows from array data with item-specific callbacks
- [examples/task_form_flat_list.js](examples/task_form_flat_list.js): simple task form that adds, completes, and deletes items inside a FlatList

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

## Running Tests

```sh
cargo test
```

## SelectInput API

`SelectInput` is backed by Iced's `PickList`, which is the closest native equivalent to a web-style single-select dropdown.

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

SelectInput({
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

FlatList({
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

## Status

This is still an MVP. The bridge, rendering flow, example loading, and integration tests are in place, but the project is still early and intended for experimentation rather than production use.

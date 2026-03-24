# Product Requirements Document (PRD)

**Product Name:** RustyJS-UI  
**Document Status:** V3 (Final Architecture & Tooling)  
**Core Technologies:** Rust, Iced (GUI), Embedded JS Engine, VDOM Bridge

---

## 1. Product Vision & Objective

The goal of **RustyJS-UI** is to build a highly performant, memory-safe JavaScript runtime optimized for native desktop GUI development. By combining an embedded JavaScript engine with Rust’s **Iced** framework, developers can build fast, cross-platform applications using a declarative, state-driven JavaScript API.

The system relies on a **Unidirectional Data Flow** and a **Virtual DOM (VDOM)** architecture:
- **JavaScript Layer:** Acts as the state container and UI declarative layer.
- **Rust Layer:** Acts as the high-performance reconciliation and rendering engine.

## 2. Core Architecture: The VDOM Bridge

The system completely separates the application state from the native UI components to ensure memory safety and maintain 60 FPS performance.

### The JavaScript Layer (State & Declaration)
Developers write pure, reactive JavaScript. State is managed locally (conceptually similar to tools like Zustand or Redux). When state changes, a `render()` function generates a lightweight, framework-agnostic object tree representing the UI (the VDOM).

### The Serialization Bridge
Instead of making expensive **Foreign Function Interface (FFI)** calls for every single UI element, the JS engine serializes the entire VDOM tree (or just the diff) into a single payload and passes it to Rust.

### The Rust Layer (Reconciliation & Rendering)
Rust receives the payload, deserializes it into an internal `UiDescription` struct, and maps it directly to **Iced’s** native widget tree. Iced handles OS events, drawing text, and rendering pixels via **wgpu** (leveraging Metal on macOS, Vulkan on Windows/Linux).

## 3. Core Features (Scope)

### 3.1. JS Engine & Event Loop
- Support for modern **ES6+** syntax.
- Non-blocking `async/await` execution tied into the Rust async runtime.
- Global `App` object to handle window lifecycle and trigger the render pipeline.

### 3.2. Declarative UI Components
A standard library of components exposed to the JS environment that map exactly to Iced primitives:
- **View:** Flexbox layout container (maps to Iced's Column or Row).
- **Text:** Typography rendering.
- **Button:** Interactive triggers.
- **TextInput:** Controlled text input fields.

### 3.3. CSS-like Styling System
Components accept a `style` attribute (a standard JS object) that Rust parses and applies strictly to the Iced layout engine.
- **Layout:** `flexDirection`, `padding`, `spacing`, `width`, `height`, `alignItems`, `justifyContent`.
- **Aesthetics:** `backgroundColor`, `color`, `fontSize`, `borderRadius`, `borderWidth`, `borderColor`.

### 3.4. Event Handling
Events are passed as string identifiers or callback IDs across the bridge. When a native Iced button is clicked, Rust sends the corresponding ID back to the JS event loop to execute the developer's callback, mutating the state and triggering a new render phase.

## 4. Required Rust Crates (The Tech Stack)

To execute this architecture, we will use the following foundational crates.

### 4.1. The JavaScript Engine
- **boa_engine:** A 100% pure Rust JavaScript engine.
  - *Why:* It is easy to embed, avoids C++ compilation overhead (V8), and plays well with Rust’s memory model. Perfect for an MVP.

### 4.2. The GUI Rendering Backend
- **iced:** The core reactive GUI framework.
  - *Why:* Type-safe, Elm-architecture native, and prevents imperative UI mistakes.

### 4.3. The Bridge (Serialization & Deserialization)
- **serde & serde_json:** For standardizing the VDOM payload structure.
  - *Why:* Industry standard for serializing Rust structs. Used to convert JS object trees into Rust data.

### 4.4. Async Runtime & OS Utilities
- **tokio:** The asynchronous runtime for Rust.
  - *Why:* Required to manage the JS event loop, handle file I/O, and process cross-thread messages without blocking Iced's UI thread.
- **anyhow:** For unified error handling.
  - *Why:* Essential for capturing and neatly formatting JS syntax errors or FFI bridge panics for diagnostic logging.


# JS Code Example

```javascript
import { App, View, Text, Button } from 'RustyJS-UI';

// Application State
let counter = 0;

// Mutator
function increment() {
    counter += 1;
    App.requestRender(); // Queues a re-render
}

// View
function AppLayout() {
    return View({
        style: { direction: 'column', padding: 20, alignItems: 'center' },
        children: [
            Text({ 
                text: `Count is: ${counter}`, 
                style: { fontSize: 24, color: '#000000' } 
            }),
            Button({ 
                text: "Increment", 
                onClick: increment, // Engine converts this to an ID!
                style: { padding: 10, backgroundColor: '#007AFF' } 
            })
        ]
    });
}

// Bootstrap
App.run({
    title: "Counter App",
    windowSize: { width: 400, height: 300 },
    render: AppLayout
});
```

# JS Engine (Bridge)

```javascript
// ==========================================
// 1. CALLBACK REGISTRY (The Event Bridge)
// ==========================================
// Stores JS functions so Rust can trigger them via IDs.
class CallbackRegistry {
    constructor() {
        this.callbacks = new Map();
        this.nextId = 1;
    }

    register(fn) {
        if (typeof fn !== 'function') return null;
        const id = `cb_${this.nextId++}`;
        this.callbacks.set(id, fn);
        return id;
    }

    // Rust will invoke this global function when an event occurs in Iced
    trigger(id, payload) {
        const fn = this.callbacks.get(id);
        if (fn) {
            fn(payload);
        } else {
            console.warn(`Callback ${id} not found.`);
        }
    }

    clear() {
        this.callbacks.clear();
    }
}

const GlobalCallbackRegistry = new CallbackRegistry();
// Expose to the global scope so Rust can call it: engine.eval("RustBridge.trigger('cb_1')")
globalThis.RustBridge = {
    trigger: (id, payload) => GlobalCallbackRegistry.trigger(id, payload)
};


// ==========================================
// 2. VDOM COMPONENT FACTORIES
// ==========================================
// These functions build the lightweight JS objects that will be serialized.

function createNode(type, props = {}) {
    const node = { type, props: {}, children: [] };

    for (const [key, value] of Object.entries(props)) {
        if (key === 'children') {
            // Flatten nested arrays of children
            node.children = Array.isArray(value) ? value.flat() : [value];
        } else if (typeof value === 'function') {
            // Convert functions to string IDs for Rust
            node.props[key] = GlobalCallbackRegistry.register(value);
        } else {
            // Standard props (style, text, value, etc.)
            node.props[key] = value;
        }
    }
    return node;
}

// Public API for developers
export const View = (props) => createNode('View', props);
export const Text = (props) => createNode('Text', props);
export const Button = (props) => createNode('Button', props);
export const TextInput = (props) => createNode('TextInput', props);


// ==========================================
// 3. THE APP RUNTIME
// ==========================================
// Manages the render cycle and serialization to Rust.

class AppEngine {
    constructor() {
        this.rootRenderFn = null;
        this.isRenderPending = false;
    }

    run(config) {
        this.rootRenderFn = config.render;
        
        // Initial setup payload (window size, title)
        const initPayload = {
            action: 'INIT_WINDOW',
            title: config.title || 'IcedJS App',
            width: config.windowSize?.width || 800,
            height: config.windowSize?.height || 600
        };
        
        // Send init config to Rust (assuming `__SEND_TO_RUST__` is injected by the Rust host)
        globalThis.__SEND_TO_RUST__(JSON.stringify(initPayload));

        // Trigger first render
        this.requestRender();
    }

    requestRender() {
        if (this.isRenderPending) return;
        this.isRenderPending = true;

        // Use Promise.resolve to batch state updates (microtask queue)
        Promise.resolve().then(() => {
            this.executeRender();
        });
    }

    executeRender() {
        // Clear old callbacks to prevent memory leaks across renders
        GlobalCallbackRegistry.clear();

        // Generate the new VDOM tree
        const vdomTree = this.rootRenderFn();

        // Package the payload
        const payload = {
            action: 'UPDATE_VDOM',
            tree: vdomTree
        };

        // Serialize and send across the FFI bridge
        globalThis.__SEND_TO_RUST__(JSON.stringify(payload));
        
        this.isRenderPending = false;
    }
}

export const App = new AppEngine();
```
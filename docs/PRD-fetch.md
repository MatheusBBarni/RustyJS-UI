# Feature PRD: Asynchronous Network Bridge (fetch API)

- **Project:** RustyJS-UI
- **Feature:** HTTP Request Capability
- **Status:** Not started
- **Target Release:** MVP

---

## 1. Feature Overview & Objective
The embedded JavaScript engine (`boa_engine`) is a pure ECMAScript environment and does not include browser-specific Web APIs like `fetch` or `XMLHttpRequest`. However, modern applications require network access to fetch data, communicate with APIs, and download assets.

This feature will implement a custom, asynchronous `fetch` function injected into the global JavaScript scope. Under the hood, it will bridge to Rust's highly optimized `reqwest` library, ensuring that network requests are executed on background threads and do not block the 60 FPS Iced UI rendering loop.

## 2. Scope

### 2.1. In Scope for MVP
- **Global `fetch` function:** Injection of a global asynchronous `fetch(url, options)` function into the JS environment.
- **HTTP Methods:** Support for standard HTTP methods: `GET`, `POST`, `PUT`, `DELETE`.
- **Payloads & Headers:** Support for passing JSON bodies and custom headers from JS to Rust.
- **Promises:** Returning JS `Promise` objects that resolve with stringified JSON or text.
- **Async Execution:** Non-blocking execution using Rust's `tokio` async runtime.

### 2.2. Out of Scope for MVP
- **Streaming:** Streaming responses (e.g., video/audio streaming).
- **WebSockets/SSE:** WebSockets or Server-Sent Events (SSE).
- **Full Response API:** Full `Response` object mapping (e.g., exposing `.blob()`, `.formData()`). The MVP will focus purely on text/JSON payloads.

## 3. Technical Architecture
The networking bridge relies on resolving the mismatch between JavaScript's single-threaded event loop and Rust's multi-threaded asynchronous model.

1.  **The Invocation:** JavaScript calls `fetch()`. The engine creates an unresolved JS Promise and passes the URL and configuration to Rust.
2.  **The Rust Delegation:** The Rust host receives the request parameters. It immediately returns control to the JS engine (allowing the UI to continue rendering) and spawns a background task using `tokio::spawn`.
3.  **The Execution:** The `tokio` background task executes the HTTP request using the `reqwest` crate.
4.  **The Resolution:** Once `reqwest` receives the response (or fails), the background task pushes the result to a thread-safe queue.
5.  **The Callback:** On the next tick of the application's event loop, Rust polls the queue, retrieves the response, and resolves or rejects the corresponding JS Promise.

## 4. Dependencies
- **`reqwest`:** The core HTTP client in Rust. (Features required: `json`, `rustls-tls` or `native-tls`).
- **`tokio`:** The async runtime required to spawn non-blocking network tasks alongside the Iced GUI.

## 5. Developer Experience (DX)
The goal is to provide a networking API that feels completely native to modern frontend developers. This architecture aligns perfectly with modern state-management paradigms like Redux or Zustand, allowing developers to fetch data and predictably trigger state mutations.

```javascript
let state = {
    data: null,
    loading: false,
    error: null
};

// State Mutator Function
async function handleFetch() {
    state.loading = true;
    App.requestRender();

    try {
        // Looks and acts like standard Web API fetch
        const responseText = await fetch("https://api.example.com/data", {
            method: "GET",
            headers: { "Authorization": "Bearer token123" }
        });
        
        state.data = JSON.parse(responseText);
    } catch (e) {
        state.error = "Network request failed.";
    } finally {
        state.loading = false;
        App.requestRender();
    }
}
```
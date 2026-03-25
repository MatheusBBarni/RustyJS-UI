# Login App Example

This example builds a routed desktop app on top of RustyJS-UI and the sibling REST API.

Routes:

- `/` public landing route with separate login and register modals
- `/tasks` protected task list using `FlatList`
- `/tasks/:taskId` protected task details route with view/edit modes
- `/users` protected user management route using `FlatList`

Notes:

- Start the API in `examples/login-app/rest-api` first.
- Task and user updates use `PUT`, which matches the current fetch bridge.
- Lists are rendered with `FlatList`.
- The app uses the runtime's router, modal, fetch, `SelectInput`, and multi-file module support.

Run it with:

```sh
cargo run -- examples/login-app/app/main.js
```

Or launch the API and app together:

```sh
bash scripts/run-login-app.sh
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run-login-app.ps1
```

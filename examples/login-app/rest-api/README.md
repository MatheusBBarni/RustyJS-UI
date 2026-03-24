# login-app/rest-api

Simple Bun API built with Elysia.

## Routes

- `POST /users` creates a user
- `POST /login` returns a bearer token
- `GET /tasks` lists the current user's tasks
- `POST /tasks` creates a task
- `GET /tasks/:id` returns one task
- `PATCH /tasks/:id` updates a task
- `DELETE /tasks/:id` deletes a task

## Run

```sh
bun install
bun run dev
```

Default port: `3000`

## Example requests

```sh
curl -X POST http://localhost:3000/users \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"Ada\",\"email\":\"ada@example.com\",\"password\":\"secret123\"}"
```

```sh
curl -X POST http://localhost:3000/login \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"ada@example.com\",\"password\":\"secret123\"}"
```

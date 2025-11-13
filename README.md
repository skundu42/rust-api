# rust-api

An educational Axum todo service that demonstrates how to build, test, and
observe a small Rust HTTP API. Everything runs entirely in memory, so you can
clone the repo and start experimenting immediately—no external services needed.

## Features
- Axum 0.7 router with typed request/response handling and middleware (trace, CORS, compression).
- In-memory repository guarded by `tokio::sync::Mutex`, exposed through a `TodoRepo`
  trait so you can swap in a database later.
- Centralized error handling that maps domain errors to consistent JSON bodies.
- Integration-style test (`tests/todos.rs`) that exercises the full router without
  binding a TCP port.
- Structured logging with `tracing` and `RUST_LOG`/`EnvFilter` support.

## Quick start
### Prerequisites
- Rust toolchain (1.74+ recommended) with `cargo`
- `pkg-config`/OpenSSL are **not** required because storage is in-memory

### Install dependencies
Everything you need is already captured in `Cargo.toml`, so a regular `cargo`
build will fetch crates automatically:

```bash
cargo check
```

### Run the API
```bash
cargo run
```

The server listens on `0.0.0.0:8080` by default. Locally, you can opt into more
verbose logs by exporting `RUST_LOG`/`RUST_API_LOG=debug`. The binary calls
`dotenvy::dotenv()`, so placing secrets or overrides inside a `.env` file keeps
them out of your shell history.

### Sample session
```bash
# health check
curl -i http://localhost:8080/health

# create a todo
curl -i -X POST http://localhost:8080/todos \
  -H 'content-type: application/json' \
  -d '{ "title": "learn rust" }'

# toggle completion
curl -i -X PUT http://localhost:8080/todos/1 \
  -H 'content-type: application/json' \
  -d '{ "done": true }'
```

## API overview
### Data model
```json
{
  "id": 1,
  "title": "learn rust",
  "done": false
}
```

### Endpoints
| Method | Path        | Description                                  | Success codes | Request body             |
|--------|-------------|----------------------------------------------|---------------|--------------------------|
| GET    | `/health`   | Liveness probe                               | 200           | _None_                   |
| GET    | `/todos`    | List every todo                              | 200           | _None_                   |
| POST   | `/todos`    | Create a todo                                | 201           | `{ "title": "..." }`     |
| GET    | `/todos/:id`| Fetch a todo                                 | 200           | _None_                   |
| PUT    | `/todos/:id`| Update title and/or completion flag          | 200           | `{ "title": "...?", "done": true? }` |
| DELETE | `/todos/:id`| Remove a todo                                | 204           | _None_                   |

### Validation & errors
- Titles are trimmed and cannot be empty.
- `PUT` requests must include at least one field.
- Missing records respond with `404 {"error":"not found"}`.
- Validation issues respond with `400 {"error":"validation error: ..."}`.
- Unexpected failures respond with `500 {"error":"internal error"}`.

## Testing
Run the full suite, including the router-level CRUD flow, with:

```bash
cargo test
```

The integration test (`tests/todos.rs`) exercises create → read → update →
list → delete. Use it as a template when adding new routes or when swapping
the repository implementation.

## Extending the service
- Replace the `InMemory` repo in `state.rs` with a database-backed struct that
  still implements `TodoRepo`.
- Add authentication/authorization layers via Axum middleware.
- Ship richer telemetry by forwarding `tracing` spans to OpenTelemetry.
- Wrap the server with Docker and deploy it wherever `cargo` binaries run.


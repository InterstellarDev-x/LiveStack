# LiveStack

LiveStack is a Rust workspace for tracking website liveness and moving monitoring data through an API + Redis stream pipeline.

## Workspace Modules

- `api` (binary): HTTP layer and application business logic (Poem framework).
- `store` (library): Diesel models, schema, and PostgreSQL access.
- `messaging` (library): Shared Redis stream client utilities.
- `producer` (binary): Periodically reads websites from DB and pushes records to Redis streams.
- `consumer` (binary): Consumer-side service experiments (currently includes HTTP fetch sample and consumer-group notes).

## High-Level Flow

```text
Client -> API -> PostgreSQL
                 |
                 v
              Producer -> Redis Stream -> Consumer
```

## Prerequisites

1. Rust toolchain (stable).
2. PostgreSQL running locally.
3. Redis running locally.

## Default Local Configuration

Current defaults in code:

- PostgreSQL URL: `postgres://postgres:mysecretpassword@localhost:5432/better-uptime`
- Redis URL: `redis://127.0.0.1/`
- API bind address: `0.0.0.0:3000`

## Setup

1. Start PostgreSQL and create the database `better-uptime`.
2. Start Redis on localhost.
3. Run Diesel migrations from the `store` crate if needed.
4. Build all crates:

```bash
cargo build
```

## Run Services

Run each binary in separate terminals from the workspace root.

### API

```bash
cargo run -p api
```

### Producer

```bash
cargo run -p producer
```

### Consumer

```bash
cargo run -p consumer
```

## API Endpoints

### Auth

- `POST /signup`
- `POST /signin`

### Website

- `POST /website`
- `GET /website/:website_id`
- `PUT /website/:website_id`
- `DELETE /website/:website_id`
- `GET /websites/:user_id`

## Current Architecture Note

The API currently uses a shared `Arc<Mutex<Store>>` connection model, which serializes DB access per request path.

Planned migration to pooled DB connections is documented in `api/futureimpl.md`.

## Learning Note

In Rust, a type is identified by its full path, not just the visible name.
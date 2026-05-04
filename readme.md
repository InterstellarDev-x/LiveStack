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
4. Diesel CLI if you want to run migrations yourself:

```bash
cargo install diesel_cli --no-default-features --features postgres
```

## Default Local Configuration

Current defaults in code:

- PostgreSQL URL: `postgres://postgres:mysecretpassword@localhost:5432/better-uptime`
- Redis URL: `redis://127.0.0.1/`
- API bind address: `0.0.0.0:3000`
- Producer interval: every `180` seconds
- Consumer group: `uptime-checkers`

Consumer-specific environment variables:

- `CONSUMER_NAME` (optional): defaults to `consumer-<pid>`
- `REGION_ID` (optional): defaults to `india`
- `REGION_NAME` (optional): defaults to `India`

## Setup

1. Start PostgreSQL and create the database `better-uptime`.
2. Start Redis on localhost.
3. Run Diesel migrations from the workspace root:

```bash
diesel migration run --migration-dir store/migrations
```

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

## Recommended Local Order

1. Start PostgreSQL and Redis.
2. Run migrations.
3. Start `api`.
4. Start `consumer`.
5. Start `producer`.

This order ensures the API can create records, the consumer group is initialized, and the producer can start queueing website checks immediately.

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

## Request Shapes

### Signup

```json
{
  "username": "alice",
  "password": "secret"
}
```

### Signin

```json
{
  "username": "alice",
  "password": "secret"
}
```

### Create or Update Website

```json
{
  "url": "https://example.com"
}
```

## Auth Note

Authenticated website routes currently expect the JWT in a `token` request header, not an `Authorization: Bearer ...` header.

## What Each Service Does

- `api`: manages users and websites, reads and writes PostgreSQL records.
- `producer`: periodically loads all websites from PostgreSQL and pushes them into a Redis stream.
- `consumer`: reads from the Redis stream consumer group, performs HTTP checks, and stores website tick results back in PostgreSQL.

## Current Architecture Notes

- The API currently uses a Diesel `r2d2` connection pool.
- `producer` still uses a direct `Store::default()` connection protected by `Arc<Mutex<_>>` for its scheduled job.
- The consumer normalizes URLs by prepending `https://` when a scheme is missing.
- Future implementation notes are documented in `api/futureimpl.md`.

## Learning Note

In Rust, a type is identified by its full path, not just the visible name.

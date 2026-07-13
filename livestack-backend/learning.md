# LiveStack Learnings

## 1. Cargo Lock / "Blocking Waiting for File Lock"

1. Cargo uses a shared global cache for crates and build artifacts.
2. Only one Cargo-related process can mutate that cache at a time.
3. `Blocking waiting for file lock` usually means another Cargo process is already running.
4. This is normally not an error.
5. A common hidden cause is the IDE, especially Rust Analyzer, running background commands like:
   - `cargo check`
   - background diagnostics
   - autocomplete and type checking

## 2. Rust Type Identity

- In Rust, a type is identified by its full path, not only by the visible type name.
- That matters when two types look similar by name but come from different modules or crates.

## 3. Workspace Architecture

- This repo is split into focused crates:
  - `api` for HTTP and business logic
  - `store` for PostgreSQL + Diesel models and queries
  - `messaging` for Redis stream utilities
  - `producer` for pushing website-check jobs into Redis
  - `consumer` for reading jobs and writing check results back to the database
- The high-level flow is:

```text
Client -> API -> PostgreSQL
                 |
                 v
              Producer -> Redis Stream -> Consumer
```

## 4. Database Concurrency: Mutex vs Pool

- A single `Arc<Mutex<Store>>` or `Arc<Mutex<PgConnection>>` serializes database access.
- That creates an artificial bottleneck because only one request can use the database connection at a time.
- A connection pool improves concurrency by letting multiple requests borrow different connections.
- PostgreSQL is then allowed to handle real concurrent work instead of the application forcing everything through one lock.

### Important distinction

- A mutex around one connection protects the whole query path.
- A pool only synchronizes connection checkout and return.
- The pool does not serialize all queries the way a single mutex-protected connection does.

## 5. Current Repo State About Pooling

- Older notes in `api/futureimpl.md` describe pooling as a migration plan.
- The current code has already implemented pooling in the API:
  - `store/src/lib.rs` defines `DbPool`
  - `api/src/main.rs` creates the pool with `Store::pool()`
  - route handlers borrow a `Store` from the pool through `Store::from_pool(...)`
- The `Store` type now supports both:
  - direct connections via `Store::default()`
  - pooled connections via `Store::from_pool(...)`

## 6. Pooling Does Not Make Diesel Async

- Diesel with `PgConnection` is still synchronous.
- Pooling improves throughput by allowing multiple concurrent connections.
- But each individual query still blocks the thread running it.
- If blocking becomes a problem later, possible next steps are:
  - use `spawn_blocking`
  - move to an async database stack if needed

## 7. Redis Stream Consumer Group Behavior

- A consumer group acts like a logical reader shared by multiple consumers.
- Each message is delivered to one consumer within the group, not to every consumer.
- Consumers are identified by a chosen case-sensitive name.
- Because of that, the group can preserve delivery state even after disconnects, as long as the same consumer name is reused.
- Each group tracks what has not been consumed yet.
- Processing requires explicit acknowledgment.
- Unacknowledged messages remain pending inside the group.
- Pending tracking allows recovery and retry behavior instead of silently losing work.

## 8. How This Repo Uses Redis Streams

- The producer reads websites from PostgreSQL and pushes batched `XADD` records into the Redis stream.
- The consumer:
  - ensures the consumer group exists
  - reads new messages through group reads
  - claims old pending messages when needed
  - checks website availability over HTTP
  - writes ticks back into PostgreSQL
  - acknowledges processed stream IDs
- Malformed stream records are acknowledged and dropped so they do not block the group forever.

## 9. Recovery / Reliability Lessons

- Acknowledgment is a real part of the workflow, not an optional detail.
- If a consumer dies after reading but before acknowledging, the message can remain pending.
- Claiming idle pending records is necessary for recovery in consumer-group based systems.
- This repo handles that by attempting to claim pending messages when no fresh messages are available.

## 10. URL / Monitoring Behavior

- The consumer normalizes URLs before sending HTTP requests.
- If a URL does not already start with `http://` or `https://`, the code prefixes `https://`.
- Website status is currently simplified to:
  - `Up` for successful HTTP status responses
  - `Down` for failed requests or non-success responses

## 11. Security / Production Gaps Already Visible in the Repo

- Passwords are still stored and compared in plaintext.
- JWT secret is hardcoded.
- Some config values are still hardcoded in code instead of coming from environment variables.
- These are already recognized in the project notes as future improvements.

## 12. Practical Takeaways

- Shared mutable access with a mutex is simple, but it can quietly destroy concurrency.
- Connection pools are usually the correct default once multiple requests or workers are involved.
- Redis consumer groups require explicit ownership, acknowledgment, and retry thinking.
- Background tooling can explain Cargo lock waits.
- Repo docs can become stale; the code should be treated as the source of truth when notes and implementation diverge.

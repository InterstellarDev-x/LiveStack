


# Database Connection Pool Migration Plan

## Goal

Replace the current single shared database connection:

```rust
Arc<Mutex<Store>>
```

with a proper pooled connection architecture so the API can handle concurrent requests efficiently.

## Why This Change?

Current architecture serializes all DB access through one connection:

```text
Multiple Requests
      ↓
Arc<Mutex<PgConnection>>
      ↓
Single Database Connection
```

### Problems

- Only one request can access DB at a time.
- Poor scalability under concurrent traffic.
- Artificial bottleneck before database.
- Does not utilize PostgreSQL's concurrent connection support.

## New Architecture

```text
Multiple Requests
      ↓
Connection Pool
      ↓
Many PostgreSQL Connections
      ↓
PostgreSQL Handles Concurrency
```

## Expected Outcome

### Before

- All DB operations serialized.
- API throughput limited by one connection.
- High latency under concurrent requests.

### After

- Multiple requests can hit DB concurrently.
- Better throughput and lower latency.
- Scales naturally with traffic.
- More production-like backend architecture.

## Internal Concurrency Model

Pool uses synchronization only for connection checkout:

```text
Thread A -> borrow Conn1
Thread B -> borrow Conn2
Thread C -> waits if pool exhausted
```

Important:

Mutex is still used internally by the pool, but only to protect pool bookkeeping, not the full DB query.

## Implementation Steps

### Step 1: Add Pool Dependency

For Diesel:

```toml
diesel = { version = "...", features = ["postgres", "r2d2"] }
```

### Step 2: Define Pool Type

```rust
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
```

### Step 3: Create Pool During Startup

```rust
let manager = ConnectionManager::<PgConnection>::new(database_url);

let pool = Pool::builder()
    .max_size(10)
    .build(manager)
    .unwrap();
```

### Step 4: Replace App State

- Old: `Arc<Mutex<Store>>`
- New: `DbPool`

### Step 5: Refactor Store Methods

Store methods should accept connection reference:

```rust
pub fn create_user(
    conn: &mut PgConnection,
    ...
)
```

instead of owning/storing the connection internally.

### Step 6: Borrow Connection Per Request

Inside handler:

```rust
let mut conn = pool.get()?;
create_user(&mut conn, ...)?;
```

Connection auto-returns when dropped.

## Suggested Pool Sizes

Initial recommendation for this project:

| Service | Pool Size |
| --- | --- |
| API | 10 |
| Producer | 2-5 |
| Consumer | 5-10 |

Tune later based on workload.

## Future Improvements After Pooling

Once pool migration is done:

1. Add Transactions
For multi-step DB operations.

Examples:
- user signup + related inserts
- atomic balance updates
- website + tick creation workflows

2. Add Query Ownership Checks
Ensure users can only modify their own websites.

3. Move Secrets to Environment Variables
- JWT secret
- DB URL
- Redis URL

4. Hash Passwords
Replace plaintext storage.

## Conceptual Takeaways

### Mutex vs Pool

**Mutex Around Connection**

- Protects one shared connection object.
- Serializes all DB work.

**Pool**

- Protects pool metadata only.
- Allows many concurrent DB operations.

### Responsibility Split

**Application**

Responsible for:

- Borrowing connection safely.
- Returning connection to pool.

**PostgreSQL**

Responsible for:

- Data consistency
- Row locking
- Transactions
- Concurrent writes/reads

## Final Summary

Connection pooling improves concurrency by allowing multiple database connections to be reused safely across requests.
Instead of serializing all database access through one mutex-protected connection, requests borrow temporary connections from a shared pool.
PostgreSQL handles data consistency internally, while the pool manages connection allocation.
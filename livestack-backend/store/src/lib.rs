use crate::config::Config;
use diesel::r2d2::{ConnectionManager, Pool, PoolError, PooledConnection};
use diesel::{Connection, ConnectionError, PgConnection};
use std::ops::{Deref, DerefMut};

pub mod config;
pub mod models;
pub mod schema;
pub mod url_guard;

pub use diesel::result::DatabaseErrorKind;
pub use diesel::result::Error as DbError;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = PooledConnection<ConnectionManager<PgConnection>>;

const API_POOL_SIZE: u32 = 20;
const WORKER_POOL_SIZE: u32 = 4;

pub enum StoreConnection {
    Direct(PgConnection),
    Pooled(DbConnection),
}

impl Deref for StoreConnection {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        match self {
            StoreConnection::Direct(conn) => conn,
            StoreConnection::Pooled(conn) => conn,
        }
    }
}

impl DerefMut for StoreConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            StoreConnection::Direct(conn) => conn,
            StoreConnection::Pooled(conn) => conn,
        }
    }
}

pub struct Store {
    pub conn: StoreConnection,
}
pub use chrono::NaiveDateTime;

impl Store {
    pub fn conn(&mut self) -> &mut PgConnection {
        &mut self.conn
    }

    /// Pool for the API process, which serves many requests concurrently.
    /// Built eagerly: a database that isn't reachable at startup is a
    /// configuration problem worth failing loudly on.
    pub fn pool() -> Result<DbPool, PoolError> {
        let config = Config::default();
        let manager = ConnectionManager::<PgConnection>::new(config.db_url);

        Pool::builder().max_size(API_POOL_SIZE).build(manager)
    }

    /// Pool for the background workers (producer, consumer, notifiers).
    ///
    /// Two deliberate differences from [`Store::pool`]:
    ///
    /// - **Lazy.** Connections are opened on first use, so a worker started
    ///   while Postgres is still coming up boots anyway instead of exiting.
    /// - **Small.** Each worker handles one message at a time, and Postgres'
    ///   `max_connections` (100 by default) is shared across *every* process;
    ///   giving each worker an API-sized pool is what exhausts it.
    ///
    /// Either way, r2d2 validates a connection when it's checked out and
    /// replaces dead ones — which is what a long-lived single `PgConnection`
    /// could never do. A worker that lost its connection to Postgres used to
    /// stay broken until someone restarted it.
    pub fn worker_pool() -> DbPool {
        let config = Config::default();
        let manager = ConnectionManager::<PgConnection>::new(config.db_url);

        Pool::builder()
            .max_size(WORKER_POOL_SIZE)
            .build_unchecked(manager)
    }

    pub fn from_pool(pool: &DbPool) -> Result<Self, PoolError> {
        let connection = pool.get()?;
        Ok(Self {
            conn: StoreConnection::Pooled(connection),
        })
    }

    pub fn default() -> Result<Self, ConnectionError> {
        let config = Config::default();
        let connection = PgConnection::establish(&config.db_url)?;
        Ok(Self {
            conn: StoreConnection::Direct(connection),
        })
    }
}

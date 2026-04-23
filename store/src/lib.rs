use crate::config::Config;
use diesel::r2d2::{ConnectionManager, Pool, PoolError, PooledConnection};
use diesel::{Connection, ConnectionError, PgConnection};
use std::ops::{Deref, DerefMut};

pub mod config;
pub mod models;
pub mod schema;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = PooledConnection<ConnectionManager<PgConnection>>;

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

    pub fn pool() -> Result<DbPool, PoolError> {
        let config = Config::default();
        let manager = ConnectionManager::<PgConnection>::new(config.db_url);

        Pool::builder().max_size(20).build(manager)
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

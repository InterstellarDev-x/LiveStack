use crate::config::Config;
use diesel::{Connection, ConnectionError, PgConnection};

pub mod config;
pub mod models;
pub mod schema;
pub struct Store {
    pub conn: PgConnection,
}

impl Store {
    pub fn default() -> Result<Self, ConnectionError> {
        let config = Config::default();
        let connection = PgConnection::establish(&config.db_url)?;
        Ok(Self { conn: connection })
    }
}




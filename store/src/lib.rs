use diesel::{Connection, ConnectionError, PgConnection};
use crate::config::Config;

pub mod config;
pub mod schema;
pub mod models;
pub struct  Store {
  pub  conn : PgConnection
}

impl  Store {

   pub fn default() -> Result<Self , ConnectionError > {
        let config = Config::default();
         let connection = PgConnection::establish(&config.db_url)?;
        Ok(Self {
            conn : connection
        })
    }

      pub fn create_user(&self){
        print!("crated user")
    }


     pub fn create_website(&self) -> String{
        format!("created Website")
    }    
}


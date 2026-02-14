

pub struct Config {
    pub db_url: String,
}

impl Default for Config {
    
    fn default() -> Self {
         
        return Self { db_url : "postgres://postgres:mysecretpassword@localhost:5432/better-uptime".into() };
    }
}

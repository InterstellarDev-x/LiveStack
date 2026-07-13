pub struct Config {
    pub db_url: String,
}

impl Default for Config {
    fn default() -> Self {
        dotenvy::dotenv().ok();

        return Self {
            db_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://postgres:mysecretpassword@localhost:5432/better-uptime".into()
            }),
        };
    }
}

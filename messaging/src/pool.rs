use deadpool_redis::{Config, Runtime};


pub fn create_pool() -> deadpool_redis::Pool {

    let cfg = Config::from_url("redis://127.0.0.1/");
    cfg.create_pool(Some(Runtime::Tokio1)).unwrap()
}


//Config parses URL

//Pool manager created

//Connections created lazily (on demand)
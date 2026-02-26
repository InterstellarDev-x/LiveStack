use redis::RedisResult;

pub struct  StreamService {
    redis : redis::Client
}


impl  StreamService {
    pub fn new(url :&str) ->  RedisResult<Self>{
        Ok(
            Self { redis: redis::Client::open(url)? }
        )
    }


    pub fn get_conn(&self) -> RedisResult<redis::Connection> {
       return self.redis.get_connection();
    }
}



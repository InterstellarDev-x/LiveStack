use redis::{Commands, RedisResult, pipe, streams::StreamMaxlen};
use store::models::website::Website;

use crate::BETTERUPTIME;

pub struct StreamService {
    redis: redis::Client,
}

impl StreamService {
    pub fn new(url: &str) -> RedisResult<Self> {
        Ok(Self {
            redis: redis::Client::open(url)?,
        })
    }

    pub fn get_conn(&self) -> RedisResult<redis::Connection> {
        return self.redis.get_connection();
    }


    pub fn add_records(&self) -> RedisResult<()> {

        println!("started broadcasting ");

       let mut con = self.get_conn().unwrap();
       let maxlen = StreamMaxlen::Approx(1000); 
    
        // a stream whose records have two fields
        for _ in 0..1 {
            let _: () = con.xadd_maxlen(
                BETTERUPTIME,
                maxlen, // how many latest entries should we keep in redis while adding
                "*",
                &[
                    ("url", String::from("www.google.com")),
                    ("url", "www.facebook.com".into()),
                ],
            )?;
        }
       
        let len: usize = con.xlen(BETTERUPTIME).unwrap();
    
        // println!("{}" , con.xlen::<_, usize>(STREAMS[0]).unwrap());
        println!("thie size is {}", len);

        println!("started broadcasting ");
    
        Ok(())
    }


    pub fn add_records_batch(&self, websites: &[Website]) -> RedisResult<()> {
        let mut con = self.get_conn()?;
        let mut p = pipe();
    
        for site in websites {
            p.cmd("XADD")
                .arg(BETTERUPTIME)
                .arg("MAXLEN")
                .arg("~")
                .arg(1000)
                .arg("*")
                .arg(&[
                    ("url", site.url.clone()),
                    ("id" , site.id.clone())
                ]);
        }
    
        let _: () = p.query(&mut con)?;

        let len: usize = con.xlen(BETTERUPTIME).unwrap();
    
        // println!("{}" , con.xlen::<_, usize>(STREAMS[0]).unwrap());
        println!("thie size is {}", len);

        Ok(())
    }
    
}

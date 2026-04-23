use redis::{
    Commands, RedisResult, Value, pipe,
    streams::{StreamId, StreamKey, StreamMaxlen, StreamReadOptions, StreamReadReply},
};
use store::models::website::Website;

use crate::BETTERUPTIME;

#[derive(Debug)]
pub struct WebsiteCheckMessage {
    pub stream_id: String,
    pub website_id: String,
    pub url: String,
}

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
        self.redis.get_connection()
    }

    pub fn add_records(&self) -> RedisResult<()> {
        println!("started broadcasting");

        let mut con = self.get_conn()?;
        let maxlen = StreamMaxlen::Approx(1000);

        let _: () = con.xadd_maxlen(
            BETTERUPTIME,
            maxlen,
            "*",
            &[
                ("url", String::from("https://www.google.com")),
                ("id", String::from("demo")),
            ],
        )?;

        let len: usize = con.xlen(BETTERUPTIME)?;
        println!("stream size is {}", len);

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
                .arg(&[("url", site.url.clone()), ("id", site.id.clone())]);
        }

        let _: () = p.query(&mut con)?;

        let len: usize = con.xlen(BETTERUPTIME)?;
        println!(
            "queued {} website checks, stream size is {}",
            websites.len(),
            len
        );

        Ok(())
    }

    pub fn ensure_consumer_group(&self, group_name: &str) -> RedisResult<()> {
        let mut con = self.get_conn()?;
        let created: RedisResult<()> = con.xgroup_create_mkstream(BETTERUPTIME, group_name, "0");

        match created {
            Ok(()) => Ok(()),
            Err(err) if err.to_string().contains("BUSYGROUP") => Ok(()),
            Err(err) => Err(err),
        }
    }

    pub fn read_group_records(
        &self,
        group_name: &str,
        consumer_name: &str,
        count: usize,
        block_millis: usize,
    ) -> RedisResult<Vec<WebsiteCheckMessage>> {
        let mut con = self.get_conn()?;
        let opts = StreamReadOptions::default()
            .block(block_millis)
            .count(count)
            .group(group_name, consumer_name);

        let reply: Option<StreamReadReply> = con.xread_options(&[BETTERUPTIME], &[">"], &opts)?;
        let Some(reply) = reply else {
            return Ok(Vec::new());
        };

        let mut messages = Vec::new();

        for StreamKey { ids, .. } in reply.keys {
            for StreamId { id, map, .. } in ids {
                let website_id = map.get("id").and_then(value_to_string);
                let url = map.get("url").and_then(value_to_string);

                if let (Some(website_id), Some(url)) = (website_id, url) {
                    messages.push(WebsiteCheckMessage {
                        stream_id: id,
                        website_id,
                        url,
                    });
                }
            }
        }

        Ok(messages)
    }

    pub fn ack_records(&self, group_name: &str, ids: &[String]) -> RedisResult<usize> {
        if ids.is_empty() {
            return Ok(0);
        }

        let mut con = self.get_conn()?;
        let ids: Vec<&str> = ids.iter().map(String::as_str).collect();
        con.xack(BETTERUPTIME, group_name, &ids)
    }
}

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::BulkString(bytes) => String::from_utf8(bytes.clone()).ok(),
        Value::SimpleString(value) => Some(value.clone()),
        _ => None,
    }
}

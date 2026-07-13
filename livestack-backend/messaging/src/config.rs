use redis::{
    Commands, RedisResult, Value, pipe,
    streams::{
        StreamAutoClaimOptions, StreamAutoClaimReply, StreamId, StreamKey, StreamMaxlen,
        StreamReadOptions, StreamReadReply,
    },
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
        let mut malformed_ids = Vec::new();

        for StreamKey { ids, .. } in reply.keys {
            for stream_id in ids {
                match stream_id_to_message(stream_id) {
                    Ok(message) => messages.push(message),
                    Err(stream_id) => malformed_ids.push(stream_id),
                }
            }
        }

        ack_malformed_records(&mut con, group_name, &malformed_ids)?;
        Ok(messages)
    }

    pub fn claim_pending_records(
        &self,
        group_name: &str,
        consumer_name: &str,
        min_idle_millis: usize,
        count: usize,
    ) -> RedisResult<Vec<WebsiteCheckMessage>> {
        let mut con = self.get_conn()?;
        let opts = StreamAutoClaimOptions::default().count(count);

        let reply: StreamAutoClaimReply = con.xautoclaim_options(
            BETTERUPTIME,
            group_name,
            consumer_name,
            min_idle_millis,
            "0-0",
            opts,
        )?;

        let mut messages = Vec::new();
        let mut malformed_ids = Vec::new();

        for stream_id in reply.claimed {
            match stream_id_to_message(stream_id) {
                Ok(message) => messages.push(message),
                Err(stream_id) => malformed_ids.push(stream_id),
            }
        }

        ack_malformed_records(&mut con, group_name, &malformed_ids)?;
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

fn stream_id_to_message(stream_id: StreamId) -> Result<WebsiteCheckMessage, String> {
    let id = stream_id.id;
    let website_id = stream_id.map.get("id").and_then(value_to_string);
    let url = stream_id.map.get("url").and_then(value_to_string);

    match (website_id, url) {
        (Some(website_id), Some(url)) => Ok(WebsiteCheckMessage {
            stream_id: id,
            website_id,
            url,
        }),
        _ => Err(id),
    }
}

fn ack_malformed_records(
    con: &mut redis::Connection,
    group_name: &str,
    ids: &[String],
) -> RedisResult<()> {
    if ids.is_empty() {
        return Ok(());
    }

    let ids: Vec<&str> = ids.iter().map(String::as_str).collect();
    let acked: usize = con.xack(BETTERUPTIME, group_name, &ids)?;
    eprintln!("acked {acked} malformed stream records");

    Ok(())
}

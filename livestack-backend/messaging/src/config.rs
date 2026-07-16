use chrono::Utc;
use redis::{
    Commands, RedisResult, Value, pipe,
    streams::{
        StreamAutoClaimOptions, StreamAutoClaimReply, StreamClaimReply, StreamId, StreamKey,
        StreamMaxlen, StreamPendingCountReply, StreamReadOptions, StreamReadReply,
    },
};
use store::models::website::Website;
use uuid::Uuid;

use crate::{BETTERUPTIME, WEBSITE_ALERTS, WEBSITE_ALERTS_DLQ};

#[derive(Debug)]
pub struct WebsiteCheckMessage {
    pub stream_id: String,
    pub website_id: String,
    pub url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertStatus {
    Down,
    Recovered,
}

impl AlertStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertStatus::Down => "down",
            AlertStatus::Recovered => "recovered",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "down" => Some(AlertStatus::Down),
            "recovered" => Some(AlertStatus::Recovered),
            _ => None,
        }
    }
}

/// A website status transition (Down or Recovered), published once per
/// incident open/resolve rather than once per check. Carries a stable
/// `alert_id` so downstream webhook payloads can be deduplicated by the
/// receiver, since consumer-group delivery is at-least-once, not exactly-once.
#[derive(Debug, Clone)]
pub struct AlertMessage {
    pub stream_id: String,
    pub alert_id: String,
    /// The incident this transition belongs to. Empty for records published
    /// before incidents existed and still sitting in the stream.
    pub incident_id: String,
    pub website_id: String,
    pub url: String,
    pub status: AlertStatus,
    pub region_id: String,
    pub response_time_ms: i32,
    pub occurred_at: String,
    /// What the failing check saw, e.g. "HTTP 503" or "timeout". Empty on
    /// pre-incident records.
    pub cause: String,
    /// Total outage length, present only on Recovered alerts.
    pub downtime_seconds: Option<i64>,
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
        ensure_consumer_group_on(&mut con, BETTERUPTIME, group_name)
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

        ack_malformed_records(&mut con, BETTERUPTIME, group_name, &malformed_ids)?;
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
        let reply = autoclaim(
            &mut con,
            BETTERUPTIME,
            group_name,
            consumer_name,
            min_idle_millis,
            count,
        )?;

        let mut messages = Vec::new();
        let mut malformed_ids = Vec::new();

        for stream_id in reply.claimed {
            match stream_id_to_message(stream_id) {
                Ok(message) => messages.push(message),
                Err(stream_id) => malformed_ids.push(stream_id),
            }
        }

        ack_malformed_records(&mut con, BETTERUPTIME, group_name, &malformed_ids)?;
        Ok(messages)
    }

    pub fn ack_records(&self, group_name: &str, ids: &[String]) -> RedisResult<usize> {
        let mut con = self.get_conn()?;
        ack(&mut con, BETTERUPTIME, group_name, ids)
    }

    /// Publishes one status-transition event. Called by the check-consumer
    /// only when an incident opens or resolves, not on every tick.
    /// `downtime_seconds` is only meaningful on Recovered alerts.
    pub fn publish_alert(
        &self,
        website_id: &str,
        url: &str,
        status: AlertStatus,
        region_id: &str,
        response_time_ms: i32,
        incident_id: &str,
        cause: &str,
        downtime_seconds: Option<i64>,
    ) -> RedisResult<()> {
        let mut con = self.get_conn()?;
        let alert_id = Uuid::new_v4().to_string();
        let occurred_at = Utc::now().to_rfc3339();
        let downtime = downtime_seconds
            .map(|secs| secs.to_string())
            .unwrap_or_default();

        con.xadd_maxlen(
            WEBSITE_ALERTS,
            StreamMaxlen::Approx(1000),
            "*",
            &[
                ("alert_id", alert_id.as_str()),
                ("incident_id", incident_id),
                ("website_id", website_id),
                ("url", url),
                ("status", status.as_str()),
                ("region_id", region_id),
                ("response_time_ms", &response_time_ms.to_string()),
                ("occurred_at", occurred_at.as_str()),
                ("cause", cause),
                ("downtime_seconds", downtime.as_str()),
            ],
        )
    }

    pub fn ensure_alert_consumer_group(&self, group_name: &str) -> RedisResult<()> {
        let mut con = self.get_conn()?;
        ensure_consumer_group_on(&mut con, WEBSITE_ALERTS, group_name)
    }

    pub fn read_alert_records(
        &self,
        group_name: &str,
        consumer_name: &str,
        count: usize,
        block_millis: usize,
    ) -> RedisResult<Vec<AlertMessage>> {
        let mut con = self.get_conn()?;
        let opts = StreamReadOptions::default()
            .block(block_millis)
            .count(count)
            .group(group_name, consumer_name);

        let reply: Option<StreamReadReply> =
            con.xread_options(&[WEBSITE_ALERTS], &[">"], &opts)?;
        let Some(reply) = reply else {
            return Ok(Vec::new());
        };

        let mut messages = Vec::new();
        let mut malformed_ids = Vec::new();

        for StreamKey { ids, .. } in reply.keys {
            for stream_id in ids {
                match stream_id_to_alert_message(stream_id) {
                    Ok(message) => messages.push(message),
                    Err(stream_id) => malformed_ids.push(stream_id),
                }
            }
        }

        ack_malformed_records(&mut con, WEBSITE_ALERTS, group_name, &malformed_ids)?;
        Ok(messages)
    }

    /// Unlimited-retry reclaim, used by email-worker: a slow mail server is
    /// worth retrying indefinitely, there's no dead-letter path here.
    pub fn claim_alert_records(
        &self,
        group_name: &str,
        consumer_name: &str,
        min_idle_millis: usize,
        count: usize,
    ) -> RedisResult<Vec<AlertMessage>> {
        let mut con = self.get_conn()?;
        let reply = autoclaim(
            &mut con,
            WEBSITE_ALERTS,
            group_name,
            consumer_name,
            min_idle_millis,
            count,
        )?;

        let mut messages = Vec::new();
        let mut malformed_ids = Vec::new();

        for stream_id in reply.claimed {
            match stream_id_to_alert_message(stream_id) {
                Ok(message) => messages.push(message),
                Err(stream_id) => malformed_ids.push(stream_id),
            }
        }

        ack_malformed_records(&mut con, WEBSITE_ALERTS, group_name, &malformed_ids)?;
        Ok(messages)
    }

    /// Reclaim with a dead-letter cap, used by webhook-worker: a webhook URL
    /// that has failed `max_attempts` times (and is currently idle, i.e. not
    /// mid-delivery by another consumer) is moved to `website-alerts-dlq` and
    /// acked instead of being retried forever, so one permanently broken
    /// endpoint can't starve other sites' alerts out of the PEL.
    pub fn claim_alert_records_with_deadletter(
        &self,
        group_name: &str,
        consumer_name: &str,
        min_idle_millis: usize,
        count: usize,
        max_attempts: usize,
    ) -> RedisResult<Vec<AlertMessage>> {
        let mut con = self.get_conn()?;

        let pending: StreamPendingCountReply =
            con.xpending_count(WEBSITE_ALERTS, group_name, "-", "+", count)?;

        let deadletter_ids: Vec<String> = pending
            .ids
            .into_iter()
            .filter(|entry| {
                entry.times_delivered >= max_attempts
                    && entry.last_delivered_ms >= min_idle_millis
            })
            .map(|entry| entry.id)
            .collect();

        if !deadletter_ids.is_empty() {
            deadletter_records(&mut con, group_name, consumer_name, &deadletter_ids)?;
        }

        let reply = autoclaim(
            &mut con,
            WEBSITE_ALERTS,
            group_name,
            consumer_name,
            min_idle_millis,
            count,
        )?;

        let mut messages = Vec::new();
        let mut malformed_ids = Vec::new();

        for stream_id in reply.claimed {
            match stream_id_to_alert_message(stream_id) {
                Ok(message) => messages.push(message),
                Err(stream_id) => malformed_ids.push(stream_id),
            }
        }

        ack_malformed_records(&mut con, WEBSITE_ALERTS, group_name, &malformed_ids)?;
        Ok(messages)
    }

    pub fn ack_alert_records(&self, group_name: &str, ids: &[String]) -> RedisResult<usize> {
        let mut con = self.get_conn()?;
        ack(&mut con, WEBSITE_ALERTS, group_name, ids)
    }
}

fn ensure_consumer_group_on(
    con: &mut redis::Connection,
    stream_key: &str,
    group_name: &str,
) -> RedisResult<()> {
    let created: RedisResult<()> = con.xgroup_create_mkstream(stream_key, group_name, "0");

    match created {
        Ok(()) => Ok(()),
        Err(err) if err.to_string().contains("BUSYGROUP") => Ok(()),
        Err(err) => Err(err),
    }
}

fn autoclaim(
    con: &mut redis::Connection,
    stream_key: &str,
    group_name: &str,
    consumer_name: &str,
    min_idle_millis: usize,
    count: usize,
) -> RedisResult<StreamAutoClaimReply> {
    let opts = StreamAutoClaimOptions::default().count(count);
    con.xautoclaim_options(
        stream_key,
        group_name,
        consumer_name,
        min_idle_millis,
        "0-0",
        opts,
    )
}

fn ack(
    con: &mut redis::Connection,
    stream_key: &str,
    group_name: &str,
    ids: &[String],
) -> RedisResult<usize> {
    if ids.is_empty() {
        return Ok(0);
    }

    let ids: Vec<&str> = ids.iter().map(String::as_str).collect();
    con.xack(stream_key, group_name, &ids)
}

/// Claims delivery-exhausted alerts (they're pending, so claiming just
/// confirms current ownership), forwards them to the DLQ stream, and acks
/// them off the source stream's PEL so they stop being redelivered.
fn deadletter_records(
    con: &mut redis::Connection,
    group_name: &str,
    consumer_name: &str,
    ids: &[String],
) -> RedisResult<()> {
    let claim_reply: StreamClaimReply =
        con.xclaim(WEBSITE_ALERTS, group_name, consumer_name, 0, ids)?;

    for stream_id in claim_reply.ids {
        if let Ok(message) = stream_id_to_alert_message(stream_id) {
            let fields = [
                ("alert_id", message.alert_id),
                ("incident_id", message.incident_id),
                ("website_id", message.website_id),
                ("url", message.url),
                ("status", message.status.as_str().to_string()),
                ("region_id", message.region_id),
                (
                    "response_time_ms",
                    message.response_time_ms.to_string(),
                ),
                ("occurred_at", message.occurred_at),
                ("cause", message.cause),
                (
                    "downtime_seconds",
                    message
                        .downtime_seconds
                        .map(|secs| secs.to_string())
                        .unwrap_or_default(),
                ),
            ];
            let _: RedisResult<()> =
                con.xadd_maxlen(WEBSITE_ALERTS_DLQ, StreamMaxlen::Approx(1000), "*", &fields);
        }
    }

    let acked: usize = ack(con, WEBSITE_ALERTS, group_name, ids)?;
    eprintln!("dead-lettered {acked} alert(s) after exhausting delivery attempts");

    Ok(())
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

fn stream_id_to_alert_message(stream_id: StreamId) -> Result<AlertMessage, String> {
    let id = stream_id.id.clone();
    let map = &stream_id.map;

    let alert_id = map.get("alert_id").and_then(value_to_string);
    let website_id = map.get("website_id").and_then(value_to_string);
    let url = map.get("url").and_then(value_to_string);
    let status = map
        .get("status")
        .and_then(value_to_string)
        .and_then(|s| AlertStatus::parse(&s));
    let region_id = map.get("region_id").and_then(value_to_string);
    let response_time_ms = map
        .get("response_time_ms")
        .and_then(value_to_string)
        .and_then(|s| s.parse::<i32>().ok());
    let occurred_at = map.get("occurred_at").and_then(value_to_string);

    // Absent on records published before incidents existed - default rather
    // than treat those still-pending records as malformed.
    let incident_id = map
        .get("incident_id")
        .and_then(value_to_string)
        .unwrap_or_default();
    let cause = map
        .get("cause")
        .and_then(value_to_string)
        .unwrap_or_default();
    let downtime_seconds = map
        .get("downtime_seconds")
        .and_then(value_to_string)
        .and_then(|s| s.parse::<i64>().ok());

    match (
        alert_id,
        website_id,
        url,
        status,
        region_id,
        response_time_ms,
        occurred_at,
    ) {
        (
            Some(alert_id),
            Some(website_id),
            Some(url),
            Some(status),
            Some(region_id),
            Some(response_time_ms),
            Some(occurred_at),
        ) => Ok(AlertMessage {
            stream_id: id,
            alert_id,
            incident_id,
            website_id,
            url,
            status,
            region_id,
            response_time_ms,
            occurred_at,
            cause,
            downtime_seconds,
        }),
        _ => Err(id),
    }
}

fn ack_malformed_records(
    con: &mut redis::Connection,
    stream_key: &str,
    group_name: &str,
    ids: &[String],
) -> RedisResult<()> {
    if ids.is_empty() {
        return Ok(());
    }

    let ids: Vec<&str> = ids.iter().map(String::as_str).collect();
    let acked: usize = con.xack(stream_key, group_name, &ids)?;
    eprintln!("acked {acked} malformed stream records");

    Ok(())
}

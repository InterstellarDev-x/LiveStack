use std::{env, sync::Arc, time::Duration};

use messaging::config::{StreamService, WebsiteCheckMessage};
use store::{Store, models::website::WebsiteStatusEnum};
use tokio::time::Instant;
const REDIS_URL: &str = "redis://127.0.0.1/";
const CONSUMER_GROUP: &str = "uptime-checkers";
const READ_BATCH_SIZE: usize = 10;
const READ_BLOCK_MILLIS: usize = 5000;
const CLAIM_MIN_IDLE_MILLIS: usize = 5000;
const DEFAULT_REGION_ID: &str = "india";
const DEFAULT_REGION_NAME: &str = "India";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let stream = Arc::new(StreamService::new(REDIS_URL)?);
    stream.ensure_consumer_group(CONSUMER_GROUP)?;

    let consumer_name =
        env::var("CONSUMER_NAME").unwrap_or_else(|_| format!("consumer-{}", std::process::id()));
    let region_id = env::var("REGION_ID").unwrap_or_else(|_| DEFAULT_REGION_ID.to_string());
    let region_name = env::var("REGION_NAME").unwrap_or_else(|_| DEFAULT_REGION_NAME.to_string());

    let mut store = Store::default()?;
    store.ensure_region(region_id.clone(), region_name)?;

    println!("consumer {consumer_name} listening on group {CONSUMER_GROUP}");

    loop {
        let stream_clone = Arc::clone(&stream);
        let consumer_name_clone = consumer_name.clone();

        let messages = tokio::task::spawn_blocking(move || {
            stream_clone.read_group_records(
                CONSUMER_GROUP,
                &consumer_name_clone,
                READ_BATCH_SIZE,
                READ_BLOCK_MILLIS,
            )
        })
        .await??;

        let messages = if messages.is_empty() {
            let stream_clone = Arc::clone(&stream);
            let consumer_name_clone = consumer_name.clone();

            tokio::task::spawn_blocking(move || {
                stream_clone.claim_pending_records(
                    CONSUMER_GROUP,
                    &consumer_name_clone,
                    CLAIM_MIN_IDLE_MILLIS,
                    READ_BATCH_SIZE,
                )
            })
            .await??
        } else {
            messages
        };

        if messages.is_empty() {
            continue;
        }

        let mut processed_ids = Vec::new();

        for message in messages {
            let tick = check_website(&client, &message, &region_id).await;

            store.create_website_tick(
                tick.website_id,
                tick.region_id,
                tick.response_time_ms,
                tick.status,
            )?;
            processed_ids.push(message.stream_id);
        }

        stream.ack_records(CONSUMER_GROUP, &processed_ids)?;
        println!("processed {} website checks", processed_ids.len());
    }
}

struct WebsiteCheck {
    website_id: String,
    region_id: String,
    response_time_ms: i32,
    status: WebsiteStatusEnum,
}

async fn check_website(
    client: &reqwest::Client,
    message: &WebsiteCheckMessage,
    region_id: &str,
) -> WebsiteCheck {
    let started_at = Instant::now();
    let response = client.get(normalize_url(&message.url)).send().await;
    let elapsed_ms = started_at.elapsed().as_millis().min(i32::MAX as u128) as i32;

    let status = match response {
        Ok(response) if response.status().is_success() => WebsiteStatusEnum::Up,
        Ok(_) | Err(_) => WebsiteStatusEnum::Down,
    };

    WebsiteCheck {
        website_id: message.website_id.clone(),
        region_id: region_id.to_string(),
        response_time_ms: elapsed_ms,
        status,
    }
}

fn normalize_url(url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("https://{}", url)
    }
}

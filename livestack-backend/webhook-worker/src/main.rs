use std::{env, sync::Arc, time::Duration};

use hmac::{Hmac, Mac};
use messaging::config::{AlertMessage, StreamService};
use serde::Serialize;
use sha2::Sha256;
use store::Store;

type HmacSha256 = Hmac<Sha256>;

const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1/";
const CONSUMER_GROUP: &str = "webhook-notifiers";
const READ_BATCH_SIZE: usize = 10;
const READ_BLOCK_MILLIS: usize = 5000;
const CLAIM_MIN_IDLE_MILLIS: usize = 5000;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
/// Deliveries beyond this count (while idle, i.e. not mid-attempt by another
/// consumer) are moved to the DLQ instead of retried again - see
/// StreamService::claim_alert_records_with_deadletter.
const MAX_DELIVERY_ATTEMPTS: usize = 5;
const SIGNATURE_HEADER: &str = "X-LiveStack-Signature";

#[derive(Serialize)]
struct WebhookPayload<'a> {
    alert_id: &'a str,
    /// Stable across the down and recovered alerts of one outage, so
    /// receivers can pair them up. Empty on pre-incident alerts.
    incident_id: &'a str,
    website_id: &'a str,
    url: &'a str,
    status: &'a str,
    region_id: &'a str,
    response_time_ms: i32,
    occurred_at: &'a str,
    /// What the failing check saw ("HTTP 503", "timeout", ...); empty when unknown.
    cause: &'a str,
    /// Present only on recovered alerts.
    downtime_seconds: Option<i64>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| DEFAULT_REDIS_URL.to_string());
    let stream = Arc::new(StreamService::new(&redis_url)?);
    stream.ensure_alert_consumer_group(CONSUMER_GROUP)?;

    let consumer_name = env::var("CONSUMER_NAME")
        .unwrap_or_else(|_| format!("webhook-worker-{}", std::process::id()));

    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()?;

    let mut store = Store::default()?;

    println!("webhook-worker {consumer_name} listening on group {CONSUMER_GROUP}");

    loop {
        let stream_clone = Arc::clone(&stream);
        let consumer_name_clone = consumer_name.clone();

        let messages = tokio::task::spawn_blocking(move || {
            stream_clone.read_alert_records(
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
                stream_clone.claim_alert_records_with_deadletter(
                    CONSUMER_GROUP,
                    &consumer_name_clone,
                    CLAIM_MIN_IDLE_MILLIS,
                    READ_BATCH_SIZE,
                    MAX_DELIVERY_ATTEMPTS,
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
            match handle_alert(&mut store, &client, &message).await {
                Ok(()) => processed_ids.push(message.stream_id),
                // Left unacked: PEL reclaim retries it, up to MAX_DELIVERY_ATTEMPTS.
                Err(err) => eprintln!(
                    "webhook-worker failed to deliver alert {}: {err}",
                    message.alert_id
                ),
            }
        }

        stream.ack_alert_records(CONSUMER_GROUP, &processed_ids)?;
        println!("delivered {} webhook alerts", processed_ids.len());
    }
}

async fn handle_alert(
    store: &mut Store,
    client: &reqwest::Client,
    alert: &AlertMessage,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(config) = store.get_notification_config(&alert.website_id)? else {
        return Ok(()); // no config for this website - nothing to deliver
    };
    if !config.webhook_enabled {
        return Ok(()); // owner has paused webhook alerts for this website
    }
    let (Some(webhook_url), Some(secret)) = (config.webhook_url, config.webhook_secret) else {
        return Ok(()); // webhook not configured - nothing to deliver
    };

    let payload = WebhookPayload {
        alert_id: &alert.alert_id,
        incident_id: &alert.incident_id,
        website_id: &alert.website_id,
        url: &alert.url,
        status: alert.status.as_str(),
        region_id: &alert.region_id,
        response_time_ms: alert.response_time_ms,
        occurred_at: &alert.occurred_at,
        cause: &alert.cause,
        downtime_seconds: alert.downtime_seconds,
    };
    let body = serde_json::to_vec(&payload)?;
    let signature = sign(&secret, &body);

    let response = client
        .post(&webhook_url)
        .header("Content-Type", "application/json")
        .header(SIGNATURE_HEADER, signature)
        .body(body)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("webhook returned status {}", response.status()).into());
    }

    Ok(())
}

/// Hex-encoded HMAC-SHA256 over the raw JSON body, so a receiver can verify
/// authenticity the same way Stripe/GitHub webhook signatures work.
fn sign(secret: &str, body: &[u8]) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key length");
    mac.update(body);
    hex::encode(mac.finalize().into_bytes())
}

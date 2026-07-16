use std::{env, sync::Arc, time::Duration};

use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::header::ContentType,
    transport::smtp::authentication::Credentials,
};
use messaging::config::{AlertMessage, AlertStatus, StreamService};
use store::Store;

const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1/";
const CONSUMER_GROUP: &str = "email-notifiers";
const READ_BATCH_SIZE: usize = 10;
const READ_BLOCK_MILLIS: usize = 5000;
const CLAIM_MIN_IDLE_MILLIS: usize = 5000;
const SEND_TIMEOUT: Duration = Duration::from_secs(10);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| DEFAULT_REDIS_URL.to_string());
    let stream = Arc::new(StreamService::new(&redis_url)?);
    stream.ensure_alert_consumer_group(CONSUMER_GROUP)?;

    let consumer_name = env::var("CONSUMER_NAME")
        .unwrap_or_else(|_| format!("email-worker-{}", std::process::id()));

    let mailer = build_mailer()?;
    let from_address = env::var("SMTP_FROM").expect("SMTP_FROM must be set");

    let mut store = Store::default()?;

    println!("email-worker {consumer_name} listening on group {CONSUMER_GROUP}");

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
                stream_clone.claim_alert_records(
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
            match handle_alert(&mut store, &mailer, &from_address, &message).await {
                Ok(()) => processed_ids.push(message.stream_id),
                // Left unacked: PEL reclaim will retry it after CLAIM_MIN_IDLE_MILLIS.
                Err(err) => eprintln!(
                    "email-worker failed to process alert {}: {err}",
                    message.alert_id
                ),
            }
        }

        stream.ack_alert_records(CONSUMER_GROUP, &processed_ids)?;
        println!("emailed {} alerts", processed_ids.len());
    }
}

fn build_mailer() -> Result<AsyncSmtpTransport<Tokio1Executor>, Box<dyn std::error::Error>> {
    let host = env::var("SMTP_HOST").expect("SMTP_HOST must be set");
    let username = env::var("SMTP_USER").expect("SMTP_USER must be set");
    let password = env::var("SMTP_PASS").expect("SMTP_PASS must be set");

    let creds = Credentials::new(username, password);

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&host)?
        .credentials(creds)
        .timeout(Some(SEND_TIMEOUT))
        .build();

    Ok(mailer)
}

async fn handle_alert(
    store: &mut Store,
    mailer: &AsyncSmtpTransport<Tokio1Executor>,
    from_address: &str,
    alert: &AlertMessage,
) -> Result<(), Box<dyn std::error::Error>> {
    let owner_id = store.get_website_owner_user_id(&alert.website_id)?;
    let Some(to_address) = store.get_user_email(&owner_id)? else {
        return Ok(()); // no email on file yet - nothing to send, not a failure
    };

    let (subject, body) = render_alert(alert);

    let email = Message::builder()
        .from(from_address.parse()?)
        .to(to_address.parse()?)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body)?;

    mailer.send(email).await?;
    Ok(())
}

fn render_alert(alert: &AlertMessage) -> (String, String) {
    match alert.status {
        AlertStatus::Down => {
            let cause = if alert.cause.is_empty() {
                String::new()
            } else {
                format!(" Cause: {}.", alert.cause)
            };
            (
                format!("{} is down", alert.url),
                format!(
                    "{} stopped responding at {} (region: {}).{}",
                    alert.url, alert.occurred_at, alert.region_id, cause
                ),
            )
        }
        AlertStatus::Recovered => {
            let downtime = alert
                .downtime_seconds
                .map(|secs| format!(" after {} of downtime", format_duration(secs)))
                .unwrap_or_default();
            (
                format!("{} has recovered", alert.url),
                format!(
                    "{} is responding again as of {}{} (region: {}, {} ms response time).",
                    alert.url, alert.occurred_at, downtime, alert.region_id, alert.response_time_ms
                ),
            )
        }
    }
}

/// "45s", "10m 32s", "2h 05m" - coarse on purpose, it's for a human email.
fn format_duration(total_seconds: i64) -> String {
    let total_seconds = total_seconds.max(0);
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{hours}h {minutes:02}m")
    } else if minutes > 0 {
        format!("{minutes}m {seconds:02}s")
    } else {
        format!("{seconds}s")
    }
}

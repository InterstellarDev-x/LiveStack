use std::{env, sync::Arc, time::Duration};

use curl::easy::Easy;
use messaging::config::{AlertStatus, StreamService, WebsiteCheckMessage};
use store::{
    DbPool, Store,
    models::{
        incident::Incident,
        website::{NewWebsiteTickTiming, WebsiteStatusEnum},
    },
};

const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1/";
const CONSUMER_GROUP: &str = "uptime-checkers";
const READ_BATCH_SIZE: usize = 10;
const READ_BLOCK_MILLIS: usize = 5000;
const CLAIM_MIN_IDLE_MILLIS: usize = 5000;
const DEFAULT_REGION_ID: &str = "india";
const DEFAULT_REGION_NAME: &str = "India";
const CHECK_TIMEOUT: Duration = Duration::from_secs(10);
/// Backoff after an infrastructure error (Redis unreachable, etc.) so a
/// prolonged outage doesn't turn into a hot retry loop.
const ERROR_BACKOFF: Duration = Duration::from_secs(5);
/// Consecutive failed checks required before an incident opens (and the down
/// alert fires). One curl blip never pages anyone; the cost is one extra
/// check interval of alert latency.
const DOWN_TICKS_TO_OPEN_INCIDENT: usize = 2;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| DEFAULT_REDIS_URL.to_string());
    let stream = Arc::new(StreamService::new(&redis_url)?);
    stream.ensure_consumer_group(CONSUMER_GROUP)?;

    let consumer_name =
        env::var("CONSUMER_NAME").unwrap_or_else(|_| format!("consumer-{}", std::process::id()));
    let region_id = env::var("REGION_ID").unwrap_or_else(|_| DEFAULT_REGION_ID.to_string());
    let region_name = env::var("REGION_NAME").unwrap_or_else(|_| DEFAULT_REGION_NAME.to_string());

    // Pooled rather than a single connection held for the process' lifetime,
    // so losing the connection to Postgres costs one batch instead of
    // permanently stopping every uptime check. See `Store::worker_pool`.
    let pool = Store::worker_pool();
    Store::from_pool(&pool)?.ensure_region(region_id.clone(), region_name)?;

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
        .await?;

        // Redis being briefly unreachable is a transient condition, not a
        // reason to exit: unacked messages stay in the PEL and are picked up
        // again once it recovers.
        let messages = match messages {
            Ok(messages) => messages,
            Err(err) => {
                eprintln!("consumer failed to read from the check stream: {err}");
                tokio::time::sleep(ERROR_BACKOFF).await;
                continue;
            }
        };

        let messages = if messages.is_empty() {
            let stream_clone = Arc::clone(&stream);
            let consumer_name_clone = consumer_name.clone();

            match tokio::task::spawn_blocking(move || {
                stream_clone.claim_pending_records(
                    CONSUMER_GROUP,
                    &consumer_name_clone,
                    CLAIM_MIN_IDLE_MILLIS,
                    READ_BATCH_SIZE,
                )
            })
            .await?
            {
                Ok(messages) => messages,
                Err(err) => {
                    eprintln!("consumer failed to reclaim pending checks: {err}");
                    tokio::time::sleep(ERROR_BACKOFF).await;
                    continue;
                }
            }
        } else {
            messages
        };

        if messages.is_empty() {
            continue;
        }

        let mut processed_ids = Vec::new();

        for message in messages {
            let region_id = region_id.clone();
            // curl::easy::Easy is a blocking API, so the check runs on a blocking thread.
            let check = match tokio::task::spawn_blocking(move || {
                check_website(&message, &region_id)
            })
            .await
            {
                Ok(check) => check,
                // A panicking check would otherwise take the process down and
                // the message would be redelivered into the same panic.
                Err(err) => {
                    eprintln!("consumer check task failed: {err}");
                    continue;
                }
            };

            let website_id = check.website_id.clone();
            let stream_id = check.stream_id.clone();

            match record_check(&pool, &check) {
                Ok(alert) => {
                    processed_ids.push(stream_id);

                    if let Some((alert_status, incident, downtime_seconds)) = alert {
                        // Best-effort: a Redis hiccup on the alert stream shouldn't
                        // fail the whole batch or block acking the check message.
                        if let Err(err) = stream.publish_alert(
                            &check.website_id,
                            &check.url,
                            alert_status,
                            &check.region_id,
                            check.response_time_ms,
                            &incident.id,
                            &incident.cause,
                            downtime_seconds,
                        ) {
                            eprintln!("failed to publish alert for {website_id}: {err}");
                        }
                    }
                }
                // Left unacked on purpose: the PEL reclaim path retries this
                // one check later, rather than one bad row killing the
                // consumer and every other site's checks with it.
                Err(err) => {
                    eprintln!("consumer failed to record check for {website_id}: {err}")
                }
            }
        }

        if let Err(err) = stream.ack_records(CONSUMER_GROUP, &processed_ids) {
            eprintln!("consumer failed to ack {} checks: {err}", processed_ids.len());
            continue;
        }
        println!("processed {} website checks", processed_ids.len());
    }
}

/// Persists one completed check and advances that website's incident state.
/// Returns the transition to alert on, if this check caused one.
fn record_check(
    pool: &DbPool,
    check: &WebsiteCheck,
) -> Result<Option<(AlertStatus, Incident, Option<i64>)>, Box<dyn std::error::Error>> {
    let mut store = Store::from_pool(pool)?;

    store.create_website_tick(
        check.website_id.clone(),
        check.region_id.clone(),
        check.response_time_ms,
        check.status,
        check.timing,
    )?;

    run_incident_state_machine(&mut store, &check.website_id, check.status, &check.cause)
}

struct WebsiteCheck {
    stream_id: String,
    website_id: String,
    url: String,
    region_id: String,
    response_time_ms: i32,
    status: WebsiteStatusEnum,
    /// What the failing check saw ("HTTP 503", "timeout", ...); empty on Up.
    cause: String,
    timing: NewWebsiteTickTiming,
}

/// Runs after the tick is inserted. Opens an incident once
/// DOWN_TICKS_TO_OPEN_INCIDENT consecutive checks have failed, resolves the
/// open incident on the first successful check. Both operations are atomic
/// in the database (partial unique index / conditional update), so only the
/// consumer that actually performed the transition gets `Some(..)` back and
/// publishes the alert — safe with any number of concurrent consumers.
fn run_incident_state_machine(
    store: &mut Store,
    website_id: &str,
    status: WebsiteStatusEnum,
    cause: &str,
) -> Result<Option<(AlertStatus, Incident, Option<i64>)>, Box<dyn std::error::Error>> {
    match status {
        WebsiteStatusEnum::Down => {
            // The just-inserted tick is included in this window.
            let recent =
                store.get_latest_ticks_by_website_id(website_id, DOWN_TICKS_TO_OPEN_INCIDENT as i64)?;
            let confirmed = recent.len() >= DOWN_TICKS_TO_OPEN_INCIDENT
                && recent
                    .iter()
                    .all(|previous| previous.status == WebsiteStatusEnum::Down);
            if !confirmed {
                return Ok(None);
            }

            Ok(store
                .open_incident(website_id, cause)?
                .map(|incident| (AlertStatus::Down, incident, None)))
        }
        WebsiteStatusEnum::Up => Ok(store.resolve_incident(website_id)?.map(|incident| {
            let downtime_seconds = incident
                .resolved_at
                .map(|resolved| (resolved - incident.started_at).num_seconds());
            (AlertStatus::Recovered, incident, downtime_seconds)
        })),
        WebsiteStatusEnum::Unknown => Ok(None),
    }
}

/// The cumulative curl timings for one request. Every field is measured from
/// the same absolute start (before any redirect is followed), not reset per
/// redirect hop — a field only advances when that milestone actually happens
/// again (e.g. a redirect to the same host reuses the DNS answer, so
/// `namelookup_ms` won't move on the second hop). See `learning.md`.
#[derive(Default)]
struct CurlTiming {
    namelookup_ms: i32,
    connect_ms: i32,
    appconnect_ms: i32,
    starttransfer_ms: i32,
    total_ms: i32,
}

impl From<CurlTiming> for NewWebsiteTickTiming {
    fn from(timing: CurlTiming) -> Self {
        let handshake_done = timing.appconnect_ms.max(timing.connect_ms);
        let transfer_start = timing.starttransfer_ms.max(handshake_done);

        NewWebsiteTickTiming {
            dns_time_ms: timing.namelookup_ms,
            connection_time_ms: (timing.connect_ms - timing.namelookup_ms).max(0),
            // appconnect_ms is 0 for plain HTTP (no TLS handshake happened).
            tls_time_ms: if timing.appconnect_ms > 0 {
                (timing.appconnect_ms - timing.connect_ms).max(0)
            } else {
                0
            },
            // Time to first byte: the server "thinking" once the connection was ready.
            waiting_time_ms: (transfer_start - handshake_done).max(0),
            // Time spent actually streaming the body in after the first byte arrived.
            data_transfer_time_ms: (timing.total_ms - transfer_start).max(0),
        }
    }
}

fn check_website(message: &WebsiteCheckMessage, region_id: &str) -> WebsiteCheck {
    let url = normalize_url(&message.url);

    let (status, cause, timing) = match perform_check(&url) {
        Ok((response_code, timing)) if (200..300).contains(&response_code) => {
            (WebsiteStatusEnum::Up, String::new(), timing)
        }
        Ok((response_code, timing)) => (
            WebsiteStatusEnum::Down,
            format!("HTTP {response_code}"),
            timing,
        ),
        Err(err) => (
            WebsiteStatusEnum::Down,
            cause_from_curl_error(&err),
            CurlTiming::default(),
        ),
    };

    WebsiteCheck {
        stream_id: message.stream_id.clone(),
        website_id: message.website_id.clone(),
        url: message.url.clone(),
        region_id: region_id.to_string(),
        response_time_ms: timing.total_ms,
        status,
        cause,
        timing: timing.into(),
    }
}

/// Human-readable reason for the incident record; the exact curl code is kept
/// as a fallback so uncommon failures stay diagnosable.
fn cause_from_curl_error(err: &curl::Error) -> String {
    if err.is_operation_timedout() {
        "timeout".to_string()
    } else if err.is_couldnt_resolve_host() {
        "DNS failure".to_string()
    } else if err.is_couldnt_connect() {
        "connection failed".to_string()
    } else if err.is_ssl_connect_error() {
        "TLS handshake failed".to_string()
    } else {
        format!("curl error {}", err.code())
    }
}

/// Runs the GET request and returns the HTTP status code alongside curl's
/// per-phase timings. `Err` covers DNS failures, connection refusals, and
/// timeouts, which the caller treats as a `Down` check.
fn perform_check(url: &str) -> Result<(u32, CurlTiming), curl::Error> {
    let mut easy = Easy::new();
    easy.url(url)?;
    easy.get(true)?;
    easy.timeout(CHECK_TIMEOUT)?;
    easy.follow_location(true)?;
    // We only need status + timings, so discard the response body.
    easy.write_function(|data| Ok(data.len()))?;
    easy.perform()?;

    let response_code = easy.response_code()?;
    let timing = CurlTiming {
        namelookup_ms: duration_to_ms(easy.namelookup_time()?),
        connect_ms: duration_to_ms(easy.connect_time()?),
        appconnect_ms: duration_to_ms(easy.appconnect_time()?),
        starttransfer_ms: duration_to_ms(easy.starttransfer_time()?),
        total_ms: duration_to_ms(easy.total_time()?),
    };

    Ok((response_code, timing))
}

fn duration_to_ms(duration: Duration) -> i32 {
    duration.as_millis().min(i32::MAX as u128) as i32
}

fn normalize_url(url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("https://{}", url)
    }
}

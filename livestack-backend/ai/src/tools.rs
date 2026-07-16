//! Tool definitions and execution for the LiveStack assistant.
//!
//! Every tool takes `user_id` from the authenticated request, never from the
//! model's arguments — the model cannot query another user's data.
//!
//! Design notes:
//! - Arguments are deserialized into typed structs (`deny_unknown_fields`)
//!   before any tool body runs, so bad args produce a clean error the model
//!   can react to, never a crash.
//! - Each tool returns a [`ToolOutcome`] with two outputs: `content` (what
//!   the LLM sees) and `details` (a compact structured summary for UI/logs).
//! - [`is_parallel_safe`] marks tools the loop may run concurrently. All
//!   current tools are read-only queries, so all are parallel-safe; a future
//!   mutating tool opts out by returning `false` and the loop will run it
//!   sequentially.

use async_openai::{
    error::OpenAIError,
    types::{ChatCompletionTool, ChatCompletionToolArgs, ChatCompletionToolType, FunctionObjectArgs},
};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use store::{DbPool, Store, models::website::WebsiteStatusEnum};

const MAX_RAW_TICKS: usize = 30;
const MAX_METRIC_HOURS: i64 = 168; // one week
const MAX_INCIDENTS: i64 = 50;

/// A tool's two outputs, kept intentionally separate: `content` is fed back
/// to the model as the tool result; `details` goes to the UI/event stream.
pub struct ToolOutcome {
    pub content: Value,
    pub details: Value,
}

/// Whether the loop may run this tool concurrently with others from the same
/// model turn. Only read-only tools qualify; unknown names are treated as
/// unsafe and run sequentially (where they fail with a clean error).
pub fn is_parallel_safe(name: &str) -> bool {
    matches!(
        name,
        "list_websites" | "get_website_metrics" | "get_incidents" | "get_status_pages"
    )
}

pub fn definitions() -> Result<Vec<ChatCompletionTool>, OpenAIError> {
    let list_websites = FunctionObjectArgs::default()
        .name("list_websites")
        .description(
            "List every website the user monitors, each with its current status and most recent check.",
        )
        .parameters(json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        }))
        .build()?;

    let get_website_metrics = FunctionObjectArgs::default()
        .name("get_website_metrics")
        .description(
            "Performance metrics for one website over a recent window: uptime, per-phase latency \
             averages (dns/connect/tls/ttfb/transfer), and a sample of the most recent raw checks.",
        )
        .parameters(json!({
            "type": "object",
            "properties": {
                "website_id": {
                    "type": "string",
                    "description": "Id of the website, from list_websites"
                },
                "hours": {
                    "type": "integer",
                    "description": "How many hours back to look (1-168). Default 24.",
                    "minimum": 1,
                    "maximum": 168
                }
            },
            "required": ["website_id"],
            "additionalProperties": false
        }))
        .build()?;

    let get_incidents = FunctionObjectArgs::default()
        .name("get_incidents")
        .description(
            "Outage history, newest first. Omit website_id to get incidents across all of the \
             user's websites. resolved_at null means the outage is still ongoing.",
        )
        .parameters(json!({
            "type": "object",
            "properties": {
                "website_id": {
                    "type": "string",
                    "description": "Optional: restrict to one website"
                },
                "limit": {
                    "type": "integer",
                    "description": "Max incidents to return (1-50). Default 20.",
                    "minimum": 1,
                    "maximum": 50
                }
            },
            "additionalProperties": false
        }))
        .build()?;

    let get_status_pages = FunctionObjectArgs::default()
        .name("get_status_pages")
        .description("List the public status pages the user has published.")
        .parameters(json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        }))
        .build()?;

    [list_websites, get_website_metrics, get_incidents, get_status_pages]
        .into_iter()
        .map(|function| {
            ChatCompletionToolArgs::default()
                .r#type(ChatCompletionToolType::Function)
                .function(function)
                .build()
        })
        .collect()
}

/// Validates raw model-supplied arguments into a typed struct before any tool
/// body runs. Unknown fields and wrong types come back as clean errors.
fn parse_args<T: DeserializeOwned>(raw: &str) -> Result<T, String> {
    let raw = if raw.trim().is_empty() { "{}" } else { raw };
    serde_json::from_str(raw).map_err(|e| format!("invalid tool arguments: {e}"))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NoArgs {}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MetricsArgs {
    website_id: String,
    hours: Option<i64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IncidentsArgs {
    website_id: Option<String>,
    limit: Option<i64>,
}

/// Runs one tool call. Errors are returned as strings so the caller can hand
/// them back to the model as a recoverable tool result.
pub fn execute(
    pool: &DbPool,
    user_id: &str,
    name: &str,
    arguments: &str,
) -> Result<ToolOutcome, String> {
    let mut store =
        Store::from_pool(pool).map_err(|_| "database temporarily unavailable".to_string())?;

    match name {
        "list_websites" => {
            let _args: NoArgs = parse_args(arguments)?;
            list_websites(&mut store, user_id)
        }
        "get_website_metrics" => {
            let args: MetricsArgs = parse_args(arguments)?;
            get_website_metrics(&mut store, user_id, args)
        }
        "get_incidents" => {
            let args: IncidentsArgs = parse_args(arguments)?;
            get_incidents(&mut store, user_id, args)
        }
        "get_status_pages" => {
            let _args: NoArgs = parse_args(arguments)?;
            get_status_pages(&mut store, user_id)
        }
        other => Err(format!("unknown tool: {other}")),
    }
}

fn list_websites(store: &mut Store, user_id: &str) -> Result<ToolOutcome, String> {
    let websites = store
        .get_websites_by_user_id(user_id.to_string())
        .map_err(|e| e.to_string())?;

    let mut out = Vec::with_capacity(websites.len());
    for site in websites {
        let latest = store
            .get_latest_tick_by_website_id(&site.id)
            .map_err(|e| e.to_string())?;

        out.push(json!({
            "id": site.id,
            "url": site.url,
            "monitored_since": site.time_added.format("%Y-%m-%d %H:%M:%S").to_string(),
            "current_status": latest.as_ref().map(|t| status_str(t.status)),
            "latest_check": latest.map(|t| json!({
                "at": t.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                "response_time_ms": t.response_time_ms,
            })),
        }));
    }

    Ok(ToolOutcome {
        details: json!({ "count": out.len() }),
        content: json!({ "websites": out }),
    })
}

fn get_website_metrics(
    store: &mut Store,
    user_id: &str,
    args: MetricsArgs,
) -> Result<ToolOutcome, String> {
    let hours = args.hours.unwrap_or(24).clamp(1, MAX_METRIC_HOURS);
    let website_id = args.website_id;

    let owned = store
        .website_belongs_to_user(&website_id, user_id)
        .map_err(|e| e.to_string())?;
    if !owned {
        return Err("no such website for this user".to_string());
    }

    let since = chrono::Utc::now().naive_utc() - chrono::Duration::hours(hours);
    let mut ticks = store
        .get_ticks_since(&website_id, since)
        .map_err(|e| e.to_string())?;
    // newest first, so "recent_checks" below samples the latest ones
    ticks.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let total = ticks.len();
    let up = ticks
        .iter()
        .filter(|t| t.status == WebsiteStatusEnum::Up)
        .count();
    let down = ticks
        .iter()
        .filter(|t| t.status == WebsiteStatusEnum::Down)
        .count();

    // Latency phases only mean something for successful checks.
    let up_ticks: Vec<_> = ticks
        .iter()
        .filter(|t| t.status == WebsiteStatusEnum::Up)
        .collect();
    let avg = |f: fn(&&store::models::website::WebsiteTick) -> i32| -> Option<i64> {
        if up_ticks.is_empty() {
            None
        } else {
            Some(up_ticks.iter().map(|t| f(t) as i64).sum::<i64>() / up_ticks.len() as i64)
        }
    };

    let recent: Vec<Value> = ticks
        .iter()
        .take(MAX_RAW_TICKS)
        .map(|t| {
            json!({
                "at": t.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                "status": status_str(t.status),
                "response_time_ms": t.response_time_ms,
                "dns_time_ms": t.dns_time_ms,
                "connection_time_ms": t.connection_time_ms,
                "tls_time_ms": t.tls_time_ms,
                "waiting_time_ms": t.waiting_time_ms,
                "data_transfer_time_ms": t.data_transfer_time_ms,
            })
        })
        .collect();

    let uptime_percent = if up + down > 0 {
        Some((up as f64 / (up + down) as f64 * 10000.0).round() / 100.0)
    } else {
        None
    };

    Ok(ToolOutcome {
        details: json!({
            "website_id": website_id,
            "window_hours": hours,
            "checks_total": total,
            "uptime_percent": uptime_percent,
        }),
        content: json!({
            "website_id": website_id,
            "window_hours": hours,
            "checks_total": total,
            "checks_up": up,
            "checks_down": down,
            "uptime_percent": uptime_percent,
            "averages_ms_over_up_checks": {
                "response_time": avg(|t| t.response_time_ms),
                "dns": avg(|t| t.dns_time_ms),
                "connection": avg(|t| t.connection_time_ms),
                "tls": avg(|t| t.tls_time_ms),
                "waiting_ttfb": avg(|t| t.waiting_time_ms),
                "data_transfer": avg(|t| t.data_transfer_time_ms),
            },
            "max_response_time_ms": up_ticks.iter().map(|t| t.response_time_ms).max(),
            "recent_checks_newest_first": recent,
        }),
    })
}

fn get_incidents(
    store: &mut Store,
    user_id: &str,
    args: IncidentsArgs,
) -> Result<ToolOutcome, String> {
    let limit = args.limit.unwrap_or(20).clamp(1, MAX_INCIDENTS);

    let incidents: Vec<Value> = match args.website_id.as_deref() {
        Some(website_id) => store
            .get_incidents_for_owner(website_id, user_id, limit)
            .map_err(|e| match e {
                store::DbError::NotFound => "no such website for this user".to_string(),
                other => other.to_string(),
            })?
            .into_iter()
            .map(|incident| incident_json(&incident, None))
            .collect(),
        None => store
            .get_incidents_for_user(user_id, limit)
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|(incident, url)| incident_json(&incident, Some(url)))
            .collect(),
    };

    let ongoing = incidents
        .iter()
        .filter(|i| i.get("ongoing") == Some(&Value::Bool(true)))
        .count();

    Ok(ToolOutcome {
        details: json!({ "count": incidents.len(), "ongoing": ongoing }),
        content: json!({ "incidents": incidents }),
    })
}

fn get_status_pages(store: &mut Store, user_id: &str) -> Result<ToolOutcome, String> {
    let pages: Vec<Value> = store
        .get_status_pages_by_user(user_id)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|page| {
            json!({
                "id": page.id,
                "slug": page.slug,
                "title": page.title,
                "created_at": page.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            })
        })
        .collect();

    Ok(ToolOutcome {
        details: json!({ "count": pages.len() }),
        content: json!({ "status_pages": pages }),
    })
}

fn incident_json(incident: &store::models::incident::Incident, url: Option<String>) -> Value {
    json!({
        "id": incident.id,
        "website_id": incident.website_id,
        "website_url": url,
        "started_at": incident.started_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        "resolved_at": incident
            .resolved_at
            .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        "ongoing": incident.resolved_at.is_none(),
        "cause": incident.cause,
    })
}

fn status_str(status: WebsiteStatusEnum) -> &'static str {
    match status {
        WebsiteStatusEnum::Up => "Up",
        WebsiteStatusEnum::Down => "Down",
        WebsiteStatusEnum::Unknown => "Unknown",
    }
}

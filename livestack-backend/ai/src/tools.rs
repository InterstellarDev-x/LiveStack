//! Tool definitions and execution for the LiveStack assistant.
//!
//! Every tool takes `user_id` from the authenticated request, never from the
//! model's arguments — the model cannot query another user's data.
//!
//! # Registry
//!
//! Every tool is one [`ToolSpec`] entry in [`REGISTRY`]: its schema, whether
//! it's safe to run concurrently, and — via [`Confirmation`] — whether it
//! mutates data and, if so, its confirmation-preview function.
//! [`definitions`], [`execute`], [`is_parallel_safe`],
//! [`requires_confirmation`], and [`describe_action`] are all thin lookups
//! over this one array, so adding a tool never means updating a second list
//! by memory. Coupling `confirmation` to the entry itself (rather than a
//! separate name-based allow-list) also means a mutating tool can't be
//! registered without deciding it: `ToolSpec { .. }` is a struct literal
//! with no default for `confirmation`, so the compiler refuses to build a
//! new entry that skips the field.
//!
//! Arguments are deserialized into typed structs (`deny_unknown_fields`)
//! before any tool body runs, so bad args produce a clean error the model
//! can react to, never a crash.
//!
//! Each tool returns a [`ToolOutcome`] with two outputs: `content` (what the
//! LLM sees) and `details` (a compact structured summary for UI/logs).

use async_openai::{
    error::OpenAIError,
    types::{ChatCompletionTool, ChatCompletionToolArgs, ChatCompletionToolType, FunctionObjectArgs},
};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use store::{DbPool, Store, models::website::WebsiteStatusEnum, url_guard};

const MAX_RAW_TICKS: usize = 30;
const MAX_METRIC_HOURS: i64 = 168; // one week
const MAX_INCIDENTS: i64 = 50;

/// A tool's two outputs, kept intentionally separate: `content` is fed back
/// to the model as the tool result; `details` goes to the UI/event stream.
pub struct ToolOutcome {
    pub content: Value,
    pub details: Value,
}

/// Whether a tool mutates data and, if so, how to describe a pending call of
/// it before it runs. Bundled into one enum (rather than a bool plus a
/// separately-optional function) so it's impossible to mark a tool as
/// mutating without also giving it a description function.
enum Confirmation {
    NotRequired,
    Required(fn(&DbPool, &str, &str) -> Result<String, String>),
}

/// Everything the agent loop needs to expose, schedule, gate, and run one
/// tool. See the module docs for why this table replaces per-capability
/// match statements.
struct ToolSpec {
    name: &'static str,
    description: &'static str,
    parameters: fn() -> Value,
    /// Whether the loop may run this concurrently with other tools from the
    /// same model turn. Only read-only tools qualify.
    parallel_safe: bool,
    confirmation: Confirmation,
    execute: fn(&DbPool, &str, &str) -> Result<ToolOutcome, String>,
}

static REGISTRY: &[ToolSpec] = &[
    ToolSpec {
        name: "list_websites",
        description: "List every website the user monitors, each with its current status and most recent check.",
        parameters: || json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        }),
        parallel_safe: true,
        confirmation: Confirmation::NotRequired,
        execute: execute_list_websites,
    },
    ToolSpec {
        name: "get_website_metrics",
        description: "Performance metrics for one website over a recent window: uptime, per-phase latency \
             averages (dns/connect/tls/ttfb/transfer), and a sample of the most recent raw checks.",
        parameters: || json!({
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
        }),
        parallel_safe: true,
        confirmation: Confirmation::NotRequired,
        execute: execute_get_website_metrics,
    },
    ToolSpec {
        name: "get_incidents",
        description: "Outage history, newest first. Omit website_id to get incidents across all of the \
             user's websites. resolved_at null means the outage is still ongoing.",
        parameters: || json!({
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
        }),
        parallel_safe: true,
        confirmation: Confirmation::NotRequired,
        execute: execute_get_incidents,
    },
    ToolSpec {
        name: "get_status_pages",
        description: "List the public status pages the user has published.",
        parameters: || json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        }),
        parallel_safe: true,
        confirmation: Confirmation::NotRequired,
        execute: execute_get_status_pages,
    },
    ToolSpec {
        name: "create_website",
        description: "Add a new website for the user to monitor. Mutating action — the user must \
             explicitly confirm before this runs.",
        parameters: || json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "Full URL to monitor, e.g. https://example.com"
                }
            },
            "required": ["url"],
            "additionalProperties": false
        }),
        parallel_safe: false,
        confirmation: Confirmation::Required(describe_create_website),
        execute: execute_create_website,
    },
    ToolSpec {
        name: "update_website",
        description: "Change the URL of a website the user is already monitoring. Mutating action — the \
             user must explicitly confirm before this runs.",
        parameters: || json!({
            "type": "object",
            "properties": {
                "website_id": {
                    "type": "string",
                    "description": "Id of the website to update, from list_websites"
                },
                "url": {
                    "type": "string",
                    "description": "The new URL to monitor, e.g. https://example.com"
                }
            },
            "required": ["website_id", "url"],
            "additionalProperties": false
        }),
        parallel_safe: false,
        confirmation: Confirmation::Required(describe_update_website),
        execute: execute_update_website,
    },
    ToolSpec {
        name: "delete_website",
        description: "Permanently stop monitoring a website and delete its history. Destructive, \
             mutating action — the user must explicitly confirm before this runs.",
        parameters: || json!({
            "type": "object",
            "properties": {
                "website_id": {
                    "type": "string",
                    "description": "Id of the website to delete, from list_websites"
                }
            },
            "required": ["website_id"],
            "additionalProperties": false
        }),
        parallel_safe: false,
        confirmation: Confirmation::Required(describe_delete_website),
        execute: execute_delete_website,
    },
];

fn find(name: &str) -> Option<&'static ToolSpec> {
    REGISTRY.iter().find(|tool| tool.name == name)
}

/// Whether the loop may run this tool concurrently with others from the same
/// model turn. Unknown names are treated as unsafe, same as any other
/// unregistered tool failing closed.
pub fn is_parallel_safe(name: &str) -> bool {
    find(name).is_some_and(|tool| tool.parallel_safe)
}

/// Tools that mutate the user's data. The agent loop must never auto-execute
/// these — it has to pause, describe what it's about to do via
/// [`describe_action`], and only call [`execute`] once the user has
/// confirmed the exact same `(name, arguments)` pair.
pub fn requires_confirmation(name: &str) -> bool {
    find(name).is_some_and(|tool| matches!(tool.confirmation, Confirmation::Required(_)))
}

pub fn definitions() -> Result<Vec<ChatCompletionTool>, OpenAIError> {
    REGISTRY
        .iter()
        .map(|tool| {
            ChatCompletionToolArgs::default()
                .r#type(ChatCompletionToolType::Function)
                .function(
                    FunctionObjectArgs::default()
                        .name(tool.name)
                        .description(tool.description)
                        .parameters((tool.parameters)())
                        .build()?,
                )
                .build()
        })
        .collect()
}

/// Runs one tool call. Errors are returned as strings so the caller can hand
/// them back to the model as a recoverable tool result.
pub fn execute(pool: &DbPool, user_id: &str, name: &str, arguments: &str) -> Result<ToolOutcome, String> {
    let tool = find(name).ok_or_else(|| format!("unknown tool: {name}"))?;
    (tool.execute)(pool, user_id, arguments)
}

/// Human-readable summary of a pending mutating action, shown to the user
/// before it runs. Resolves display details (like a website's URL) from the
/// database rather than trusting the model's arguments, since this text is
/// what the user's confirmation decision is actually based on.
pub fn describe_action(pool: &DbPool, user_id: &str, name: &str, arguments: &str) -> Result<String, String> {
    match find(name).map(|tool| &tool.confirmation) {
        Some(Confirmation::Required(describe)) => describe(pool, user_id, arguments),
        Some(Confirmation::NotRequired) => Err(format!("{name} does not require confirmation")),
        None => Err(format!("unknown tool: {name}")),
    }
}

/// Opens a pooled connection and runs `f` against it. Shared by every tool's
/// `execute`/`describe`, so opening the connection lives in one place
/// instead of being repeated per tool.
fn with_store<T>(pool: &DbPool, f: impl FnOnce(&mut Store) -> Result<T, String>) -> Result<T, String> {
    let mut store =
        Store::from_pool(pool).map_err(|_| "database temporarily unavailable".to_string())?;
    f(&mut store)
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateWebsiteArgs {
    url: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateWebsiteArgs {
    website_id: String,
    url: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeleteWebsiteArgs {
    website_id: String,
}

fn execute_list_websites(pool: &DbPool, user_id: &str, arguments: &str) -> Result<ToolOutcome, String> {
    let _args: NoArgs = parse_args(arguments)?;
    with_store(pool, |store| list_websites(store, user_id))
}

fn execute_get_website_metrics(pool: &DbPool, user_id: &str, arguments: &str) -> Result<ToolOutcome, String> {
    let args: MetricsArgs = parse_args(arguments)?;
    with_store(pool, |store| get_website_metrics(store, user_id, args))
}

fn execute_get_incidents(pool: &DbPool, user_id: &str, arguments: &str) -> Result<ToolOutcome, String> {
    let args: IncidentsArgs = parse_args(arguments)?;
    with_store(pool, |store| get_incidents(store, user_id, args))
}

fn execute_get_status_pages(pool: &DbPool, user_id: &str, arguments: &str) -> Result<ToolOutcome, String> {
    let _args: NoArgs = parse_args(arguments)?;
    with_store(pool, |store| get_status_pages(store, user_id))
}

fn execute_create_website(pool: &DbPool, user_id: &str, arguments: &str) -> Result<ToolOutcome, String> {
    let args: CreateWebsiteArgs = parse_args(arguments)?;
    with_store(pool, |store| create_website(store, user_id, args))
}

fn execute_update_website(pool: &DbPool, user_id: &str, arguments: &str) -> Result<ToolOutcome, String> {
    let args: UpdateWebsiteArgs = parse_args(arguments)?;
    with_store(pool, |store| update_website(store, user_id, args))
}

fn execute_delete_website(pool: &DbPool, user_id: &str, arguments: &str) -> Result<ToolOutcome, String> {
    let args: DeleteWebsiteArgs = parse_args(arguments)?;
    with_store(pool, |store| delete_website(store, user_id, args))
}

fn describe_create_website(_pool: &DbPool, _user_id: &str, arguments: &str) -> Result<String, String> {
    let args: CreateWebsiteArgs = parse_args(arguments)?;
    Ok(format!("Add **{}** as a new monitor.", args.url))
}

fn describe_update_website(pool: &DbPool, user_id: &str, arguments: &str) -> Result<String, String> {
    let args: UpdateWebsiteArgs = parse_args(arguments)?;
    with_store(pool, |store| {
        let current = store
            .get_website_by_id(args.website_id.clone(), user_id)
            .map_err(|e| match e {
                store::DbError::NotFound => "no such website for this user".to_string(),
                other => other.to_string(),
            })?;
        Ok(format!(
            "Change the monitored URL for **{}** to **{}**.",
            current.website.url, args.url
        ))
    })
}

fn describe_delete_website(pool: &DbPool, user_id: &str, arguments: &str) -> Result<String, String> {
    let args: DeleteWebsiteArgs = parse_args(arguments)?;
    with_store(pool, |store| {
        let website = store
            .get_website_by_id(args.website_id.clone(), user_id)
            .map_err(|e| match e {
                store::DbError::NotFound => "no such website for this user".to_string(),
                other => other.to_string(),
            })?;
        Ok(format!(
            "**Permanently delete** the monitor for **{}**. This cannot be undone.",
            website.website.url
        ))
    })
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

fn create_website(
    store: &mut Store,
    user_id: &str,
    args: CreateWebsiteArgs,
) -> Result<ToolOutcome, String> {
    // The same guard the HTTP route uses, so a monitor created by the
    // assistant can't be a target the API would have rejected.
    let url = url_guard::normalize_monitor_url(&args.url).map_err(|e| e.message().to_string())?;

    let website = store
        .create_website(user_id.to_string(), url)
        .map_err(|e| e.to_string())?;

    Ok(ToolOutcome {
        details: json!({ "website_id": website.id, "url": website.url }),
        content: json!({ "created": true, "website_id": website.id, "url": website.url }),
    })
}

fn update_website(
    store: &mut Store,
    user_id: &str,
    args: UpdateWebsiteArgs,
) -> Result<ToolOutcome, String> {
    let url = url_guard::normalize_monitor_url(&args.url).map_err(|e| e.message().to_string())?;

    let website = store
        .update_by_id(args.website_id, url, user_id)
        .map_err(|e| match e {
            store::DbError::NotFound => "no such website for this user".to_string(),
            other => other.to_string(),
        })?;

    Ok(ToolOutcome {
        details: json!({ "website_id": website.id, "url": website.url }),
        content: json!({ "updated": true, "website_id": website.id, "url": website.url }),
    })
}

fn delete_website(
    store: &mut Store,
    user_id: &str,
    args: DeleteWebsiteArgs,
) -> Result<ToolOutcome, String> {
    let deleted = store
        .delete_by_id(args.website_id.clone(), user_id)
        .map_err(|e| e.to_string())?;

    if !deleted {
        return Err("no such website for this user".to_string());
    }

    Ok(ToolOutcome {
        details: json!({ "website_id": args.website_id, "deleted": true }),
        content: json!({ "deleted": true, "website_id": args.website_id }),
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

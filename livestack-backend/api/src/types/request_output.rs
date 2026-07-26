use serde::{Deserialize, Serialize};
use store::{
    NaiveDateTime,
    models::website::{WebsiteStatusEnum, WebsiteTick},
};

#[derive(Serialize, Deserialize)]
pub struct CreateWebsiteOutput {
    pub success: bool,
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct WebsiteOutput {
    pub id: String,
    pub url: String,
    pub user_id: String,
    pub time_added: NaiveDateTime,
}

#[derive(Serialize, Deserialize)]
pub struct WebsiteOutputWithTick {
    pub id: String,
    pub url: String,
    pub user_id: String,
    pub time_added: NaiveDateTime,
    pub website_tick: Option<WebsiteTick>,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteWebsiteOutput {
    pub success: bool,
}

#[derive(Serialize, Deserialize)]
pub struct WebsitesByUserOutput {
    pub websites: Vec<WebsiteOutput>,
}

#[derive(Serialize, Deserialize)]
pub struct WebsiteTicksOutput {
    pub ticks: Vec<WebsiteTick>,
}

#[derive(Serialize, Deserialize)]
pub struct SignUpOutput {
    pub success: bool,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct SignInpOutput {
    pub success: bool,
    pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct SetWebsiteWebhookOutput {
    pub success: bool,
    pub webhook_url: Option<String>,
    pub webhook_enabled: bool,
}

#[derive(Serialize, Deserialize)]
pub struct WebsiteWebhookOutput {
    pub webhook_url: Option<String>,
    pub webhook_secret: Option<String>,
    pub webhook_enabled: bool,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateEmailOutput {
    pub success: bool,
    pub email: String,
}

#[derive(Serialize, Deserialize)]
pub struct CurrentUserOutput {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub email_alerts_enabled: bool,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateEmailAlertsOutput {
    pub success: bool,
    pub email_alerts_enabled: bool,
}

#[derive(Serialize, Deserialize)]
pub struct StatusPageActionOutput {
    pub success: bool,
}

#[derive(Serialize, Deserialize)]
pub struct StatusPageOutput {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize)]
pub struct StatusPagesOutput {
    pub pages: Vec<StatusPageOutput>,
}

/// Owner-facing view of a published monitor - includes the real URL so the
/// owner can tell monitors apart while managing the page.
#[derive(Serialize, Deserialize)]
pub struct StatusPageMonitorOutput {
    pub website_id: String,
    pub url: String,
    pub display_name: String,
    pub sort_order: i32,
}

#[derive(Serialize, Deserialize)]
pub struct StatusPageDetailOutput {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub monitors: Vec<StatusPageMonitorOutput>,
}

/// Public view of a published monitor - deliberately excludes the real URL
/// and any ids, only the label the owner chose to show.
#[derive(Serialize, Deserialize)]
pub struct PublicStatusPageMonitorOutput {
    pub display_name: String,
    pub status: WebsiteStatusEnum,
    pub uptime_24h: Option<f64>,
    pub uptime_7d: Option<f64>,
    pub uptime_30d: Option<f64>,
}

/// `resolved_at: None` means the outage is still ongoing. `cause` is what the
/// failing check saw (e.g. "HTTP 503") - the same thing any visitor would
/// have seen, so it's safe to publish.
#[derive(Serialize, Deserialize)]
pub struct PublicStatusPageIncidentOutput {
    pub display_name: String,
    pub started_at: NaiveDateTime,
    pub resolved_at: Option<NaiveDateTime>,
    pub cause: String,
}

#[derive(Serialize, Deserialize)]
pub struct PublicStatusPageOutput {
    pub title: String,
    pub monitors: Vec<PublicStatusPageMonitorOutput>,
    /// Open incidents plus anything from the last 30 days, newest first.
    pub incidents: Vec<PublicStatusPageIncidentOutput>,
}

#[derive(Serialize, Deserialize)]
pub struct IncidentOutput {
    pub id: String,
    pub website_id: String,
    pub started_at: NaiveDateTime,
    pub resolved_at: Option<NaiveDateTime>,
    pub cause: String,
    /// Total outage length; None while the incident is still open.
    pub duration_seconds: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct WebsiteIncidentsOutput {
    pub incidents: Vec<IncidentOutput>,
}

/// One row of the account-wide incident feed - carries the website URL so
/// the list is readable without a second lookup per row.
#[derive(Serialize, Deserialize)]
pub struct UserIncidentOutput {
    pub id: String,
    pub website_id: String,
    pub url: String,
    pub started_at: NaiveDateTime,
    pub resolved_at: Option<NaiveDateTime>,
    pub cause: String,
    pub duration_seconds: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct UserIncidentsOutput {
    pub incidents: Vec<UserIncidentOutput>,
}

/// Worker-facing: tells the gateway whether it can call `/internal/ai/reply`
/// yet, or needs to show `pairing_code` to the chat and wait.
#[derive(Serialize, Deserialize)]
pub struct ResolveChannelLinkOutput {
    pub linked: bool,
    pub pairing_code: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ChannelAiReplyOutput {
    pub reply: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChannelLinkOutput {
    pub id: String,
    pub channel: String,
    pub channel_user_id: String,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize)]
pub struct ChannelLinksOutput {
    pub links: Vec<ChannelLinkOutput>,
}

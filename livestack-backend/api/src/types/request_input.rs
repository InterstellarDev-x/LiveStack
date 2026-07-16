use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CreateWebsiteInput {
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateWebsiteInput {
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct SetWebsiteWebhookInput {
    /// `None` (or omitted) clears the webhook.
    pub webhook_url: Option<String>,
    pub webhook_enabled: bool,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateEmailInput {
    pub email: String,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateEmailAlertsInput {
    pub enabled: bool,
}
#[derive(Serialize, Deserialize)]
pub struct SignupInput {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct SigninInput {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateStatusPageInput {
    pub slug: String,
    pub title: String,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateStatusPageInput {
    pub slug: String,
    pub title: String,
}

#[derive(Serialize, Deserialize)]
pub struct AddStatusPageMonitorInput {
    pub website_id: String,
    pub display_name: String,
    pub sort_order: i32,
}

#[derive(Serialize, Deserialize)]
pub struct AiChatMessageInput {
    /// "user" or "assistant"
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize)]
pub struct AiChatInput {
    /// Full conversation so far, oldest first; the API is stateless.
    pub messages: Vec<AiChatMessageInput>,
}

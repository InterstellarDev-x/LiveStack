use serde::{Deserialize, Serialize};

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
pub struct SignUpOutput {
    pub success: bool,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct SignInpOutput {
    pub success: bool,
    pub token: String,
}

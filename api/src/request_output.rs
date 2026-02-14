use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CreateWebsiteOutput {
    pub id: String,
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

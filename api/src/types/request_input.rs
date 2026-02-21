use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CreateWebsiteInput {
    pub url: String,
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

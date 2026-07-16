use crate::types::{request_input::AiChatInput, request_output::AiChatOutput};
use ai::{AiError, ChatMessage};
use poem::{
    Error, Request, handler,
    http::StatusCode,
    web::{Data, Json},
};
use store::DbPool;

use crate::middleware::auth::UserId;

/// The UserId the auth middleware inserted; 401 if the route isn't behind it.
fn authenticated_user(req: &Request) -> Result<String, Error> {
    req.extensions()
        .get::<UserId>()
        .map(|UserId(id)| id.clone())
        .ok_or_else(|| Error::from_status(StatusCode::UNAUTHORIZED))
}

#[handler]
pub async fn chat(
    Data(pool): Data<&DbPool>,
    req: &Request,
    Json(input): Json<AiChatInput>,
) -> Result<Json<AiChatOutput>, Error> {
    let user_id = authenticated_user(req)?;

    let history: Vec<ChatMessage> = input
        .messages
        .into_iter()
        .map(|m| ChatMessage {
            role: m.role,
            content: m.content,
        })
        .collect();

    let reply = ai::run_chat(pool, &user_id, history)
        .await
        .map_err(|err| match err {
            AiError::BadInput(_) => Error::from_status(StatusCode::BAD_REQUEST),
            AiError::Db(_) => Error::from_status(StatusCode::SERVICE_UNAVAILABLE),
            AiError::Upstream(msg) => {
                eprintln!("ai chat upstream error: {msg}");
                Error::from_status(StatusCode::BAD_GATEWAY)
            }
        })?;

    Ok(Json(AiChatOutput { reply }))
}

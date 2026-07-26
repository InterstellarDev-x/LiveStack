use ai::{ChatMessage, MAX_HISTORY_MESSAGES};
use poem::{
    Error, Request, handler,
    http::StatusCode,
    web::{Data, Json},
};
use store::{DbError, DbPool, Store};

use crate::{
    middleware::auth::UserId,
    types::{
        request_input::{ChannelAiReplyInput, LinkChannelInput, ResolveChannelLinkInput},
        request_output::{
            ChannelAiReplyOutput, ChannelLinkOutput, ChannelLinksOutput, ResolveChannelLinkOutput,
        },
    },
};

fn store_from_pool(pool: &DbPool) -> Result<Store, Error> {
    Store::from_pool(pool).map_err(|_| Error::from_status(StatusCode::SERVICE_UNAVAILABLE))
}

/// The UserId the auth middleware inserted; 401 if the route isn't behind it.
fn authenticated_user(req: &Request) -> Result<String, Error> {
    req.extensions()
        .get::<UserId>()
        .map(|UserId(id)| id.clone())
        .ok_or_else(|| Error::from_status(StatusCode::UNAUTHORIZED))
}

fn map_db_error(err: DbError) -> Error {
    match err {
        DbError::NotFound => Error::from_status(StatusCode::NOT_FOUND),
        _ => Error::from_status(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Called by the channel gateway before it knows whether a sender is linked
/// to a LiveStack account. Creates a pending link (with a fresh pairing
/// code) on first contact from a given (channel, channel_user_id) pair.
#[handler]
pub fn resolve_channel_link(
    Json(input): Json<ResolveChannelLinkInput>,
    Data(pool): Data<&DbPool>,
) -> Result<Json<ResolveChannelLinkOutput>, Error> {
    let mut store = store_from_pool(pool)?;

    let link = store
        .find_or_create_channel_link(&input.channel, &input.channel_user_id)
        .map_err(map_db_error)?;

    Ok(Json(match link.user_id {
        Some(_) => ResolveChannelLinkOutput {
            linked: true,
            pairing_code: None,
        },
        None => ResolveChannelLinkOutput {
            linked: false,
            pairing_code: Some(link.pairing_code),
        },
    }))
}

/// Called by the channel gateway once a sender is linked. Runs the message
/// through the agent with that channel's own conversation history, bypassing
/// the confirmation prompt the web UI shows (see `ai::run_chat_direct`).
#[handler]
pub async fn channel_ai_reply(
    Json(input): Json<ChannelAiReplyInput>,
    Data(pool): Data<&DbPool>,
) -> Result<Json<ChannelAiReplyOutput>, Error> {
    let pool = pool.clone();

    let (link_id, user_id, mut history) = {
        let mut store = store_from_pool(&pool)?;
        let link = store
            .get_channel_link(&input.channel, &input.channel_user_id)
            .map_err(map_db_error)?
            .ok_or_else(|| Error::from_status(StatusCode::NOT_FOUND))?;
        let user_id = link
            .user_id
            .ok_or_else(|| Error::from_status(StatusCode::NOT_FOUND))?;
        let history: Vec<ChatMessage> = serde_json::from_str(&link.history).unwrap_or_default();
        (link.id, user_id, history)
    };

    history.push(ChatMessage {
        role: "user".to_string(),
        content: input.message,
    });

    let reply = ai::run_chat_direct(&pool, &user_id, history.clone())
        .await
        .map_err(|err| {
            eprintln!("channel ai reply error: {err}");
            Error::from_status(StatusCode::BAD_GATEWAY)
        })?;

    history.push(ChatMessage {
        role: "assistant".to_string(),
        content: reply.clone(),
    });
    // Keep only the most recent turns, so the stored transcript stays within
    // what `run_chat_direct` will send upstream.
    if history.len() > MAX_HISTORY_MESSAGES {
        let drop = history.len() - MAX_HISTORY_MESSAGES;
        history.drain(0..drop);
    }

    let serialized = serde_json::to_string(&history).unwrap_or_else(|_| "[]".to_string());
    let mut store = store_from_pool(&pool)?;
    store
        .save_channel_link_history(&link_id, &serialized)
        .map_err(map_db_error)?;

    Ok(Json(ChannelAiReplyOutput { reply }))
}

/// Confirms the pairing code the user got from the bot, linking that chat to
/// their account.
#[handler]
pub fn link_channel(
    Json(input): Json<LinkChannelInput>,
    Data(pool): Data<&DbPool>,
    req: &Request,
) -> Result<Json<ChannelLinkOutput>, Error> {
    let user_id = authenticated_user(req)?;
    let mut store = store_from_pool(pool)?;

    let link = store
        .approve_channel_link(&input.pairing_code, &user_id)
        .map_err(map_db_error)?;

    Ok(Json(ChannelLinkOutput {
        id: link.id,
        channel: link.channel,
        channel_user_id: link.channel_user_id,
        created_at: link.created_at,
    }))
}

#[handler]
pub fn list_channel_links(
    Data(pool): Data<&DbPool>,
    req: &Request,
) -> Result<Json<ChannelLinksOutput>, Error> {
    let user_id = authenticated_user(req)?;
    let mut store = store_from_pool(pool)?;

    let links = store
        .list_channel_links_for_user(&user_id)
        .map_err(map_db_error)?
        .into_iter()
        .map(|link| ChannelLinkOutput {
            id: link.id,
            channel: link.channel,
            channel_user_id: link.channel_user_id,
            created_at: link.created_at,
        })
        .collect();

    Ok(Json(ChannelLinksOutput { links }))
}

#[handler]
pub fn delete_channel_link(
    poem::web::Path(link_id): poem::web::Path<String>,
    Data(pool): Data<&DbPool>,
    req: &Request,
) -> Result<(), Error> {
    let user_id = authenticated_user(req)?;
    let mut store = store_from_pool(pool)?;

    store
        .unlink_channel_link(&link_id, &user_id)
        .map_err(map_db_error)
}

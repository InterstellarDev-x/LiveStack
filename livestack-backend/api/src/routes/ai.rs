use std::time::Duration;

use crate::types::request_input::AiChatInput;
use ai::{AgentEvent, AiError, ChatMessage};
use poem::{
    Error, Request, handler,
    http::StatusCode,
    web::{
        Data, Json,
        sse::{Event, SSE},
    },
};
use store::DbPool;
use tokio::sync::mpsc;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};

use crate::middleware::auth::UserId;

/// The UserId the auth middleware inserted; 401 if the route isn't behind it.
fn authenticated_user(req: &Request) -> Result<String, Error> {
    req.extensions()
        .get::<UserId>()
        .map(|UserId(id)| id.clone())
        .ok_or_else(|| Error::from_status(StatusCode::UNAUTHORIZED))
}

fn event_type(event: &AgentEvent) -> &'static str {
    match event {
        AgentEvent::Thinking => "thinking",
        AgentEvent::ToolStarted { .. } => "tool_started",
        AgentEvent::ToolFinished { .. } => "tool_finished",
        AgentEvent::ConfirmationRequired { .. } => "confirmation_required",
        AgentEvent::Reply { .. } => "reply",
        AgentEvent::Error { .. } => "error",
    }
}

/// A safe, generic message for the client; the real cause is logged
/// server-side only, mirroring the status-code mapping the old non-streaming
/// handler used to do (BadInput/Db/Upstream), now expressed as the terminal
/// `error` event instead of an HTTP status — the response has already
/// committed to a 200 SSE stream by the time the agent can fail.
fn client_facing_message(err: &AiError) -> &'static str {
    match err {
        AiError::BadInput(_) => "that message couldn't be processed",
        AiError::Db(_) => "your data is temporarily unavailable",
        AiError::Upstream(_) => "the assistant is temporarily unavailable",
    }
}

/// Streams the agent's progress as SSE: a `thinking` event before every model
/// call, `tool_started`/`tool_finished` pairs around each tool call, then a
/// terminal `reply` (or `error`) event. The agent loop runs in a background
/// task so events reach the client as they happen instead of buffering until
/// the whole turn completes.
#[handler]
pub async fn chat(Data(pool): Data<&DbPool>, req: &Request, Json(input): Json<AiChatInput>) -> Result<SSE, Error> {
    let user_id = authenticated_user(req)?;
    let pool = pool.clone();

    let history: Vec<ChatMessage> = input
        .messages
        .into_iter()
        .map(|m| ChatMessage {
            role: m.role,
            content: m.content,
        })
        .collect();

    let (tx, rx) = mpsc::unbounded_channel::<AgentEvent>();
    // A separate handle: `tx` is moved into the agent loop and dropped when
    // it returns (success or failure), so the error path needs its own
    // sender to report a terminal `Error` event after that happens.
    let error_tx = tx.clone();

    let confirmed_actions = input.confirmed_actions;

    tokio::spawn(async move {
        if let Err(err) =
            ai::run_chat_streaming(&pool, &user_id, history, confirmed_actions, tx).await
        {
            eprintln!("ai chat error: {err}");
            let _ = error_tx.send(AgentEvent::Error {
                message: client_facing_message(&err).to_string(),
            });
        }
    });

    let stream = UnboundedReceiverStream::new(rx).map(|event| {
        let ty = event_type(&event);
        let data = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());
        Event::message(data).event_type(ty)
    });

    Ok(SSE::new(stream).keep_alive(Duration::from_secs(15)))
}

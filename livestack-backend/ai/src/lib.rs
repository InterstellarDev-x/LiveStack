//! In-app AI assistant: an agentic loop over the OpenAI chat completions API
//! whose tools query and, for a small allow-listed set, mutate this user's
//! monitoring data.
//!
//! Security invariant: every tool call runs against the `user_id` the caller
//! authenticated with — the model can never pick whose data it reads or
//! changes.
//!
//! # Mutating tools require confirmation, statelessly
//!
//! Every HTTP request is still handled start-to-finish with no server-side
//! session. So rather than pausing a live task mid-flight, a tool flagged by
//! [`tools::requires_confirmation`] is never auto-executed: the loop stops
//! and emits `AgentEvent::ConfirmationRequired` with the proposed
//! `(name, arguments)` pairs and a server-generated description. The turn
//! ends there. Nothing runs until the client echoes those exact actions back
//! as `confirmed_actions` on a later request, at which point they're
//! executed up front, before the model is called again. See
//! [`tools::describe_action`] for why the description is generated
//! server-side rather than trusted from the model's arguments.
//!
//! # Loop shape (two loops, not one)
//!
//! A single "LLM → tools → LLM" cycle handles a self-contained task, but an
//! embedded assistant also needs to accept *steering* (a user message that
//! arrives mid-task), accept *follow-ups* (a new request after the task
//! settles), and know when it is truly done (no tool calls pending AND no
//! queued input). Those concerns are split across two loops:
//!
//! - INNER loop: the tool-call cycle. Before every LLM call it drains the
//!   steering inbox and prepends any queued user messages. It exits only when
//!   the model produced a plain reply AND nothing new is queued.
//! - OUTER loop: after the inner loop settles, it waits on the inbox. A
//!   follow-up message re-enters the inner loop with the transcript intact;
//!   a closed inbox means the conversation is over.
//!
//! The one-shot HTTP endpoint is the degenerate case: it passes an inbox
//! whose sender is already dropped, so the outer loop exits after the first
//! settled reply. A future session endpoint (SSE/WebSocket) can keep the
//! sender open to get real mid-task steering without touching this loop.

use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestToolMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use store::DbPool;
use tokio::sync::mpsc;

mod tools;

pub use tools::ToolOutcome;

/// One turn of the conversation as the frontend sends it.
#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    /// "user" or "assistant"
    pub role: String,
    pub content: String,
}

/// A mutating tool call the model wants to make, awaiting the user's
/// explicit confirmation before it runs. The backend hands this to the
/// client on `AgentEvent::ConfirmationRequired`; the client must echo it
/// back verbatim (as `confirmed_actions` on the next request) for
/// [`run_agent`] to actually execute it — nothing is ever inferred from a
/// bare "yes".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingAction {
    pub name: String,
    pub arguments: Value,
    /// Human-readable summary, generated server-side from real data (e.g.
    /// the website's URL, not just its id) so the confirmation text can't be
    /// spoofed by whatever the model put in `arguments`.
    pub description: String,
}

/// Progress the loop emits while working. `ToolFinished.details` carries the
/// tool's structured summary (for UI/logs) — intentionally separate from the
/// `content` the LLM sees. Serializes with a `type` tag so a UI can switch on
/// it directly (e.g. over SSE).
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
    /// Emitted before every model call: the assistant is deciding what to
    /// say or which tools to call next.
    Thinking,
    ToolStarted { name: String, arguments: Value },
    ToolFinished { name: String, details: Value },
    /// The model wants to run one or more mutating tools. None of them have
    /// executed yet — the whole batch is on hold until the client resends
    /// these exact actions as `confirmed_actions`. Always the last event on
    /// a stream (like `Reply`): the turn ends here either way.
    ConfirmationRequired { actions: Vec<PendingAction> },
    Reply { content: String },
    /// The loop failed. Always the last event on a stream.
    Error { message: String },
}

#[derive(Debug)]
pub enum AiError {
    /// OPENAI_API_KEY missing or the upstream call failed.
    Upstream(String),
    /// A tool hit the database and failed.
    Db(String),
    /// The client sent something unusable (bad role, empty history).
    BadInput(String),
}

impl std::fmt::Display for AiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiError::Upstream(msg) => write!(f, "upstream AI error: {msg}"),
            AiError::Db(msg) => write!(f, "db error: {msg}"),
            AiError::BadInput(msg) => write!(f, "bad input: {msg}"),
        }
    }
}

impl std::error::Error for AiError {}

/// Tool-call budget per user turn (resets whenever new user input arrives).
/// The assistant never mutates anything, so this only bounds data gathering.
const MAX_TOOL_ROUNDS: usize = 6;
/// Public so channel integrations (which persist their own history between
/// messages) can cap it the same way this crate does internally.
pub const MAX_HISTORY_MESSAGES: usize = 40;

fn model_name() -> String {
    std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string())
}

fn system_prompt() -> String {
    format!(
        "You are the LiveStack assistant, embedded in an uptime-monitoring app. \
You help the signed-in user understand the health and performance of the websites they monitor.\n\
\n\
Data model you can query (always scoped to this user):\n\
- Websites: monitored URLs.\n\
- Ticks: periodic checks with a per-phase timing breakdown in milliseconds: \
dns_time_ms (DNS resolution), connection_time_ms (TCP connect), tls_time_ms (TLS handshake), \
waiting_time_ms (time to first byte — server think time), data_transfer_time_ms (body download), \
and response_time_ms (total). status is Up, Down, or Unknown.\n\
- Incidents: continuous outages; resolved_at null means still ongoing.\n\
- Status pages: public pages the user publishes for their monitors.\n\
\n\
Diagnostic guidance: use the phase breakdown to localise problems — high dns_time_ms points at the \
DNS provider, high connection/tls times at the network or CDN edge, high waiting_time_ms at the \
origin server (slow application/database), high data_transfer_time_ms at payload size or bandwidth.\n\
\n\
Rules:\n\
- Call tools to get real data before answering questions about the user's sites; never invent numbers.\n\
- Answer in plain language, concise and specific. Use concrete numbers from the data.\n\
- Format answers as Markdown. When listing several websites, incidents, or checks, use a numbered \
list with the site name in bold followed by indented `- Key: value` lines, e.g.:\n\
  1. **google.com**\n     - Status: Up\n     - Latest Response Time: 1660 ms\n     - Monitored Since: 2026-07-13\n\
For a single fact or diagnosis, a short paragraph is fine — don't force a list.\n\
- If asked something unrelated to website monitoring or this app, politely decline.\n\
- To add, change the URL of, or delete a monitor, call the tool directly — the app shows the user \
a confirmation prompt automatically before anything happens, so don't ask \"are you sure?\" yourself first.\n\
- All timestamps in the data are UTC. Current UTC time: {now}.",
        now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
    )
}

/// One-shot entry point for the HTTP handler: the whole conversation comes in
/// the request, the inbox is born closed, and the outer loop exits after the
/// first settled reply.
pub async fn run_chat(
    pool: &DbPool,
    user_id: &str,
    history: Vec<ChatMessage>,
) -> Result<String, AiError> {
    let (steer_tx, steer_rx) = mpsc::unbounded_channel();
    drop(steer_tx); // no live steering in one-shot mode
    run_agent(pool, user_id, history, Vec::new(), steer_rx, None, false).await
}

/// Same as [`run_chat`], but for channels (e.g. a linked Telegram chat)
/// where the pairing step is already the trust boundary: mutating tools
/// execute immediately instead of pausing for a `ConfirmationRequired`
/// round-trip, since there's no UI to show that prompt in.
pub async fn run_chat_direct(
    pool: &DbPool,
    user_id: &str,
    history: Vec<ChatMessage>,
) -> Result<String, AiError> {
    let (steer_tx, steer_rx) = mpsc::unbounded_channel();
    drop(steer_tx); // no live steering in one-shot mode
    run_agent(pool, user_id, history, Vec::new(), steer_rx, None, true).await
}

/// Same as [`run_chat`], but reports [`AgentEvent`]s as the loop works —
/// "thinking", tool start/finish, the final reply — so a caller (typically
/// an SSE handler) can forward them to the client without waiting for the
/// whole turn to finish. `confirmed_actions` are mutating tool calls the
/// user just approved (echoed back from a prior `ConfirmationRequired`
/// event); pass an empty vec on a normal turn.
pub async fn run_chat_streaming(
    pool: &DbPool,
    user_id: &str,
    history: Vec<ChatMessage>,
    confirmed_actions: Vec<PendingAction>,
    events: mpsc::UnboundedSender<AgentEvent>,
) -> Result<String, AiError> {
    let (steer_tx, steer_rx) = mpsc::unbounded_channel();
    drop(steer_tx); // no live steering in one-shot HTTP mode (yet)
    run_agent(
        pool,
        user_id,
        history,
        confirmed_actions,
        steer_rx,
        Some(events),
        false,
    )
    .await
}

/// Full agent loop. `inbox` carries steering and follow-up user messages;
/// `events` (optional) receives progress for a UI. `skip_confirmation`
/// bypasses the `ConfirmationRequired` pause for mutating tools — only set
/// for callers (like a linked chat channel) that have no UI to show that
/// prompt in and treat their own auth boundary as sufficient. Returns the
/// last settled reply once the inbox closes.
pub async fn run_agent(
    pool: &DbPool,
    user_id: &str,
    history: Vec<ChatMessage>,
    confirmed_actions: Vec<PendingAction>,
    mut inbox: mpsc::UnboundedReceiver<ChatMessage>,
    events: Option<mpsc::UnboundedSender<AgentEvent>>,
    skip_confirmation: bool,
) -> Result<String, AiError> {
    if history.is_empty() {
        return Err(AiError::BadInput("empty message history".into()));
    }
    if history.len() > MAX_HISTORY_MESSAGES {
        return Err(AiError::BadInput("conversation too long".into()));
    }

    let mut messages: Vec<ChatCompletionRequestMessage> = Vec::with_capacity(history.len() + 1);
    messages.push(
        ChatCompletionRequestSystemMessageArgs::default()
            .content(system_prompt())
            .build()
            .map_err(|e| AiError::Upstream(e.to_string()))?
            .into(),
    );
    for msg in history {
        messages.push(request_message(&msg)?);
    }

    // Reads OPENAI_API_KEY (and optional OPENAI_API_BASE) from the environment.
    let client = Client::<OpenAIConfig>::new();
    let model = model_name();
    let tool_defs = tools::definitions().map_err(|e| AiError::Upstream(e.to_string()))?;

    let emit = |event: AgentEvent| {
        if let Some(tx) = &events {
            let _ = tx.send(event);
        }
    };

    // Actions the user just approved (echoed back from a prior
    // ConfirmationRequired event) run once, up front, before any model call.
    // There's no prior `tool_calls` message in this request to attach real
    // `tool` results to — the approval happened in a previous, now-forgotten
    // HTTP request — so the outcome is folded in as a plain-language system
    // note instead of a fabricated tool exchange.
    if !confirmed_actions.is_empty() {
        let mut outcomes = Vec::with_capacity(confirmed_actions.len());
        for action in &confirmed_actions {
            emit(AgentEvent::ToolStarted {
                name: action.name.clone(),
                arguments: action.arguments.clone(),
            });

            let arguments = action.arguments.to_string();
            let result = tools::execute(pool, user_id, &action.name, &arguments);
            let summary = match &result {
                Ok(ToolOutcome { details, .. }) => {
                    emit(AgentEvent::ToolFinished {
                        name: action.name.clone(),
                        details: details.clone(),
                    });
                    format!("- Done: {} ({details})", action.description)
                }
                Err(err) => {
                    emit(AgentEvent::ToolFinished {
                        name: action.name.clone(),
                        details: serde_json::json!({ "error": err }),
                    });
                    format!("- Failed: {} — {err}", action.description)
                }
            };
            outcomes.push(summary);
        }

        messages.push(
            ChatCompletionRequestSystemMessageArgs::default()
                .content(format!(
                    "The user just confirmed and the assistant completed the following \
                     previously-proposed action(s):\n{}\n\nContinue the conversation \
                     naturally, acknowledging what happened.",
                    outcomes.join("\n")
                ))
                .build()
                .map_err(|e| AiError::Upstream(e.to_string()))?
                .into(),
        );
    }

    // OUTER loop: one iteration per settled reply; re-entered on follow-ups.
    loop {
        let mut rounds = 0usize;

        // INNER loop: the tool-call cycle, with steering injected before
        // every LLM call.
        let reply = loop {
            while let Ok(steer) = inbox.try_recv() {
                messages.push(request_message(&steer)?);
                rounds = 0; // new user input, fresh budget
            }

            if rounds >= MAX_TOOL_ROUNDS {
                return Err(AiError::Upstream(
                    "assistant did not finish within the tool-call budget".into(),
                ));
            }
            rounds += 1;

            emit(AgentEvent::Thinking);

            let request = CreateChatCompletionRequestArgs::default()
                .model(&model)
                .messages(messages.clone())
                .tools(tool_defs.clone())
                .build()
                .map_err(|e| AiError::Upstream(e.to_string()))?;

            let response = client
                .chat()
                .create(request)
                .await
                .map_err(|e| AiError::Upstream(e.to_string()))?;

            let choice = response
                .choices
                .into_iter()
                .next()
                .ok_or_else(|| AiError::Upstream("no choices in response".into()))?;

            let tool_calls = choice.message.tool_calls.unwrap_or_default();

            if tool_calls.is_empty() {
                let content = choice.message.content.unwrap_or_default();

                // Done only if nothing arrived while the model was answering:
                // a steering message that raced the final reply re-enters the
                // cycle instead of being silently deferred.
                match inbox.try_recv() {
                    Ok(steer) => {
                        messages.push(assistant_text(&content)?);
                        messages.push(request_message(&steer)?);
                        rounds = 0;
                        continue;
                    }
                    Err(_) => break content,
                }
            }

            // If any call in this response needs confirmation, hold the
            // *whole* response — including any read-only calls bundled
            // alongside it. Nothing here has executed yet, so there's no
            // partial-execution state to reconcile: the model just re-asks
            // for whatever it still needs on the next turn.
            if !skip_confirmation
                && tool_calls
                    .iter()
                    .any(|call| tools::requires_confirmation(&call.function.name))
            {
                let mut actions = Vec::with_capacity(tool_calls.len());
                for call in &tool_calls {
                    let description = tools::describe_action(
                        pool,
                        user_id,
                        &call.function.name,
                        &call.function.arguments,
                    )
                    .map_err(|e| AiError::Upstream(format!("couldn't prepare confirmation: {e}")))?;
                    actions.push(PendingAction {
                        name: call.function.name.clone(),
                        arguments: serde_json::from_str(&call.function.arguments)
                            .unwrap_or(Value::Null),
                        description,
                    });
                }
                emit(AgentEvent::ConfirmationRequired { actions });
                return Ok(String::new());
            }

            // Echo the assistant turn (with its tool calls) back into the
            // transcript, then answer every call before the next round.
            let mut assistant_msg = ChatCompletionRequestAssistantMessageArgs::default();
            assistant_msg.tool_calls(tool_calls.clone());
            if let Some(content) = choice.message.content {
                assistant_msg.content(content);
            }
            messages.push(
                assistant_msg
                    .build()
                    .map_err(|e| AiError::Upstream(e.to_string()))?
                    .into(),
            );

            // Scheduling rules for a multi-tool turn:
            //   1. safety — only parallel-safe (read-only) tools run
            //      concurrently; anything else runs one at a time, after the
            //      concurrent batch has fully drained.
            //   2. order — whatever order tools *finish* in, results are
            //      reported back to the model in the order it requested them.
            enum Execution {
                Spawned(tokio::task::JoinHandle<Result<ToolOutcome, String>>),
                Sequential,
            }

            let mut executions = Vec::with_capacity(tool_calls.len());
            for call in &tool_calls {
                emit(AgentEvent::ToolStarted {
                    name: call.function.name.clone(),
                    arguments: serde_json::from_str(&call.function.arguments)
                        .unwrap_or(Value::Null),
                });

                if tools::is_parallel_safe(&call.function.name) {
                    // Diesel is sync, so each concurrent call gets its own
                    // blocking thread and pool connection.
                    let pool = pool.clone();
                    let user = user_id.to_string();
                    let name = call.function.name.clone();
                    let args = call.function.arguments.clone();
                    executions.push(Execution::Spawned(tokio::task::spawn_blocking(
                        move || tools::execute(&pool, &user, &name, &args),
                    )));
                } else {
                    executions.push(Execution::Sequential);
                }
            }

            // Drain the concurrent batch (handles are all running already, so
            // awaiting in source order loses no parallelism)...
            let mut outcomes: Vec<Option<Result<ToolOutcome, String>>> =
                Vec::with_capacity(tool_calls.len());
            for execution in executions {
                match execution {
                    Execution::Spawned(handle) => outcomes.push(Some(match handle.await {
                        Ok(result) => result,
                        Err(join_err) => Err(format!("tool task failed: {join_err}")),
                    })),
                    Execution::Sequential => outcomes.push(None),
                }
            }
            // ...then run sequential tools one at a time, in source order.
            for (slot, call) in outcomes.iter_mut().zip(&tool_calls) {
                if slot.is_none() {
                    *slot = Some(tools::execute(
                        pool,
                        user_id,
                        &call.function.name,
                        &call.function.arguments,
                    ));
                }
            }

            // Report every result in SOURCE order.
            for (call, outcome) in tool_calls.iter().zip(outcomes) {
                let outcome = outcome.expect("every slot filled above");

                // `content` goes to the model; `details` to the UI/log stream.
                let payload = match outcome {
                    Ok(ToolOutcome { content, details }) => {
                        emit(AgentEvent::ToolFinished {
                            name: call.function.name.clone(),
                            details,
                        });
                        content.to_string()
                    }
                    // Tool errors go back to the model as content so it can
                    // adjust (e.g. bad website id) instead of aborting the turn.
                    Err(err) => {
                        emit(AgentEvent::ToolFinished {
                            name: call.function.name.clone(),
                            details: serde_json::json!({ "error": err }),
                        });
                        serde_json::json!({ "error": err }).to_string()
                    }
                };

                messages.push(
                    ChatCompletionRequestToolMessageArgs::default()
                        .tool_call_id(call.id.clone())
                        .content(payload)
                        .build()
                        .map_err(|e| AiError::Upstream(e.to_string()))?
                        .into(),
                );
            }
        };

        emit(AgentEvent::Reply {
            content: reply.clone(),
        });

        // OUTER: wait for a follow-up. Closed inbox → conversation over.
        match inbox.recv().await {
            None => return Ok(reply),
            Some(follow_up) => {
                messages.push(assistant_text(&reply)?);
                messages.push(request_message(&follow_up)?);
            }
        }
    }
}

fn request_message(msg: &ChatMessage) -> Result<ChatCompletionRequestMessage, AiError> {
    match msg.role.as_str() {
        "user" => Ok(ChatCompletionRequestUserMessageArgs::default()
            .content(msg.content.clone())
            .build()
            .map_err(|e| AiError::Upstream(e.to_string()))?
            .into()),
        "assistant" => Ok(assistant_text(&msg.content)?),
        other => Err(AiError::BadInput(format!("unsupported role: {other}"))),
    }
}

fn assistant_text(content: &str) -> Result<ChatCompletionRequestMessage, AiError> {
    Ok(ChatCompletionRequestAssistantMessageArgs::default()
        .content(content.to_string())
        .build()
        .map_err(|e| AiError::Upstream(e.to_string()))?
        .into())
}

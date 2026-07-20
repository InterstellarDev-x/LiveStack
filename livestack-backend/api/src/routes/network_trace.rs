use std::sync::Arc;

use crate::types::request_input::NetworkTraceInput;
use maxminddb::Reader;
use nettrace::TraceError;
use poem::{
    Error, Request, handler,
    http::StatusCode,
    web::{
        Data, Json,
        sse::{Event, SSE},
    },
};
use tokio_stream::iter as stream_iter;

use crate::middleware::auth::UserId;

/// The UserId the auth middleware inserted; 401 if the route isn't behind it.
fn authenticated_user(req: &Request) -> Result<String, Error> {
    req.extensions()
        .get::<UserId>()
        .map(|UserId(id)| id.clone())
        .ok_or_else(|| Error::from_status(StatusCode::UNAUTHORIZED))
}

fn map_trace_error(err: TraceError) -> Error {
    match err {
        TraceError::InvalidTarget(_)
        | TraceError::PrivateTarget(_)
        | TraceError::ResolutionFailed(_) => Error::from_string(err.to_string(), StatusCode::BAD_REQUEST),
        TraceError::TraceFailed(_) => {
            Error::from_string(err.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Runs a single on-demand traceroute against the given host/URL and streams
/// each resolved, geolocated hop back as an SSE `hop` event. Nothing is
/// persisted; this is a diagnostic tool, not a monitor check.
#[handler]
pub async fn network_trace(
    Data(geoip): Data<&Arc<Reader<Vec<u8>>>>,
    req: &Request,
    Json(input): Json<NetworkTraceInput>,
) -> Result<SSE, Error> {
    authenticated_user(req)?;

    let hops = nettrace::run_trace(geoip, &input.target)
        .await
        .map_err(map_trace_error)?;

    let events = hops.into_iter().map(|hop| {
        let data = serde_json::to_string(&hop).unwrap_or_else(|_| "{}".to_string());
        Event::message(data).event_type("hop")
    });

    Ok(SSE::new(stream_iter(events)))
}

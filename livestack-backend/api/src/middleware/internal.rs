use poem::{Endpoint, Error, IntoResponse, Request, Response, Result, http::StatusCode};

use crate::utils::internal_secret;

/// Gate for routes called by trusted backend services (e.g. the Telegram
/// gateway), not browsers — checked against a static shared secret instead
/// of the user-facing JWT flow in `auth::log`.
pub async fn internal_auth<E: Endpoint>(next: E, req: Request) -> Result<Response, Error> {
    let provided = req
        .headers()
        .get("x-internal-secret")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| Error::from_status(StatusCode::UNAUTHORIZED))?;

    if provided != internal_secret() {
        return Err(Error::from_status(StatusCode::UNAUTHORIZED));
    }

    Ok(next.call(req).await?.into_response())
}

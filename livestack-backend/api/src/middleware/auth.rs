use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use poem::{Endpoint, Error, IntoResponse, Request, Response, Result, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::utils::jwt_secret;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    user_id: String,
    exp: u64,
}

#[derive(Clone)]
pub struct UserId(pub String);

pub async fn log<E: Endpoint>(next: E, mut req: Request) -> Result<Response, Error> {
    println!("request: {}", req.uri().path());
    let token = req
        .headers()
        .get("token")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| Error::from_status(StatusCode::UNAUTHORIZED))?; // If header exists → continue

    // default validation verifies the signature and rejects expired tokens
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(jwt_secret()), &validation)
        .map_err(|_| Error::from_status(StatusCode::UNAUTHORIZED))?; // any bad token → 401, never panic

    req.extensions_mut()
        .insert(UserId(token_data.claims.user_id)); // inserting userId to req

    let res = next.call(req).await; // calling next route
    match res {
        Ok(resp) => {
            let resp = resp.into_response();
            println!("response: {}", resp.status());
            Ok(resp)
        }
        Err(err) => {
            println!("error: {err}");
            Err(err)
        }
    }
}

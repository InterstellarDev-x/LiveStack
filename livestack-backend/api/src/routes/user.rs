use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    middleware::auth::UserId,
    types::request_input::{SigninInput, SignupInput, UpdateEmailAlertsInput, UpdateEmailInput},
    types::request_output::{
        CurrentUserOutput, SignInpOutput, SignUpOutput, UpdateEmailAlertsOutput,
        UpdateEmailOutput,
    },
    utils::{hash_password, jwt_secret, verify_password},
};
use jsonwebtoken::{EncodingKey, Header, encode};
use poem::{Error, Request, handler, web::Data};
use poem::{http::StatusCode, web::Json};
use serde::{Deserialize, Serialize};
use store::{DatabaseErrorKind, DbError, DbPool, Store};

/// Long enough to be worth having, short enough not to lock anyone out.
const MIN_PASSWORD_LEN: usize = 8;
const MAX_USERNAME_LEN: usize = 64;
/// Argon2 hashes the whole input, so a very long password is a cheap way to
/// make signup expensive for the server.
const MAX_PASSWORD_LEN: usize = 256;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    user_id: String,
    exp: u64,
}

fn store_from_pool(pool: &DbPool) -> Result<Store, Error> {
    Store::from_pool(pool).map_err(|_| Error::from_status(StatusCode::SERVICE_UNAVAILABLE))
}

fn rejected(message: &str) -> Json<SignUpOutput> {
    Json(SignUpOutput {
        success: false,
        message: message.to_string(),
    })
}

/// Rejects credentials that can't work before any hashing or database work
/// happens. Returns the message to show the user, or `None` if they're fine.
fn validate_credentials(username: &str, password: &str) -> Option<&'static str> {
    if username.is_empty() {
        return Some("Username must not be empty");
    }
    if username.len() > MAX_USERNAME_LEN {
        return Some("Username is too long");
    }
    if password.len() < MIN_PASSWORD_LEN {
        return Some("Password must be at least 8 characters");
    }
    if password.len() > MAX_PASSWORD_LEN {
        return Some("Password is too long");
    }
    None
}

#[handler]
pub fn signup(
    Json(data): Json<SignupInput>,
    Data(pool): Data<&DbPool>,
) -> Result<Json<SignUpOutput>, Error> {
    let username = data.username.trim().to_string();

    if let Some(message) = validate_credentials(&username, &data.password) {
        return Ok(rejected(message));
    }

    let mut store = store_from_pool(pool)?;

    match store.is_user_exist(&username) {
        Ok(true) => Ok(rejected("User already registered")),
        Ok(false) => {
            // never store the raw password, only an argon2 hash
            let password_hash = hash_password(&data.password)
                .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

            match store.create_user(username, password_hash) {
                Ok(u) => Ok(Json(SignUpOutput {
                    success: true,
                    message: format!("Successfully signed up with user_id {}", u.id),
                })),
                // The check above races another signup for the same name; the
                // unique index is what actually decides, so a violation here
                // means "taken", not a server fault.
                Err(DbError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
                    Ok(rejected("User already registered"))
                }
                Err(_) => Ok(rejected("Internal Server Error")),
            }
        }
        Err(_) => Ok(rejected("Internal Server Error")),
    }
}

#[handler]
pub fn signin(
    Json(data): Json<SigninInput>,
    Data(pool): Data<&DbPool>,
) -> Result<Json<SignInpOutput>, Error> {
    let mut store = store_from_pool(pool)?;

    // Trimmed to match how signup stores it.
    let username = data.username.trim().to_string();

    let user = store
        .get_user_by_username(&username)
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?
        // same 401 for unknown user and wrong password, so usernames can't be enumerated
        .ok_or_else(|| Error::from_status(StatusCode::UNAUTHORIZED))?;

    if !verify_password(&data.password, &user.password) {
        return Err(Error::from_status(StatusCode::UNAUTHORIZED));
    }

    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 36000; // testing purpose

    let claim = Claims {
        user_id: user.id,
        exp,
    };
    let token = encode(
        &Header::default(),
        &claim,
        &EncodingKey::from_secret(jwt_secret()),
    )
    .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Json(SignInpOutput {
        success: true,
        token,
    }))
}

#[handler]
pub fn update_email(
    Json(data): Json<UpdateEmailInput>,
    Data(pool): Data<&DbPool>,
    req: &Request,
) -> Result<Json<UpdateEmailOutput>, Error> {
    let UserId(user_id) = req
        .extensions()
        .get::<UserId>()
        .cloned()
        .ok_or_else(|| Error::from_status(StatusCode::UNAUTHORIZED))?;

    // full RFC 5322 validation isn't worth it here; just catch obvious typos
    if !data.email.contains('@') {
        return Err(Error::from_status(StatusCode::BAD_REQUEST));
    }

    let mut store = store_from_pool(pool)?;

    let updated = store
        .update_user_email(user_id, data.email)
        .map_err(|err| match err {
            DbError::NotFound => Error::from_status(StatusCode::NOT_FOUND),
            _ => Error::from_status(StatusCode::INTERNAL_SERVER_ERROR),
        })?;

    Ok(Json(UpdateEmailOutput {
        success: true,
        email: updated.email.unwrap_or_default(),
    }))
}

#[handler]
pub fn get_current_user(
    Data(pool): Data<&DbPool>,
    req: &Request,
) -> Result<Json<CurrentUserOutput>, Error> {
    let UserId(user_id) = req
        .extensions()
        .get::<UserId>()
        .cloned()
        .ok_or_else(|| Error::from_status(StatusCode::UNAUTHORIZED))?;

    let mut store = store_from_pool(pool)?;

    let user = store.get_user_by_id(&user_id).map_err(|err| match err {
        DbError::NotFound => Error::from_status(StatusCode::NOT_FOUND),
        _ => Error::from_status(StatusCode::INTERNAL_SERVER_ERROR),
    })?;

    Ok(Json(CurrentUserOutput {
        id: user.id,
        username: user.username,
        email: user.email,
        email_alerts_enabled: user.email_alerts_enabled,
    }))
}

#[handler]
pub fn update_email_alerts(
    Json(data): Json<UpdateEmailAlertsInput>,
    Data(pool): Data<&DbPool>,
    req: &Request,
) -> Result<Json<UpdateEmailAlertsOutput>, Error> {
    let UserId(user_id) = req
        .extensions()
        .get::<UserId>()
        .cloned()
        .ok_or_else(|| Error::from_status(StatusCode::UNAUTHORIZED))?;

    let mut store = store_from_pool(pool)?;

    let updated = store
        .set_email_alerts_enabled(user_id, data.enabled)
        .map_err(|err| match err {
            DbError::NotFound => Error::from_status(StatusCode::NOT_FOUND),
            _ => Error::from_status(StatusCode::INTERNAL_SERVER_ERROR),
        })?;

    Ok(Json(UpdateEmailAlertsOutput {
        success: true,
        email_alerts_enabled: updated.email_alerts_enabled,
    }))
}

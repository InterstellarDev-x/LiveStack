use std::{
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    types::request_input::SigninInput,
    types::request_output::{SignInpOutput, SignUpOutput},
};
use jsonwebtoken::{EncodingKey, Header, encode};
use poem::{Error, handler, web::Data};
use poem::{http::StatusCode, web::Json};
use serde::{Deserialize, Serialize};
use store::Store;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    user_id: String,
    exp: u64,
}

#[handler]
pub fn signup(
    Json(data): Json<SigninInput>,
    Data(store): Data<&Arc<Mutex<Store>>>,
) -> Result<Json<SignUpOutput>, Error> {
    let mut locked_store = store.lock().unwrap();

    match locked_store.is_user_exist(&data.username) {
        Ok(true) => {
            return Ok(Json(SignUpOutput {
                success: false,
                message: "User already registerd".into(),
            }));
        }
        Ok(false) => match locked_store.create_user(data.username, data.password) {
            Ok(u) => {
                return Ok(Json(SignUpOutput {
                    success: true,
                    message: format!("Successfully signed up witth user_id {}", u.id),
                }));
            }

            Err(_) => {
                return Ok(Json(SignUpOutput {
                    success: false,
                    message: "Internal Server Error".into(),
                }));
            }
        },

        Err(_) => {
            return Ok(Json(SignUpOutput {
                success: false,
                message: "Internal Server Error".into(),
            }));
        }
    }
}

#[handler]
pub fn signin(
    Json(data): Json<SigninInput>,
    Data(store): Data<&Arc<Mutex<Store>>>,
) -> Result<Json<SignInpOutput>, Error> {
    let mut store = store.lock().unwrap();

    match store.is_user_exist(&data.username) {
        Ok(true) => {
            match store.is_exist_and_password_match(&data.username) {
                Ok(u) => {
                    // username check is pending

                    if u.password != data.password {
                        return Err(Error::from_status(StatusCode::UNAUTHORIZED));
                    }

                    let exp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                        + 36000; // testing purpose

                    let claim = Claims { user_id: u.id, exp };
                    let key = b"fdfad"; // should be stored in env file
                    let token =
                        match encode(&Header::default(), &claim, &EncodingKey::from_secret(key)) {
                            Ok(t) => t,
                            Err(_) => panic!("Erron while token creation"),
                        };

                    return Ok(Json(SignInpOutput {
                        success: true,
                        token: token,
                    }));
                }
                Err(_) => {
                    return Ok(Json(SignInpOutput {
                        success: false,
                        token: "not generated".into(),
                    }));
                }
            }
        }

        Ok(false) => {
            return Err(Error::from_status(StatusCode::BAD_REQUEST));
        }

        Err(_) => return Err(Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)),
    }
}

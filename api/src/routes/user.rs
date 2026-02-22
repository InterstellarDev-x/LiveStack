use std::sync::{Arc, Mutex};

use crate::{
    types::request_input::SigninInput,
    types::request_output::{SignInpOutput, SignUpOutput},
};
use poem::{handler, web::Data};
use poem::web::Json;
use store::Store;

#[handler]
pub fn signup(Json(data): Json<SigninInput> , Data(store) : Data<&Arc<Mutex<Store>>>) -> Json<SignUpOutput> {
    
    let mut locked_store = store.lock().unwrap();

    match locked_store.is_user_exist(&data.username) {
        Ok(true) => {
            return Json(SignUpOutput {
                success: false,
                message: "User already registerd".into(),
            });
        }
        Ok(false) => match locked_store.create_user(data.username, data.password) {
            Ok(u) => {
                return Json(SignUpOutput {
                    success: true,
                    message: format!("Successfully signed up witth user_id {}", u.id),
                });
            }

            Err(_) => {
                return Json(SignUpOutput {
                    success: false,
                    message: "Internal Server Error".into(),
                });
            }
        },

        Err(_) => {
            return Json(SignUpOutput {
                success: false,
                message: "Internal Server Error".into(),
            });
        }
    }
}

#[handler]
pub fn signin(Json(data): Json<SigninInput>, Data(store) : Data<&Arc<Mutex<Store>>>) -> Json<SignInpOutput> {
    let mut store = store.lock().unwrap();

    match store.is_exist_and_password_match(&data.username, &data.password) {
        Ok(true) => {
            return Json(SignInpOutput {
                success: true,
                token: "token".into(),
            });
        }

        Ok(false) => {
            return Json(SignInpOutput {
                success: false,
                token: "no token".into(),
            });
        }

        Err(_) => {
            return Json(SignInpOutput {
                success: false,
                token: "not generated".into(),
            });
        }
    }
}




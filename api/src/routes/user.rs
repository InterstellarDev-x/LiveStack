use crate::{
    types::request_input::SigninInput,
    types::request_output::{SignInpOutput, SignUpOutput},
};
use poem::handler;
use poem::web::Json;
use store::Store;

#[handler]
pub fn signup(Json(data): Json<SigninInput>) -> Json<SignUpOutput> {
    let mut store = Store::default().unwrap();

    match store.is_user_exist(&data.username) {
        Ok(true) => {
            return Json(SignUpOutput {
                success: false,
                message: "User already registerd".into(),
            });
        }
        Ok(false) => match store.create_user(data.username, data.password) {
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
pub fn signin(Json(data): Json<SigninInput>) -> Json<SignInpOutput> {
    let mut store = Store::default().unwrap();

    match store.is_exist_and_password_match(&data.username, &data.password) {
        Ok(true) => {
            return Json(SignInpOutput {
                success: true,
                token: "tokne".into(),
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

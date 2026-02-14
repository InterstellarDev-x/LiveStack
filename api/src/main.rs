use poem::{
    Route, Server, get, handler,
    listener::TcpListener,
    post,
    web::{Json, Path},
};
use store::Store;

use crate::{
    request_input::{CreateWebsiteInput, SigninInput},
    request_output::{CreateWebsiteOutput, SignInpOutput, SignUpOutput},
};
pub mod request_input;
pub mod request_output;
pub mod routes;

#[handler] //macros , make this below little more complex 
fn get_website(Path(website_id): Path<String>) -> String {
    format!("hello: {website_id} ") // for using dynamic variables inside the string
}

#[handler]
fn create_website(Json(data): Json<CreateWebsiteInput>) -> Json<CreateWebsiteOutput> {
    let store = Store::default().unwrap();
    let response = CreateWebsiteOutput { id: data.url };

    return Json(response);
}

#[handler]
fn signup(Json(data): Json<SigninInput>) -> Json<SignUpOutput> {
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
fn signin(Json(data): Json<SigninInput>) -> Json<SignInpOutput> {
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

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // specify the business logic
    let app = Route::new()
        .at("/website/:website_id", get(get_website))
        .at("/website", post(create_website))
        .at("/signup" , post(signup))
        .at("/signin" , post(signin));

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .name("hello-world") // give it a name to server
        .run(app) // this
        .await
}

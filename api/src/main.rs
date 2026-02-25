use std::sync::{Arc, Mutex};

use poem::{EndpointExt, Route, Server, get, listener::TcpListener, post};
use store::Store;

use crate::{
    routes::user::{signin, signup},
    routes::website::{
        create_website, delete_website, get_website, get_websites_by_user, update_website,
    },

    middleware::auth::log
};
pub  mod middleware;
pub mod routes;
pub mod types;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), std::io::Error> {
    let arched_store = Arc::new(Mutex::new(Store::default().unwrap()));

    // specify the business logic
    let app = Route::new()
        .at(
            "/website/:website_id",
            get(get_website).put(update_website).delete(delete_website).around(log)
        )
        .at("/website", post(create_website).around(log))
        .at("/websites/:user_id", get(get_websites_by_user))
        .at("/signup", post(signup))
        .at("/signin", post(signin))
        .data(arched_store);

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .name("LiveStack Server") // give it a name to server
        .run(app) // this
        .await
}

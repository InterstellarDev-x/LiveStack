use poem::{EndpointExt, Route, Server, get, listener::TcpListener, post};
use store::Store;

use crate::{
    middleware::auth::log,
    routes::user::{signin, signup},
    routes::website::{
        create_website, delete_website, get_website, get_websites_by_user, update_website,
    },
};
pub mod middleware;
pub mod routes;
pub mod types;
pub mod utils;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // fail fast so we never run with a missing signing key
    if std::env::var("JWT_SECRET").is_err() {
        return Err("JWT_SECRET must be set (e.g. in a .env file)".into());
    }

    let store_pool =
        Store::pool().map_err(|err| -> Box<dyn std::error::Error> { Box::new(err) })?;

    // specify the business logic
    let app = Route::new()
        .at(
            "/website/:website_id",
            get(get_website)
                .put(update_website)
                .delete(delete_website)
                .around(log), // middleware
        )
        .at("/website", post(create_website).around(log))
        .at("/websites", get(get_websites_by_user).around(log)) // user comes from the token
        .at("/signup", post(signup))
        .at("/signin", post(signin))
        .data(store_pool);

    Ok(Server::new(TcpListener::bind("0.0.0.0:3000"))
        .name("LiveStack Server") // give it a name to server
        .run(app) // this
        .await?)
}

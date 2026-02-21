use poem::{Route, Server, get, listener::TcpListener, post};

use crate::{
    routes::user::{signin, signup},
    routes::website::{create_website, get_website},
};

pub mod routes;
pub mod types;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // specify the business logic
    let app = Route::new()
        .at("/website/:website_id", get(get_website))
        .at("/website", post(create_website))
        .at("/signup", post(signup))
        .at("/signin", post(signin));

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .name("hello-world") // give it a name to server
        .run(app) // this
        .await
}

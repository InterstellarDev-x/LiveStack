use std::sync::{Arc, Mutex};

use poem::{ EndpointExt, Route, Server, get, listener::TcpListener, post};
use store::Store;

use crate::{
    routes::user::{signin, signup},
    routes::website::{get_website , create_website},
};

pub mod routes;
pub mod types;

#[tokio::main(flavor = "multi_thread")] 
async fn main() -> Result<(), std::io::Error> {



   let arched_store = Arc::new(Mutex::new(Store::default().unwrap()));

    // specify the business logic
    let app = Route::new()
        .at("/website/:website_id", get(get_website))
        .at("/website", post(create_website))
        .at("/signup", post(signup))
        .at("/signin", post(signin))
        .data(arched_store);


 
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .name("LiveStack Server") // give it a name to server
        .run(app) // this
        .await
}





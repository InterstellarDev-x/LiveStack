use poem::{
    Route, Server, get, handler, listener::TcpListener, post, web::{Json, Path}
};
use store::Store;

use crate::{request_input::CreateWebsiteInput, request_output::CreateWebsiteOutput};


pub mod request_input;
pub mod request_output;


#[handler] //macros , make this below little more complex 
fn get_website(Path(website_id): Path<String> ) -> String {
    format!("hello: {website_id} ") // for using dynamic variables inside the string
}


#[handler]
fn create_website(Json(data) : Json<CreateWebsiteInput>) -> Json<CreateWebsiteOutput>{
    let store = Store::default().unwrap();
    let url = store.create_website();
    let response = CreateWebsiteOutput{
        id :  url
    };

    return Json(response);
}







#[tokio::main]
async fn main() -> Result<(), std::io::Error> {

    // specify the business logic 
    let app = Route::new()
    .at("/website/:website_id", get(get_website))
    .at("/website", post(create_website));


    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .name("hello-world") // give it a name to server
        .run(app) // this 
        .await
}
use poem::{
    handler,
    web::{Json, Path},
};
use store::Store;

use crate::{types::request_input::CreateWebsiteInput, types::request_output::CreateWebsiteOutput};

#[handler] //macros , make this below little more complex 
pub fn get_website(Path(website_id): Path<String>) -> String {
    format!("hello: {website_id} ") // for using dynamic variables inside the string
}

#[handler]
pub fn create_website(Json(data): Json<CreateWebsiteInput>) -> Json<CreateWebsiteOutput> {
    let store = Store::default().unwrap();
    let response = CreateWebsiteOutput { id: data.url };

    return Json(response);
}

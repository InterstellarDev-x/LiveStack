use std::sync::{Arc, Mutex};

use poem::{
    Error, handler, web::{Data, Json, Path}
};
use store::Store;
use crate::{types::request_input::CreateWebsiteInput, types::request_output::CreateWebsiteOutput};
#[handler] //macros , make this below little more complex 
pub fn get_website(Path(website_id): Path<String>) -> String {
    format!("hello: {website_id} ") // for using dynamic variables inside the string
}

#[handler]
pub fn create_website(Json(data): Json<CreateWebsiteInput>, Data(store) : Data<&Arc<Mutex<Store>>>) -> Result<Json<CreateWebsiteOutput> , Error> {
    let mut store = store.lock().unwrap();

    let response = store.create_website("1".into(), data.url).unwrap();

    let new_website = CreateWebsiteOutput {
        success : true,
        id : response.id,
        
    };

    return Ok(Json(new_website));
}

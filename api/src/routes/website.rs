use std::sync::{Arc, Mutex};

use crate::types::{
    request_input::{CreateWebsiteInput, UpdateWebsiteInput},
    request_output::{
        CreateWebsiteOutput, DeleteWebsiteOutput, WebsiteOutput, WebsitesByUserOutput,
    },
};
use poem::{
    handler,
    http::StatusCode,
    web::{Data, Json, Path},
    Error,
};
use store::{models::website::Website, Store};

fn map_website_to_output(website: Website) -> WebsiteOutput {
    WebsiteOutput {
        id: website.id,
        url: website.url,
        user_id: website.user_id,
    }
}

#[handler]
pub fn get_website(
    Path(website_id): Path<String>,
    Data(store): Data<&Arc<Mutex<Store>>>,
) -> Result<Json<WebsiteOutput>, Error> {
    let mut store = store.lock().unwrap();

    let website = store
        .get_website_by_id(website_id)
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Json(map_website_to_output(website)))
}

#[handler]
pub fn create_website(
    Json(data): Json<CreateWebsiteInput>,
    Data(store): Data<&Arc<Mutex<Store>>>,
) -> Result<Json<CreateWebsiteOutput>, Error> {
    let mut store = store.lock().unwrap();

    let response = store
        .create_website("1".into(), data.url)
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

    let new_website = CreateWebsiteOutput {
        success: true,
        id: response.id,
    };

    Ok(Json(new_website))
}

#[handler]
pub fn delete_website(
    Path(website_id): Path<String>,
    Data(store): Data<&Arc<Mutex<Store>>>,
) -> Result<Json<DeleteWebsiteOutput>, Error> {
    let mut store = store.lock().unwrap();

    let deleted = store
        .delete_by_id(website_id)
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Json(DeleteWebsiteOutput { success: deleted }))
}

#[handler]
pub fn update_website(
    Path(website_id): Path<String>,
    Json(data): Json<UpdateWebsiteInput>,
    Data(store): Data<&Arc<Mutex<Store>>>,
) -> Result<Json<WebsiteOutput>, Error> {
    let mut store = store.lock().unwrap();

    let updated = store
        .update_by_id(website_id, data.url)
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Json(map_website_to_output(updated)))
}

#[handler]
pub fn get_websites_by_user(
    Path(user_id): Path<String>,
    Data(store): Data<&Arc<Mutex<Store>>>,
) -> Result<Json<WebsitesByUserOutput>, Error> {
    let mut store = store.lock().unwrap();

    let websites = store
        .get_websites_by_user_id(user_id)
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

    let output = WebsitesByUserOutput {
        websites: websites.into_iter().map(map_website_to_output).collect(),
    };

    Ok(Json(output))
}

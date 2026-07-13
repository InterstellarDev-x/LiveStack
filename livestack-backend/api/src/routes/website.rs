use crate::{
    middleware::auth::UserId,
    types::{
        request_input::{CreateWebsiteInput, UpdateWebsiteInput},
        request_output::{
            CreateWebsiteOutput, DeleteWebsiteOutput, WebsiteOutput, WebsiteOutputWithTick,
            WebsitesByUserOutput,
        },
    },
};
use poem::{
    Error, Request, handler,
    http::StatusCode,
    web::{Data, Json, Path},
};
use store::{
    DbError, DbPool, Store,
    models::website::{Website, WebsiteWithLatestTick},
};

fn store_from_pool(pool: &DbPool) -> Result<Store, Error> {
    Store::from_pool(pool).map_err(|_| Error::from_status(StatusCode::SERVICE_UNAVAILABLE))
}

/// The UserId the auth middleware inserted; 401 if the route isn't behind it.
fn authenticated_user(req: &Request) -> Result<String, Error> {
    req.extensions()
        .get::<UserId>()
        .map(|UserId(id)| id.clone())
        .ok_or_else(|| Error::from_status(StatusCode::UNAUTHORIZED))
}

fn map_db_error(err: DbError) -> Error {
    match err {
        DbError::NotFound => Error::from_status(StatusCode::NOT_FOUND),
        _ => Error::from_status(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

fn map_website_with_tick_to_output(website: WebsiteWithLatestTick) -> WebsiteOutputWithTick {
    WebsiteOutputWithTick {
        id: website.website.id,
        url: website.website.url,
        user_id: website.website.user_id,
        time_added: website.website.time_added,
        website_tick: website.latest_tick,
    }
}

fn map_website_to_output(website: Website) -> WebsiteOutput {
    WebsiteOutput {
        id: website.id,
        url: website.url,
        user_id: website.user_id,
        time_added: website.time_added,
    }
}

#[handler]
pub fn get_website(
    Path(website_id): Path<String>,
    Data(pool): Data<&DbPool>,
    req: &Request,
) -> Result<Json<WebsiteOutputWithTick>, Error> {
    let user_id = authenticated_user(req)?;
    let mut store = store_from_pool(pool)?;

    let website = store
        .get_website_by_id(website_id, &user_id)
        .map_err(map_db_error)?;

    Ok(Json(map_website_with_tick_to_output(website)))
}

#[handler]
pub fn create_website(
    Json(data): Json<CreateWebsiteInput>,
    Data(pool): Data<&DbPool>,
    req: &Request,
) -> Result<Json<CreateWebsiteOutput>, Error> {
    let user_id = authenticated_user(req)?;
    let mut store = store_from_pool(pool)?;

    let response = store
        .create_website(user_id, data.url)
        .map_err(map_db_error)?;

    Ok(Json(CreateWebsiteOutput {
        success: true,
        id: response.id,
    }))
}

#[handler]
pub fn delete_website(
    Path(website_id): Path<String>,
    Data(pool): Data<&DbPool>,
    req: &Request,
) -> Result<Json<DeleteWebsiteOutput>, Error> {
    let user_id = authenticated_user(req)?;
    let mut store = store_from_pool(pool)?;

    let deleted = store
        .delete_by_id(website_id, &user_id)
        .map_err(map_db_error)?;

    if !deleted {
        return Err(poem::Error::from_status(StatusCode::NOT_FOUND));
    }

    Ok(Json(DeleteWebsiteOutput { success: deleted }))
}

#[handler]
pub fn update_website(
    Path(website_id): Path<String>,
    Json(data): Json<UpdateWebsiteInput>,
    Data(pool): Data<&DbPool>,
    req: &Request,
) -> Result<Json<WebsiteOutput>, Error> {
    let user_id = authenticated_user(req)?;
    let mut store = store_from_pool(pool)?;

    // should have regex that match for url if pass then move ahead

    let updated = store
        .update_by_id(website_id, data.url, &user_id)
        .map_err(map_db_error)?;

    Ok(Json(map_website_to_output(updated)))
}

#[handler]
pub fn get_websites_by_user(
    Data(pool): Data<&DbPool>,
    req: &Request,
) -> Result<Json<WebsitesByUserOutput>, Error> {
    // the user comes from the verified token, not from the path
    let user_id = authenticated_user(req)?;
    let mut store = store_from_pool(pool)?;

    let websites = store
        .get_websites_by_user_id(user_id)
        .map_err(map_db_error)?;

    let output = WebsitesByUserOutput {
        websites: websites.into_iter().map(map_website_to_output).collect(),
    };

    Ok(Json(output))
}

#[handler]
pub fn get_status() -> Result<(), Error> {
    Ok(())
}

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
    DbPool, Store,
    models::website::{Website, WebsiteWithLatestTick},
};

fn store_from_pool(pool: &DbPool) -> Result<Store, Error> {
    Store::from_pool(pool).map_err(|_| Error::from_status(StatusCode::SERVICE_UNAVAILABLE))
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
) -> Result<Json<WebsiteOutputWithTick>, Error> {
    let mut store = store_from_pool(pool)?;

    let website = store
        .get_website_by_id(website_id)
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Json(map_website_with_tick_to_output(website)))
}

#[handler]
pub fn create_website(
    Json(data): Json<CreateWebsiteInput>,
    Data(pool): Data<&DbPool>,
    req: &Request,
) -> Result<Json<CreateWebsiteOutput>, Error> {
    let mut store = store_from_pool(pool)?;
    let user_id = req.extensions().get::<UserId>();

    match user_id {
        Some(u) => {
            let UserId(id) = u;
            let response = store
                .create_website(id.to_string(), data.url)
                .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;
            let new_website = CreateWebsiteOutput {
                success: true,
                id: response.id,
            };
            Ok(Json(new_website))
        }
        None => {
            println!("error occured");
            return Err(poem::Error::from_status(StatusCode::UNAUTHORIZED));
        }
    }
}

#[handler]
pub fn delete_website(
    Path(website_id): Path<String>,
    Data(pool): Data<&DbPool>,
) -> Result<Json<DeleteWebsiteOutput>, Error> {
    let mut store = store_from_pool(pool)?;

    let deleted = store
        .delete_by_id(website_id)
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

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
) -> Result<Json<WebsiteOutput>, Error> {
    let mut store = store_from_pool(pool)?;

    // should have regex that match for url if pass then move ahead

    let updated = store
        .update_by_id(website_id, data.url)
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Json(map_website_to_output(updated)))
}

#[handler]
pub fn get_websites_by_user(
    Path(user_id): Path<String>,
    Data(pool): Data<&DbPool>,
) -> Result<Json<WebsitesByUserOutput>, Error> {
    let mut store = store_from_pool(pool)?;

    let websites = store
        .get_websites_by_user_id(user_id)
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

    let output = WebsitesByUserOutput {
        websites: websites.into_iter().map(map_website_to_output).collect(),
    };

    Ok(Json(output))
}

#[handler]
pub fn get_status() -> Result<(), Error> {
    Ok(())
}

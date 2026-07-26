use chrono::{Duration, Utc};
use poem::{
    Error, Request, handler,
    http::StatusCode,
    web::{Data, Json, Path},
};
use store::{DatabaseErrorKind, DbError, DbPool, Store, models::website::WebsiteStatusEnum};

use crate::{
    middleware::auth::UserId,
    types::{
        request_input::{AddStatusPageMonitorInput, CreateStatusPageInput, UpdateStatusPageInput},
        request_output::{
            PublicStatusPageIncidentOutput, PublicStatusPageMonitorOutput, PublicStatusPageOutput,
            StatusPageActionOutput, StatusPageDetailOutput, StatusPageMonitorOutput,
            StatusPageOutput, StatusPagesOutput,
        },
    },
};

fn store_from_pool(pool: &DbPool) -> Result<Store, Error> {
    Store::from_pool(pool).map_err(|_| Error::from_status(StatusCode::SERVICE_UNAVAILABLE))
}

fn authenticated_user(req: &Request) -> Result<String, Error> {
    req.extensions()
        .get::<UserId>()
        .map(|UserId(id)| id.clone())
        .ok_or_else(|| Error::from_status(StatusCode::UNAUTHORIZED))
}

fn map_db_error(err: DbError) -> Error {
    match err {
        DbError::NotFound => Error::from_status(StatusCode::NOT_FOUND),
        DbError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
            Error::from_status(StatusCode::CONFLICT)
        }
        _ => Error::from_status(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Lowercase letters, digits and hyphens only - keeps the slug safe to drop
/// straight into a URL path with no escaping.
fn is_valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn map_status_page_to_output(page: store::models::status_page::StatusPage) -> StatusPageOutput {
    StatusPageOutput {
        id: page.id,
        slug: page.slug,
        title: page.title,
        created_at: page.created_at,
    }
}

#[handler]
pub fn create_status_page(
    Json(data): Json<CreateStatusPageInput>,
    Data(pool): Data<&DbPool>,
    req: &Request,
) -> Result<Json<StatusPageOutput>, Error> {
    let user_id = authenticated_user(req)?;

    if !is_valid_slug(&data.slug) {
        return Err(Error::from_status(StatusCode::BAD_REQUEST));
    }

    let mut store = store_from_pool(pool)?;

    let page = store
        .create_status_page(user_id, data.slug, data.title)
        .map_err(map_db_error)?;

    Ok(Json(map_status_page_to_output(page)))
}

#[handler]
pub fn get_status_pages(
    Data(pool): Data<&DbPool>,
    req: &Request,
) -> Result<Json<StatusPagesOutput>, Error> {
    let user_id = authenticated_user(req)?;
    let mut store = store_from_pool(pool)?;

    let pages = store
        .get_status_pages_by_user(&user_id)
        .map_err(map_db_error)?;

    Ok(Json(StatusPagesOutput {
        pages: pages.into_iter().map(map_status_page_to_output).collect(),
    }))
}

#[handler]
pub fn get_status_page(
    Path(status_page_id): Path<String>,
    Data(pool): Data<&DbPool>,
    req: &Request,
) -> Result<Json<StatusPageDetailOutput>, Error> {
    let user_id = authenticated_user(req)?;
    let mut store = store_from_pool(pool)?;

    let page = store
        .get_status_page_for_owner(&status_page_id, &user_id)
        .map_err(map_db_error)?;

    let monitor_rows = store
        .get_status_page_monitors_for_owner(&status_page_id, &user_id)
        .map_err(map_db_error)?;

    let mut monitors = Vec::with_capacity(monitor_rows.len());
    for monitor in monitor_rows {
        let website = store
            .get_website_by_id(monitor.website_id.clone(), &user_id)
            .map_err(map_db_error)?;
        monitors.push(StatusPageMonitorOutput {
            website_id: monitor.website_id,
            url: website.website.url,
            display_name: monitor.display_name,
            sort_order: monitor.sort_order,
        });
    }

    Ok(Json(StatusPageDetailOutput {
        id: page.id,
        slug: page.slug,
        title: page.title,
        monitors,
    }))
}

#[handler]
pub fn update_status_page(
    Path(status_page_id): Path<String>,
    Json(data): Json<UpdateStatusPageInput>,
    Data(pool): Data<&DbPool>,
    req: &Request,
) -> Result<Json<StatusPageOutput>, Error> {
    let user_id = authenticated_user(req)?;

    if !is_valid_slug(&data.slug) {
        return Err(Error::from_status(StatusCode::BAD_REQUEST));
    }

    let mut store = store_from_pool(pool)?;

    let page = store
        .update_status_page(&status_page_id, &user_id, data.slug, data.title)
        .map_err(map_db_error)?;

    Ok(Json(map_status_page_to_output(page)))
}

#[handler]
pub fn delete_status_page(
    Path(status_page_id): Path<String>,
    Data(pool): Data<&DbPool>,
    req: &Request,
) -> Result<Json<StatusPageActionOutput>, Error> {
    let user_id = authenticated_user(req)?;
    let mut store = store_from_pool(pool)?;

    let deleted = store
        .delete_status_page(&status_page_id, &user_id)
        .map_err(map_db_error)?;

    if !deleted {
        return Err(Error::from_status(StatusCode::NOT_FOUND));
    }

    Ok(Json(StatusPageActionOutput { success: true }))
}

#[handler]
pub fn add_status_page_monitor(
    Path(status_page_id): Path<String>,
    Json(data): Json<AddStatusPageMonitorInput>,
    Data(pool): Data<&DbPool>,
    req: &Request,
) -> Result<Json<StatusPageMonitorOutput>, Error> {
    let user_id = authenticated_user(req)?;
    let mut store = store_from_pool(pool)?;

    let monitor = store
        .add_status_page_monitor(
            &status_page_id,
            &user_id,
            data.website_id,
            data.display_name,
            data.sort_order,
        )
        .map_err(map_db_error)?;

    let website = store
        .get_website_by_id(monitor.website_id.clone(), &user_id)
        .map_err(map_db_error)?;

    Ok(Json(StatusPageMonitorOutput {
        website_id: monitor.website_id,
        url: website.website.url,
        display_name: monitor.display_name,
        sort_order: monitor.sort_order,
    }))
}

#[handler]
pub fn remove_status_page_monitor(
    Path((status_page_id, website_id)): Path<(String, String)>,
    Data(pool): Data<&DbPool>,
    req: &Request,
) -> Result<Json<StatusPageActionOutput>, Error> {
    let user_id = authenticated_user(req)?;
    let mut store = store_from_pool(pool)?;

    let removed = store
        .remove_status_page_monitor(&status_page_id, &user_id, &website_id)
        .map_err(map_db_error)?;

    if !removed {
        return Err(Error::from_status(StatusCode::NOT_FOUND));
    }

    Ok(Json(StatusPageActionOutput { success: true }))
}

/// Share of checks since `since` that were `Up`; `None` if there were no
/// checks in that window at all (not the same as 0% uptime).
fn uptime_percent(
    store: &mut Store,
    website_id: &str,
    since: chrono::NaiveDateTime,
) -> Result<Option<f64>, Error> {
    let (total, up) = store
        .count_ticks_since(website_id, since)
        .map_err(map_db_error)?;

    Ok(if total == 0 {
        None
    } else {
        Some(up as f64 / total as f64 * 100.0)
    })
}

#[handler]
pub fn get_public_status_page(
    Path(slug): Path<String>,
    Data(pool): Data<&DbPool>,
) -> Result<Json<PublicStatusPageOutput>, Error> {
    let mut store = store_from_pool(pool)?;

    let page = store
        .get_public_status_page_by_slug(&slug)
        .map_err(map_db_error)?
        .ok_or_else(|| Error::from_status(StatusCode::NOT_FOUND))?;

    let monitor_rows = store
        .get_status_page_monitors(&page.id)
        .map_err(map_db_error)?;

    let now = Utc::now().naive_utc();
    let since_24h = now - Duration::hours(24);
    let since_7d = now - Duration::days(7);
    let since_30d = now - Duration::days(30);

    let mut monitors = Vec::with_capacity(monitor_rows.len());
    let mut incidents = Vec::new();
    for monitor in monitor_rows {
        let latest_tick = store
            .get_latest_tick_by_website_id(&monitor.website_id)
            .map_err(map_db_error)?;
        let status = latest_tick
            .map(|tick| tick.status)
            .unwrap_or(WebsiteStatusEnum::Unknown);

        let uptime_24h = uptime_percent(&mut store, &monitor.website_id, since_24h)?;
        let uptime_7d = uptime_percent(&mut store, &monitor.website_id, since_7d)?;
        let uptime_30d = uptime_percent(&mut store, &monitor.website_id, since_30d)?;

        // Anything still open plus the last 30 days, labelled with the
        // monitor's public display name rather than its real URL.
        let monitor_incidents = store
            .get_public_incidents_since(&monitor.website_id, since_30d)
            .map_err(map_db_error)?;
        incidents.extend(
            monitor_incidents
                .into_iter()
                .map(|incident| PublicStatusPageIncidentOutput {
                    display_name: monitor.display_name.clone(),
                    started_at: incident.started_at,
                    resolved_at: incident.resolved_at,
                    cause: incident.cause,
                }),
        );

        monitors.push(PublicStatusPageMonitorOutput {
            display_name: monitor.display_name,
            status,
            uptime_24h,
            uptime_7d,
            uptime_30d,
        });
    }

    // Per-monitor lists are each newest-first; merge into one page-wide feed.
    incidents.sort_by(|a, b| b.started_at.cmp(&a.started_at));

    Ok(Json(PublicStatusPageOutput {
        title: page.title,
        monitors,
        incidents,
    }))
}

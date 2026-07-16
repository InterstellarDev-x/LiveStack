use poem::{
    Error, Request, handler,
    http::StatusCode,
    web::{Data, Json, Path},
};
use store::{DbError, DbPool, Store, models::incident::Incident};

use crate::{
    middleware::auth::UserId,
    types::request_output::{
        IncidentOutput, UserIncidentOutput, UserIncidentsOutput, WebsiteIncidentsOutput,
    },
};

const INCIDENTS_LIMIT: i64 = 50;

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
        _ => Error::from_status(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

fn duration_seconds(incident: &Incident) -> Option<i64> {
    incident
        .resolved_at
        .map(|resolved| (resolved - incident.started_at).num_seconds())
}

fn map_incident_to_output(incident: Incident) -> IncidentOutput {
    let duration = duration_seconds(&incident);
    IncidentOutput {
        id: incident.id,
        website_id: incident.website_id,
        started_at: incident.started_at,
        resolved_at: incident.resolved_at,
        cause: incident.cause,
        duration_seconds: duration,
    }
}

/// Outage history for one website, newest first. Owner-checked.
#[handler]
pub fn get_website_incidents(
    Path(website_id): Path<String>,
    Data(pool): Data<&DbPool>,
    req: &Request,
) -> Result<Json<WebsiteIncidentsOutput>, Error> {
    let user_id = authenticated_user(req)?;
    let mut store = store_from_pool(pool)?;

    let incidents = store
        .get_incidents_for_owner(&website_id, &user_id, INCIDENTS_LIMIT)
        .map_err(map_db_error)?;

    Ok(Json(WebsiteIncidentsOutput {
        incidents: incidents.into_iter().map(map_incident_to_output).collect(),
    }))
}

/// Account-wide incident feed across every website the user owns.
#[handler]
pub fn get_user_incidents(
    Data(pool): Data<&DbPool>,
    req: &Request,
) -> Result<Json<UserIncidentsOutput>, Error> {
    let user_id = authenticated_user(req)?;
    let mut store = store_from_pool(pool)?;

    let incidents = store
        .get_incidents_for_user(&user_id, INCIDENTS_LIMIT)
        .map_err(map_db_error)?;

    Ok(Json(UserIncidentsOutput {
        incidents: incidents
            .into_iter()
            .map(|(incident, url)| {
                let duration = duration_seconds(&incident);
                UserIncidentOutput {
                    id: incident.id,
                    website_id: incident.website_id,
                    url,
                    started_at: incident.started_at,
                    resolved_at: incident.resolved_at,
                    cause: incident.cause,
                    duration_seconds: duration,
                }
            })
            .collect(),
    }))
}

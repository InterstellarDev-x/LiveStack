use crate::Store;
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// One continuous outage of a website: opened when consecutive failed checks
/// confirm the site is down, resolved when a check succeeds again.
/// `resolved_at: None` means the outage is still ongoing.
#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::incident)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Serialize, Deserialize)]
pub struct Incident {
    pub id: String,
    pub website_id: String,
    pub started_at: NaiveDateTime,
    pub resolved_at: Option<NaiveDateTime>,
    pub cause: String,
}

impl Store {
    /// Atomically opens an incident for this website. The partial unique index
    /// (`one_open_incident_per_website`) turns concurrent opens into a single
    /// winner: `Some(incident)` means this caller created it and owns the
    /// "incident opened" transition (i.e. should publish the alert); `None`
    /// means an incident was already open, so this is a no-op.
    pub fn open_incident(
        &mut self,
        input_website_id: &str,
        input_cause: &str,
    ) -> Result<Option<Incident>, diesel::result::Error> {
        use crate::schema::incident::dsl::*;

        let new_incident = Incident {
            id: Uuid::new_v4().to_string(),
            website_id: input_website_id.to_string(),
            started_at: Utc::now().naive_utc(),
            resolved_at: None,
            cause: input_cause.to_string(),
        };

        diesel::insert_into(incident)
            .values(&new_incident)
            .on_conflict_do_nothing()
            .returning(Incident::as_returning())
            .get_result(self.conn())
            .optional()
    }

    /// Atomically resolves this website's open incident, if any. A returned
    /// row (with `resolved_at` now set) means this caller performed the
    /// resolve and owns the "recovered" transition; `None` means there was
    /// nothing open — e.g. a blip that never crossed the open threshold.
    pub fn resolve_incident(
        &mut self,
        input_website_id: &str,
    ) -> Result<Option<Incident>, diesel::result::Error> {
        use crate::schema::incident::dsl::*;

        diesel::update(
            incident
                .filter(website_id.eq(input_website_id))
                .filter(resolved_at.is_null()),
        )
        .set(resolved_at.eq(Some(Utc::now().naive_utc())))
        .returning(Incident::as_returning())
        .get_result(self.conn())
        .optional()
    }

    /// Owner-checked incident history for one website, newest first.
    /// 404s (as `NotFound`) if the website isn't owned by this user.
    pub fn get_incidents_for_owner(
        &mut self,
        input_website_id: &str,
        input_user_id: &str,
        limit: i64,
    ) -> Result<Vec<Incident>, diesel::result::Error> {
        use crate::schema::incident::dsl::*;

        if !self.website_belongs_to_user(input_website_id, input_user_id)? {
            return Err(diesel::result::Error::NotFound);
        }

        incident
            .filter(website_id.eq(input_website_id))
            .order(started_at.desc())
            .limit(limit)
            .select(Incident::as_select())
            .load(self.conn())
    }

    /// All incidents across every website this user owns, newest first,
    /// paired with the website's URL so callers don't need a second lookup.
    pub fn get_incidents_for_user(
        &mut self,
        input_user_id: &str,
        limit: i64,
    ) -> Result<Vec<(Incident, String)>, diesel::result::Error> {
        use crate::schema::incident;
        use crate::schema::website;

        incident::table
            .inner_join(website::table)
            .filter(website::user_id.eq(input_user_id))
            .order(incident::started_at.desc())
            .limit(limit)
            .select((Incident::as_select(), website::url))
            .load(self.conn())
    }

    /// Incidents worth showing on a public status page: anything still open
    /// (however old) plus anything that started at or after `since`. Newest
    /// first. No ownership check — callers pass website ids they already
    /// resolved from a published status page.
    pub fn get_public_incidents_since(
        &mut self,
        input_website_id: &str,
        since: NaiveDateTime,
    ) -> Result<Vec<Incident>, diesel::result::Error> {
        use crate::schema::incident::dsl::*;

        incident
            .filter(website_id.eq(input_website_id))
            .filter(resolved_at.is_null().or(started_at.ge(since)))
            .order(started_at.desc())
            .select(Incident::as_select())
            .load(self.conn())
    }
}

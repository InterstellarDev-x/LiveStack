use crate::{Store, schema::sql_types::WebsiteStatus};
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::website)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Serialize, Deserialize, Debug)]
pub struct Website {
    pub id: String,
    pub url: String,
    pub time_added: NaiveDateTime,
    pub user_id: String,
}

#[derive(Debug, DbEnum, PartialEq, Eq, Clone, Copy)]
#[ExistingTypePath = "WebsiteStatus"]
#[DbValueStyle = "verbatim"]
#[derive(Serialize, Deserialize)]
pub enum WebsiteStatusEnum {
    Up,
    Down,
    Unknown,
}

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::website_tick)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Serialize, Deserialize)]
pub struct WebsiteTick {
    pub id: String,
    pub response_time_ms: i32,
    pub status: WebsiteStatusEnum,
    pub region_id: String,
    pub website_id: String,
    #[diesel(column_name = createdAt)]
    #[serde(rename = "createdAt")]
    pub created_at: NaiveDateTime,
    pub dns_time_ms: i32,
    pub connection_time_ms: i32,
    pub tls_time_ms: i32,
    pub data_transfer_time_ms: i32,
    pub waiting_time_ms: i32,
}

/// Per-phase curl timings for a single website check, in whole milliseconds.
#[derive(Clone, Copy)]
pub struct NewWebsiteTickTiming {
    pub dns_time_ms: i32,
    pub connection_time_ms: i32,
    pub tls_time_ms: i32,
    /// Time to first byte: server think-time after the connection was ready.
    pub waiting_time_ms: i32,
    /// Time spent streaming the body in after the first byte arrived.
    pub data_transfer_time_ms: i32,
}

#[derive(Serialize, Deserialize)]
pub struct WebsiteWithLatestTick {
    pub website: Website,
    pub latest_tick: Option<WebsiteTick>,
}

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::region)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Region {
    pub id: String,
    pub name: String,
}

impl Store {
    pub fn create_website(
        &mut self,
        input_user_id: String,
        url: String,
    ) -> Result<Website, diesel::result::Error> {
        let website_id = Uuid::new_v4().to_string();
        let new_webiste = Website {
            id: website_id,
            url: url,
            time_added: Utc::now().naive_utc(),
            user_id: input_user_id,
        };

        let response = diesel::insert_into(crate::schema::website::table)
            .values(new_webiste)
            .returning(Website::as_returning())
            .get_result(self.conn())?;

        Ok(response)
    }

    pub fn get_website_by_id(
        &mut self,
        input_website_id: String,
        input_user_id: &str,
    ) -> Result<WebsiteWithLatestTick, diesel::result::Error> {
        use crate::schema::website::dsl::*;

        let website_result = website
            .filter(id.eq(&input_website_id))
            .filter(user_id.eq(input_user_id)) // only the owner can read it
            .select(Website::as_select())
            .first(self.conn())?;

        let latest_ticks: Option<WebsiteTick>;
        {
            use crate::schema::website_tick::dsl::*;

            latest_ticks = website_tick
                .filter(website_id.eq(&input_website_id))
                .order(createdAt.desc())
                .select(WebsiteTick::as_select())
                .first::<WebsiteTick>(self.conn())
                .optional()?;
        }

        match latest_ticks {
            Some(latest) => Ok(WebsiteWithLatestTick {
                website: website_result,
                latest_tick: Some(latest),
            }),
            None => Ok(WebsiteWithLatestTick {
                website: website_result,
                latest_tick: None,
            }),
        }
    }

    pub fn delete_by_id(
        &mut self,
        input_website_id: String,
        input_user_id: &str,
    ) -> Result<bool, diesel::result::Error> {
        use crate::schema::website::dsl::*;

        let deleted_site = diesel::delete(website)
            .filter(id.eq(input_website_id))
            .filter(user_id.eq(input_user_id)) // only the owner can delete it
            .execute(self.conn())?;

        Ok(deleted_site > 0)
    }

    pub fn update_by_id(
        &mut self,
        input_website_id: String,
        updated_url: String,
        input_user_id: &str,
    ) -> Result<Website, diesel::result::Error> {
        use crate::schema::website::dsl::*;

        let updated_site = diesel::update(
            website
                .filter(id.eq(input_website_id))
                .filter(user_id.eq(input_user_id)), // only the owner can update it
        )
        .set(url.eq(updated_url))
        .get_result(self.conn())?;

        return Ok(updated_site);
    }

    /// Ordered oldest-first. Without an explicit order Postgres is free to
    /// return rows in any order, which in practice means the monitor list
    /// reshuffles after an update moves a row.
    pub fn get_websites_by_user_id(
        &mut self,
        input_user_id: String,
    ) -> Result<Vec<Website>, diesel::result::Error> {
        use crate::schema::website::dsl::*;
        let response = website
            .filter(user_id.eq(input_user_id))
            .order(time_added.asc())
            .load::<Website>(self.conn())?;
        return Ok(response);
    }

    // for producer to proudce
    pub fn get_all_websites(&mut self) -> Result<Vec<Website>, diesel::result::Error> {
        use crate::schema::website::dsl::*;
        let response = website.load::<Website>(self.conn())?;
        return Ok(response);
    }

    pub fn ensure_region(
        &mut self,
        input_region_id: String,
        input_region_name: String,
    ) -> Result<(), diesel::result::Error> {
        use crate::schema::region::dsl::*;

        let new_region = Region {
            id: input_region_id,
            name: input_region_name,
        };

        diesel::insert_into(region)
            .values(new_region)
            .on_conflict(id)
            .do_nothing()
            .execute(self.conn())?;

        Ok(())
    }

    pub fn get_latest_ticks_by_website_id(
        &mut self,
        input_website_id: &str,
        limit: i64,
    ) -> Result<Vec<WebsiteTick>, diesel::result::Error> {
        use crate::schema::website_tick::dsl::*;

        website_tick
            .filter(website_id.eq(input_website_id))
            .order(createdAt.desc())
            .limit(limit)
            .select(WebsiteTick::as_select())
            .load::<WebsiteTick>(self.conn())
    }

    /// The single most recent tick, if any - used to show a monitor's
    /// current status without pulling a whole history.
    pub fn get_latest_tick_by_website_id(
        &mut self,
        input_website_id: &str,
    ) -> Result<Option<WebsiteTick>, diesel::result::Error> {
        use crate::schema::website_tick::dsl::*;

        website_tick
            .filter(website_id.eq(input_website_id))
            .order(createdAt.desc())
            .select(WebsiteTick::as_select())
            .first(self.conn())
            .optional()
    }

    /// `(total, up)` tick counts at or after `since`, computed in the
    /// database.
    ///
    /// The public status page needs three of these (24h/7d/30d) per published
    /// monitor and nothing else about the rows. Loading a 30-day window into
    /// memory to count it means transferring thousands of rows per monitor on
    /// every anonymous page view; two aggregates over an index do not.
    pub fn count_ticks_since(
        &mut self,
        input_website_id: &str,
        since: NaiveDateTime,
    ) -> Result<(i64, i64), diesel::result::Error> {
        use crate::schema::website_tick::dsl::*;
        use diesel::dsl::count_star;

        let total: i64 = website_tick
            .filter(website_id.eq(input_website_id))
            .filter(createdAt.ge(since))
            .select(count_star())
            .first(self.conn())?;

        let up: i64 = website_tick
            .filter(website_id.eq(input_website_id))
            .filter(createdAt.ge(since))
            .filter(status.eq(WebsiteStatusEnum::Up))
            .select(count_star())
            .first(self.conn())?;

        Ok((total, up))
    }

    /// All ticks at or after `since` - the caller buckets these into
    /// whatever uptime windows it needs (e.g. 24h/7d/30d).
    pub fn get_ticks_since(
        &mut self,
        input_website_id: &str,
        since: NaiveDateTime,
    ) -> Result<Vec<WebsiteTick>, diesel::result::Error> {
        use crate::schema::website_tick::dsl::*;

        website_tick
            .filter(website_id.eq(input_website_id))
            .filter(createdAt.ge(since))
            .select(WebsiteTick::as_select())
            .load::<WebsiteTick>(self.conn())
    }

    /// Read-only ownership check with no other side effects - used by
    /// features (e.g. status pages) that need to confirm a website belongs
    /// to a user without pulling the whole row.
    pub fn website_belongs_to_user(
        &mut self,
        input_website_id: &str,
        input_user_id: &str,
    ) -> Result<bool, diesel::result::Error> {
        use crate::schema::website::dsl::*;

        let owner: Option<String> = website
            .filter(id.eq(input_website_id))
            .select(user_id)
            .first(self.conn())
            .optional()?;

        Ok(owner.as_deref() == Some(input_user_id))
    }

    pub fn create_website_tick(
        &mut self,
        input_website_id: String,
        input_region_id: String,
        input_response_time_ms: i32,
        input_status: WebsiteStatusEnum,
        timing: NewWebsiteTickTiming,
    ) -> Result<WebsiteTick, diesel::result::Error> {
        let new_tick = WebsiteTick {
            id: Uuid::new_v4().to_string(),
            response_time_ms: input_response_time_ms,
            status: input_status,
            region_id: input_region_id,
            website_id: input_website_id,
            created_at: Utc::now().naive_utc(),
            dns_time_ms: timing.dns_time_ms,
            connection_time_ms: timing.connection_time_ms,
            tls_time_ms: timing.tls_time_ms,
            waiting_time_ms: timing.waiting_time_ms,
            data_transfer_time_ms: timing.data_transfer_time_ms,
        };

        let response = diesel::insert_into(crate::schema::website_tick::table)
            .values(new_tick)
            .returning(WebsiteTick::as_returning())
            .get_result(self.conn())?;

        Ok(response)
    }
}

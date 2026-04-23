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

#[derive(Debug, DbEnum)]
#[ExistingTypePath = "WebsiteStatus"]
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
    pub createdAt: NaiveDateTime,
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
            .get_result(&mut self.conn)?;

        Ok(response)
    }

    pub fn get_website_by_id(
        &mut self,
        input_website_id: String,
    ) -> Result<WebsiteWithLatestTick, diesel::result::Error> {
        use crate::schema::website::dsl::*;

        let website_result = website
            .filter(id.eq(&input_website_id))
            .select(Website::as_select())
            .first(&mut self.conn)?;

        let latest_ticks: Option<WebsiteTick>;
        {
            use crate::schema::website_tick::dsl::*;

            latest_ticks = website_tick
                .filter(website_id.eq(&input_website_id))
                .order(createdAt.desc())
                .select(WebsiteTick::as_select())
                .first::<WebsiteTick>(&mut self.conn)
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
    ) -> Result<bool, diesel::result::Error> {
        use crate::schema::website::dsl::*;

        let deleted_site = diesel::delete(website)
            .filter(id.eq(input_website_id))
            .execute(&mut self.conn)?;

        Ok(deleted_site > 0)
    }

    pub fn update_by_id(
        &mut self,
        input_website_id: String,
        updated_url: String,
    ) -> Result<Website, diesel::result::Error> {
        use crate::schema::website::dsl::*;

        let updated_site = diesel::update(website.filter(id.eq(input_website_id)))
            .set(url.eq(updated_url))
            .get_result(&mut self.conn)?;

        return Ok(updated_site);
    }

    pub fn get_websites_by_user_id(
        &mut self,
        input_user_id: String,
    ) -> Result<Vec<Website>, diesel::result::Error> {
        use crate::schema::website::dsl::*;
        let response = website
            .filter(user_id.eq(input_user_id))
            .load::<Website>(&mut self.conn)?;
        return Ok(response);
    }

    // for producer to proudce
    pub fn get_all_websites(&mut self) -> Result<Vec<Website>, diesel::result::Error> {
        use crate::schema::website::dsl::*;
        let response = website.load::<Website>(&mut self.conn)?;
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
            .execute(&mut self.conn)?;

        Ok(())
    }

    pub fn create_website_tick(
        &mut self,
        input_website_id: String,
        input_region_id: String,
        input_response_time_ms: i32,
        input_status: WebsiteStatusEnum,
    ) -> Result<WebsiteTick, diesel::result::Error> {
        let new_tick = WebsiteTick {
            id: Uuid::new_v4().to_string(),
            response_time_ms: input_response_time_ms,
            status: input_status,
            region_id: input_region_id,
            website_id: input_website_id,
            createdAt: Utc::now().naive_utc(),
        };

        let response = diesel::insert_into(crate::schema::website_tick::table)
            .values(new_tick)
            .returning(WebsiteTick::as_returning())
            .get_result(&mut self.conn)?;

        Ok(response)
    }
}

use crate::Store;
use chrono::{NaiveDateTime, Utc};
use diesel::{ prelude::*};

use uuid::Uuid;
#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::website)]
#[diesel(check_for_backend(diesel::pg::Pg))]

pub struct Website {
    pub id: String,
    pub url: String,
    pub time_added: NaiveDateTime,
    pub user_id: String,
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
    ) -> Result<Website, diesel::result::Error> {
        use crate::schema::website::dsl::*;

        let website_result = website
            .filter(id.eq(input_website_id))
            .select(Website::as_select())
            .first(&mut self.conn)?;

        return Ok(website_result);
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


    pub fn get_websites_by_user_id(&mut self , input_user_id: String) -> Result<Vec<Website> , diesel::result::Error> {
           use crate::schema::website::dsl::*;
           let response = website.filter(user_id.eq(input_user_id)).load::<Website>(&mut self.conn)?;
           return Ok(response);
    }
}

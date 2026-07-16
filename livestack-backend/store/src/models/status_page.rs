use crate::Store;
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::status_page)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Serialize, Deserialize)]
pub struct StatusPage {
    pub id: String,
    pub user_id: String,
    pub slug: String,
    pub title: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::status_page_monitor)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Serialize, Deserialize)]
pub struct StatusPageMonitor {
    pub id: String,
    pub status_page_id: String,
    pub website_id: String,
    pub display_name: String,
    pub sort_order: i32,
}

/// Confirms `input_user_id` owns `input_status_page_id`, 404-ing (as
/// `NotFound`) if not.
fn assert_owns_status_page(
    conn: &mut diesel::pg::PgConnection,
    input_status_page_id: &str,
    input_user_id: &str,
) -> Result<(), diesel::result::Error> {
    use crate::schema::status_page::dsl as status_page_dsl;

    let owner_id: String = status_page_dsl::status_page
        .filter(status_page_dsl::id.eq(input_status_page_id))
        .select(status_page_dsl::user_id)
        .first(conn)?;

    if owner_id != input_user_id {
        return Err(diesel::result::Error::NotFound);
    }

    Ok(())
}

impl Store {
    pub fn create_status_page(
        &mut self,
        input_user_id: String,
        input_slug: String,
        input_title: String,
    ) -> Result<StatusPage, diesel::result::Error> {
        let now = Utc::now().naive_utc();
        let new_page = StatusPage {
            id: Uuid::new_v4().to_string(),
            user_id: input_user_id,
            slug: input_slug,
            title: input_title,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(crate::schema::status_page::table)
            .values(&new_page)
            .returning(StatusPage::as_returning())
            .get_result(self.conn())
    }

    pub fn get_status_pages_by_user(
        &mut self,
        input_user_id: &str,
    ) -> Result<Vec<StatusPage>, diesel::result::Error> {
        use crate::schema::status_page::dsl::*;

        status_page
            .filter(user_id.eq(input_user_id))
            .select(StatusPage::as_select())
            .load(self.conn())
    }

    pub fn get_status_page_for_owner(
        &mut self,
        input_status_page_id: &str,
        input_user_id: &str,
    ) -> Result<StatusPage, diesel::result::Error> {
        use crate::schema::status_page::dsl::*;

        assert_owns_status_page(self.conn(), input_status_page_id, input_user_id)?;

        status_page
            .filter(id.eq(input_status_page_id))
            .select(StatusPage::as_select())
            .first(self.conn())
    }

    pub fn update_status_page(
        &mut self,
        input_status_page_id: &str,
        input_user_id: &str,
        input_slug: String,
        input_title: String,
    ) -> Result<StatusPage, diesel::result::Error> {
        use crate::schema::status_page::dsl::*;

        assert_owns_status_page(self.conn(), input_status_page_id, input_user_id)?;

        diesel::update(status_page.filter(id.eq(input_status_page_id)))
            .set((
                slug.eq(input_slug),
                title.eq(input_title),
                updated_at.eq(Utc::now().naive_utc()),
            ))
            .returning(StatusPage::as_returning())
            .get_result(self.conn())
    }

    pub fn delete_status_page(
        &mut self,
        input_status_page_id: &str,
        input_user_id: &str,
    ) -> Result<bool, diesel::result::Error> {
        use crate::schema::status_page::dsl::*;

        let deleted = diesel::delete(
            status_page
                .filter(id.eq(input_status_page_id))
                .filter(user_id.eq(input_user_id)),
        )
        .execute(self.conn())?;

        Ok(deleted > 0)
    }

    /// The page and the website must both belong to `input_user_id` - this
    /// is what stops an owner publishing someone else's monitor.
    pub fn add_status_page_monitor(
        &mut self,
        input_status_page_id: &str,
        input_user_id: &str,
        input_website_id: String,
        input_display_name: String,
        input_sort_order: i32,
    ) -> Result<StatusPageMonitor, diesel::result::Error> {
        assert_owns_status_page(self.conn(), input_status_page_id, input_user_id)?;

        if !self.website_belongs_to_user(&input_website_id, input_user_id)? {
            return Err(diesel::result::Error::NotFound);
        }

        use crate::schema::status_page_monitor::dsl::*;

        let new_monitor = StatusPageMonitor {
            id: Uuid::new_v4().to_string(),
            status_page_id: input_status_page_id.to_string(),
            website_id: input_website_id,
            display_name: input_display_name,
            sort_order: input_sort_order,
        };

        diesel::insert_into(status_page_monitor)
            .values(&new_monitor)
            .returning(StatusPageMonitor::as_returning())
            .get_result(self.conn())
    }

    pub fn get_status_page_monitors_for_owner(
        &mut self,
        input_status_page_id: &str,
        input_user_id: &str,
    ) -> Result<Vec<StatusPageMonitor>, diesel::result::Error> {
        assert_owns_status_page(self.conn(), input_status_page_id, input_user_id)?;

        self.get_status_page_monitors(input_status_page_id)
    }

    pub fn remove_status_page_monitor(
        &mut self,
        input_status_page_id: &str,
        input_user_id: &str,
        input_website_id: &str,
    ) -> Result<bool, diesel::result::Error> {
        assert_owns_status_page(self.conn(), input_status_page_id, input_user_id)?;

        use crate::schema::status_page_monitor::dsl::*;

        let deleted = diesel::delete(
            status_page_monitor
                .filter(status_page_id.eq(input_status_page_id))
                .filter(website_id.eq(input_website_id)),
        )
        .execute(self.conn())?;

        Ok(deleted > 0)
    }

    /// Used by the public status page route, which acts without an
    /// authenticated owner - no owner check here.
    pub fn get_public_status_page_by_slug(
        &mut self,
        input_slug: &str,
    ) -> Result<Option<StatusPage>, diesel::result::Error> {
        use crate::schema::status_page::dsl::*;

        status_page
            .filter(slug.eq(input_slug))
            .select(StatusPage::as_select())
            .first(self.conn())
            .optional()
    }

    /// Used by both the owner editor and the public route - no owner check
    /// here, ordered for stable public display.
    pub fn get_status_page_monitors(
        &mut self,
        input_status_page_id: &str,
    ) -> Result<Vec<StatusPageMonitor>, diesel::result::Error> {
        use crate::schema::status_page_monitor::dsl::*;

        status_page_monitor
            .filter(status_page_id.eq(input_status_page_id))
            .order(sort_order.asc())
            .select(StatusPageMonitor::as_select())
            .load(self.conn())
    }
}

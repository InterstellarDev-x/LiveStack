use crate::Store;
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::website_notification_config)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Serialize, Deserialize)]
pub struct WebsiteNotificationConfig {
    pub website_id: String,
    pub webhook_url: Option<String>,
    pub webhook_secret: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub webhook_enabled: bool,
}

/// Confirms `input_user_id` owns `input_website_id`, 404-ing (as `NotFound`) if not.
fn assert_owns_website(
    conn: &mut diesel::pg::PgConnection,
    input_website_id: &str,
    input_user_id: &str,
) -> Result<(), diesel::result::Error> {
    use crate::schema::website::dsl as website_dsl;

    let owner_id: String = website_dsl::website
        .filter(website_dsl::id.eq(input_website_id))
        .select(website_dsl::user_id)
        .first(conn)?;

    if owner_id != input_user_id {
        return Err(diesel::result::Error::NotFound);
    }

    Ok(())
}

impl Store {
    /// Only the website's owner may set its webhook. `webhook_url: None` clears it.
    /// The signing secret is generated once, the first time this website gets a
    /// notification config row, and stays stable across later URL updates.
    pub fn upsert_website_webhook(
        &mut self,
        input_website_id: String,
        input_user_id: &str,
        input_webhook_url: Option<String>,
        input_webhook_enabled: bool,
    ) -> Result<WebsiteNotificationConfig, diesel::result::Error> {
        use crate::schema::website_notification_config::dsl::*;

        assert_owns_website(self.conn(), &input_website_id, input_user_id)?;

        let now = Utc::now().naive_utc();
        let new_secret = Uuid::new_v4().simple().to_string();

        diesel::insert_into(website_notification_config)
            .values((
                website_id.eq(&input_website_id),
                webhook_url.eq(&input_webhook_url),
                webhook_secret.eq(Some(new_secret)),
                webhook_enabled.eq(input_webhook_enabled),
                created_at.eq(now),
                updated_at.eq(now),
            ))
            .on_conflict(website_id)
            .do_update()
            .set((
                webhook_url.eq(&input_webhook_url),
                webhook_enabled.eq(input_webhook_enabled),
                updated_at.eq(now),
            ))
            .returning(WebsiteNotificationConfig::as_returning())
            .get_result(self.conn())
    }

    /// Used by notification workers, which act on behalf of the system
    /// rather than an authenticated user, so there is no owner check here.
    pub fn get_notification_config(
        &mut self,
        input_website_id: &str,
    ) -> Result<Option<WebsiteNotificationConfig>, diesel::result::Error> {
        use crate::schema::website_notification_config::dsl::*;

        website_notification_config
            .filter(website_id.eq(input_website_id))
            .select(WebsiteNotificationConfig::as_select())
            .first(self.conn())
            .optional()
    }

    /// Owner-checked read, for the website's own settings UI.
    pub fn get_notification_config_for_owner(
        &mut self,
        input_website_id: &str,
        input_user_id: &str,
    ) -> Result<Option<WebsiteNotificationConfig>, diesel::result::Error> {
        assert_owns_website(self.conn(), input_website_id, input_user_id)?;

        self.get_notification_config(input_website_id)
    }

    /// Owner-checked. Rotates the signing secret without touching the URL or
    /// enabled flag. 404s (as `NotFound`) if no config row exists yet.
    pub fn regenerate_webhook_secret(
        &mut self,
        input_website_id: &str,
        input_user_id: &str,
    ) -> Result<WebsiteNotificationConfig, diesel::result::Error> {
        use crate::schema::website_notification_config::dsl::*;

        assert_owns_website(self.conn(), input_website_id, input_user_id)?;

        let new_secret = Uuid::new_v4().simple().to_string();
        let now = Utc::now().naive_utc();

        diesel::update(website_notification_config.filter(website_id.eq(input_website_id)))
            .set((webhook_secret.eq(Some(new_secret)), updated_at.eq(now)))
            .returning(WebsiteNotificationConfig::as_returning())
            .get_result(self.conn())
    }

    /// Used by notification workers to resolve who owns a website so its
    /// email alert can be routed, without requiring a caller-supplied user_id.
    pub fn get_website_owner_user_id(
        &mut self,
        input_website_id: &str,
    ) -> Result<String, diesel::result::Error> {
        use crate::schema::website::dsl::*;

        website
            .filter(id.eq(input_website_id))
            .select(user_id)
            .first(self.conn())
    }
}

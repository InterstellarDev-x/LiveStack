use crate::Store;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::user)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: String,
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub email_alerts_enabled: bool,
}

use uuid::Uuid;

impl Store {
    pub fn create_user(
        &mut self,
        input_username: String,
        input_paasword: String,
    ) -> Result<User, diesel::result::Error> {
        let user_id = Uuid::new_v4().to_string();

        let new_user = User {
            id: user_id,
            username: input_username,
            password: input_paasword,
            email: None,
            email_alerts_enabled: true,
        };

        let response = diesel::insert_into(crate::schema::user::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(self.conn())?;

        Ok(response)
    }

    pub fn is_user_exist(
        &mut self,
        input_username: &String,
    ) -> Result<bool, diesel::result::Error> {
        use crate::schema::user::dsl::*;

        let user_result = user
            .filter(username.eq(input_username))
            .select(User::as_select())
            .first(self.conn())
            .optional()?;

        Ok(match user_result {
            Some(_u) => true,
            None => false,
        })
    }

    pub fn get_user_by_username(
        &mut self,
        input_username: &String,
    ) -> Result<Option<User>, diesel::result::Error> {
        use crate::schema::user::dsl::*;

        let user_result = user
            .filter(username.eq(input_username))
            .select(User::as_select())
            .first(self.conn())
            .optional()?;

        return Ok(user_result);
    }

    pub fn update_user_email(
        &mut self,
        input_user_id: String,
        input_email: String,
    ) -> Result<User, diesel::result::Error> {
        use crate::schema::user::dsl::*;

        diesel::update(user.filter(id.eq(input_user_id)))
            .set(email.eq(Some(input_email)))
            .get_result(self.conn())
    }

    /// Used by notification workers, which act on behalf of the system
    /// rather than an authenticated user, so there is no owner check here.
    /// Returns `None` if the user no longer exists, has no email, or has
    /// turned email alerts off — all "nothing to send", not errors.
    pub fn get_user_email(
        &mut self,
        input_user_id: &str,
    ) -> Result<Option<String>, diesel::result::Error> {
        use crate::schema::user::dsl::*;

        let row: Option<(Option<String>, bool)> = user
            .filter(id.eq(input_user_id))
            .select((email, email_alerts_enabled))
            .first(self.conn())
            .optional()?;

        Ok(match row {
            Some((user_email, true)) => user_email,
            _ => None,
        })
    }

    pub fn get_user_by_id(
        &mut self,
        input_user_id: &str,
    ) -> Result<User, diesel::result::Error> {
        use crate::schema::user::dsl::*;

        user.filter(id.eq(input_user_id))
            .select(User::as_select())
            .first(self.conn())
    }

    pub fn set_email_alerts_enabled(
        &mut self,
        input_user_id: String,
        enabled: bool,
    ) -> Result<User, diesel::result::Error> {
        use crate::schema::user::dsl::*;

        diesel::update(user.filter(id.eq(input_user_id)))
            .set(email_alerts_enabled.eq(enabled))
            .get_result(self.conn())
    }
}

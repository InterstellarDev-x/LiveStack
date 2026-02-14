use crate::Store;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::user)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: String,
    pub username: String,
    pub password: String,
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
        };

        let response = diesel::insert_into(crate::schema::user::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(&mut self.conn)?;

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
            .first(&mut self.conn)
            .optional()?;

        Ok(match user_result {
            Some(_u) => true,
            None => false,
        })
    }

    pub fn is_exist_and_password_match(
        &mut self,
        input_username: &String,
        input_paasword: &String,
    ) -> Result<bool, diesel::result::Error> {
        use crate::schema::user::dsl::*;

        let user_result = user
            .filter(username.eq(input_username))
            .select(User::as_select())
            .first(&mut self.conn)
            .optional()?;

        Ok(match user_result {
            Some(u) => u.password == *input_paasword,
            None => false,
        })
    }
}

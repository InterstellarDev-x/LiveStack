use diesel::{ExpressionMethods, RunQueryDsl, Selectable, SelectableHelper, prelude::{Insertable, Queryable}, query_dsl::methods::{FilterDsl, SelectDsl}};

use crate::Store;

#[derive(Queryable, Selectable , Insertable)]
#[diesel(table_name = crate::schema::user)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
   pub   id : String,
   pub  username : String,
   pub  password : String
}




 impl  Store  {
   pub fn signp (&mut self , username : String , password : String)  -> Result<User , diesel::result::Error> {
      
      let id = String::from("id");
      let u = User {
         id,
         username,
         password
      };  
      use crate::schema::user;
      let response = diesel::insert_into(user::table)
                           .values(&u)
                           .returning(User::as_returning()).get_result(&mut self.conn)?;


      Ok(response)
   }


pub fn sign_in(&mut self , input_username : String, input_password : String) -> Result<bool , diesel::result::Error>{
   
    use crate::schema::user::dsl::*;

   let user_result  =   user
                              .filter(username.eq(input_username))
                              .select(User::as_select())
                              .first(&mut self.conn)?;

   if user_result.password != input_password {
      return Ok(false);
   }

   return Ok(true);

}
}





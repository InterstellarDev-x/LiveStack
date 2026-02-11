pub struct  Store {
   conn : Connection
}



impl  Default for Store {

     fn default() -> Self {


        Store {
            conn     
        }
    }
}



impl  Store {
    pub fn create_user(&self){
        print!("crated user")
    }

    pub fn create_website(&self) -> String{
        format!("created Website")
    }
}
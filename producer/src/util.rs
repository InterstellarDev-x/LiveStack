use store::models::website::Website;

#[derive(Debug)]
pub struct  Messages {
 pub  id : String,
 pub url : String
}


pub fn website_to_message(websites : Vec<Website>) -> Vec<Messages> {
  
  let mut messages  = Vec::new();

  for sites in websites {

    messages.push(Messages {
        id : sites.id,
        url : sites.url
    });
  }

  messages

}
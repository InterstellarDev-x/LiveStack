use std::{collections::HashMap, time::Duration};
pub mod pool;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {


    let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(10))
    .build()?;


    //for connection pooling keep-alive sockets DNS caching TLS session reuse This drastically reduces overhead.

    loop {

         let resp = reqwest::get("https://jsonplaceholder.typicode.com/todos/1")
        .await?
        .json::<HashMap<String, String>>()
        .await?;
         println!("{resp:#?}");
       

    }
   
}
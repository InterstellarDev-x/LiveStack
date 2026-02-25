use jsonwebtoken::{DecodingKey , Validation, decode, errors::ErrorKind};
use poem::{
     Endpoint, IntoResponse, Request, Response,
    Result,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    user_id: String,
}


pub async fn log<E: Endpoint>(next: E, req: Request) -> Result<Response> {
    println!("request: {}", req.uri().path());
    let res = next.call(req).await;
    
    // let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2lkIjoiMGUzMDk3YmQtMTU2MC00Y2RlLWEwZWItYTc2ZmNhZTljZTRiIn0.-6IoDRN6FMkJyNw-E3wHmTcknk58V2tE44Rgk8BcsXM";

    

    // let validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    // let key = b"fdfad"; 
    
    // // validation.sub = Some() 

    // let token_data    = match decode::<Claims>(token, &DecodingKey::from_secret(key), &validation) {
    //     Ok(c) => c,
    //     Err(err) => match *err.kind() {
    //         ErrorKind::InvalidToken => panic!("Token is invalid"), // Example on how to handle a specific error
    //         ErrorKind::InvalidIssuer => panic!("Issuer is invalid"), // Example on how to handle a specific error
    //         _ => panic!("Some other errors"),
    //     },
    // };

    // println!("{:?}" , token_data);

    match res {
        Ok(resp) => {
            let resp = resp.into_response();
            println!("response: {}", resp.status());
            Ok(resp)
        }
        Err(err) => {
            println!("error: {err}");
            Err(err)
        }
    }
}







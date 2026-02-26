use jsonwebtoken::{DecodingKey, Validation, decode, errors::ErrorKind};
use poem::{Endpoint, Error, IntoResponse, Request, Response, Result, http::{StatusCode} };
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    user_id: String,
}

#[derive(Clone)]
struct  UserId(String);


pub async fn log<E: Endpoint>(next: E, mut req: Request) -> Result<Response , Error> {
    println!("request: {}", req.uri().path());
    let token =  req.headers().get("token").ok_or_else(||  Error::from_status(StatusCode::UNAUTHORIZED))?; // If header exists → continue 
    println!("{:?}" , token);
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.validate_exp = false;
    validation.set_required_spec_claims(&["exp"]);
    let key = b"fdfad";
    let token_data = match decode::<Claims>(token, &DecodingKey::from_secret(key), &validation) {
        Ok(c) => c,
        Err(err) =>  match *err.kind()  {
            ErrorKind::InvalidToken => panic!("Token is invalid"), // Example on how to handle a specific error
            ErrorKind::InvalidIssuer => panic!("Issuer is invalid"), // Example on how to handle a specific error
            _ => panic!("Some other errors"),
        },

    };
    req.extensions_mut().insert(UserId(token_data.claims.user_id));

    let res = next.call(req).await;
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

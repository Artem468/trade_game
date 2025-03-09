use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::AppState;
use actix_web::error::InternalError;
use actix_web::{dev::Payload, Error, FromRequest, HttpRequest, HttpResponse};
use chrono::{Duration, Utc};
use futures::future::{ready, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation};
use jsonwebtoken::{encode, EncodingKey, Header, TokenData};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,         
    pub iat: usize,       
    pub exp: usize,       
    pub email: String,
    pub username: String,
    pub token_type: String,  // "access" или "refresh"
}


pub fn generate_access_token(user_id: i32, username: &str, email: &str, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = now + Duration::hours(3);
    let claims = Claims {
        sub: user_id,
        iat: now.timestamp() as usize,
        exp: exp.timestamp() as usize,
        username: username.to_owned(),
        email: email.to_owned(),
        token_type: "access".to_owned(),
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
}


pub fn generate_refresh_token(user_id: i32, username: &str, email: &str, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = now + Duration::days(30);
    let claims = Claims {
        sub: user_id,
        iat: now.timestamp() as usize,
        exp: exp.timestamp() as usize,
        username: username.to_owned(),
        email: email.to_owned(),
        token_type: "refresh".to_owned(),
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
}


#[derive(Debug)]
pub struct AccessToken(pub TokenData<Claims>);

impl FromRequest for AccessToken {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        fn unauthorized_response(message: String) -> Error {
            let response = HttpResponse::Unauthorized().json(CommonResponse::<()> {
                status: ResponseStatus::Error,
                data: (),
                error: Some(message.clone()),
            });
            InternalError::from_response(message, response).into()
        }

        let state = match req.app_data::<actix_web::web::Data<AppState>>() {
            Some(data) => data.get_ref().jwt_secret.clone(),
            None => return ready(Err(unauthorized_response("AppState not configured".to_string()))),
        };

        if let Some(auth_header) = req.headers().get("Authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                let parts: Vec<&str> = auth_str.split_whitespace().collect();
                if parts.len() == 2 && parts[0] == "Bearer" {
                    let token = parts[1];
                    return match decode::<Claims>(
                        token,
                        &DecodingKey::from_secret(state.as_ref()),
                        &Validation::default(),
                    ) {
                        Ok(token_data) => {
                            if token_data.claims.token_type == "access" {
                                ready(Ok(AccessToken(token_data)))
                            } else {
                                ready(Err(unauthorized_response("Invalid token type".to_string())))
                            }
                        }
                        Err(_) => {
                            ready(Err(unauthorized_response("Invalid access token".to_string())))
                        }
                    }
                }
            }
        }
        ready(Err(unauthorized_response("Missing token".to_string())))
    }
}
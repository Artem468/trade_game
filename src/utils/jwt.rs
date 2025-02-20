use chrono::{Utc, Duration};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,         // идентификатор пользователя
    pub iat: usize,          // время выпуска
    pub exp: usize,          // время истечения
    pub email: String,
    pub token_type: String,  // "access" или "refresh"
}


pub fn generate_access_token(user_id: i32, email: &str, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = now + Duration::hours(1);
    let claims = Claims {
        sub: user_id.to_string(),
        iat: now.timestamp() as usize,
        exp: exp.timestamp() as usize,
        email: email.to_owned(),
        token_type: "access".to_owned(),
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
}


pub fn generate_refresh_token(user_id: i32, email: &str, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = now + Duration::days(30);
    let claims = Claims {
        sub: user_id.to_string(),
        iat: now.timestamp() as usize,
        exp: exp.timestamp() as usize,
        email: email.to_owned(),
        token_type: "refresh".to_owned(),
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
}
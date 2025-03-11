use crate::utils::jwt::{generate_access_token, generate_refresh_token};
use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::{try_or_http_err, AppState};
use actix_web::{post, web, HttpResponse, Responder};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use entity::users;
use rand_core::OsRng;
use sea_orm::{ActiveModelTrait, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[utoipa::path(
    request_body = RegisterInput,
    tag="Authorization"
)]
#[post("/api/v1/auth/register")]
pub async fn register(
    state: web::Data<AppState>,
    input: web::Json<RegisterInput>,
) -> impl Responder {
    let input = input.into_inner();

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(input.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let new_user = users::ActiveModel {
        email: Set(input.email),
        hashed_password: Set(password_hash),
        username: Set(input.username),
        ..Default::default()
    };
    
    match new_user.insert(state.db.as_ref()).await {
        Ok(data) => {
            let access_token = try_or_http_err!(generate_access_token(data.id, data.username.as_str(), data.email.as_str(), state.jwt_secret.as_str()));
            let refresh_token = try_or_http_err!(generate_refresh_token(data.id, data.username.as_str(), data.email.as_str(), state.jwt_secret.as_str()));

            HttpResponse::Created().json(
                CommonResponse::<Option<RegisterResponse>> {
                    status: ResponseStatus::Ok,
                    data: Some(RegisterResponse {
                        access_token,
                        refresh_token,
                        user_id: data.id,
                        email: data.email,
                        username: data.username,
                    }),
                    error: None,
                }
            )
        }
        Err(err) => HttpResponse::InternalServerError().json(CommonResponse::<Option<RegisterResponse>> {
            status: ResponseStatus::Error,
            data: None,
            error: Some(err.to_string()),
        }),
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterInput {
    email: String,
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    access_token: String,
    refresh_token: String,
    user_id: i32,
    email: String,
    username: String,
}
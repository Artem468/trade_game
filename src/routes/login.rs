use crate::utils::jwt::{generate_access_token, generate_refresh_token};
use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::{try_or_http_err, AppState};
use actix_web::{post, web, HttpResponse, Responder};
use argon2::password_hash::PasswordHash;
use argon2::{Argon2, PasswordVerifier};
use entity::users;
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;


#[utoipa::path(
    request_body = LoginInput,
)]
#[post("/api/v1/auth/login")]
pub async fn login(state: web::Data<AppState>, input: web::Json<LoginInput>) -> impl Responder {
    let input = input.into_inner();

    let user = try_or_http_err!(
        users::Entity::find()
            .filter(
                Condition::any()
                    .add(users::Column::Email.eq(&input.username))
                    .add(users::Column::Username.eq(&input.username))
            )
            .one(state.db.as_ref())
            .await
    );

    if let Some(user) = user {
        let stored_hash = user.hashed_password;
        let parsed_hash = PasswordHash::new(&stored_hash).expect("Failed to parse hash");
        if Argon2::default()
            .verify_password(input.password.as_bytes(), &parsed_hash)
            .is_ok()
        {
            let access_token = try_or_http_err!(generate_access_token(
                user.id,
                user.email.as_str(),
                state.jwt_secret.as_str()
            ));
            let refresh_token = try_or_http_err!(generate_refresh_token(
                user.id,
                user.email.as_str(),
                state.jwt_secret.as_str()
            ));

            return HttpResponse::Ok().json(CommonResponse::<Option<LoginResponse>> {
                status: ResponseStatus::Ok,
                data: Some(LoginResponse {
                    access_token,
                    refresh_token,
                    email: user.email,
                    username: user.username,
                }),
                error: None,
            });
        }
    }
    HttpResponse::Unauthorized().json(CommonResponse::<Option<LoginResponse>> {
        status: ResponseStatus::Error,
        data: None,
        error: Some("Unauthorized user".into()),
    })
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginInput {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    access_token: String,
    refresh_token: String,
    email: String,
    username: String,
}

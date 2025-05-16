use crate::unwrap_or_http_err_with_opt_msg;
use crate::utils::jwt::AccessToken;
use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::{extract_db_response_or_http_err_with_opt_msg, try_or_http_err, AppState};
use actix_web::{post, web, HttpResponse, Responder};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use entity::users;
use rand_core::OsRng;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, Set};
use serde::Deserialize;
use utoipa::ToSchema;

#[utoipa::path(
    request_body = ChangePasswordInput,
    tag="Authorization",
    security(
        ("bearer_token" = [])
    )
)]
#[post("/api/v1/change/password")]
pub async fn change_password(
    state: web::Data<AppState>,
    input: web::Json<ChangePasswordInput>,
    token: AccessToken,
) -> impl Responder {
    let input = input.into_inner();
    let user_id = token.claims.sub;
    
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(input.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let user = extract_db_response_or_http_err_with_opt_msg!(
        users::Entity::find_by_id(user_id)
            .one(state.db.as_ref())
            .await,
        "User not found"
    );
    let mut active_user: users::ActiveModel = user.into_active_model();
    active_user.hashed_password = Set(password_hash);
    try_or_http_err!(active_user.update(state.db.as_ref()).await);
    
    HttpResponse::Ok().json(CommonResponse::<()> {
        status: ResponseStatus::Ok,
        data: (),
        error: None,
    })
}


#[derive(Debug, Deserialize, ToSchema)]
pub struct ChangePasswordInput {
    password: String,
}
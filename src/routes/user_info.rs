use crate::utils::jwt::AccessToken;
use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::{try_or_http_err, AppState};
use actix_web::{get, web, HttpResponse, Responder};
use entity::users;
use sea_orm::prelude::{DateTime, Decimal};
use sea_orm::EntityTrait;
use serde::Serialize;

#[utoipa::path(
    tag = "User",
    security(
        ("bearer_token" = [])
    )
)]
#[get("/api/v1/users/info")]
pub async fn user_info(
    state: web::Data<AppState>,
    token: AccessToken
) -> impl Responder {
    match try_or_http_err!(
        users::Entity::find_by_id(token.0.claims.sub)
            .one(state.db.as_ref())
            .await
    ) {
        Some(user) => HttpResponse::Ok().json(CommonResponse::<UserResponse> {
            status: ResponseStatus::Ok,
            data: UserResponse {
                id: user.id,
                username: user.username,
                email: user.email,
                balance: user.balance,
                created_at: user.created_at,
            },
            error: None,
        }),
        None => HttpResponse::InternalServerError().json(CommonResponse::<()> {
            status: ResponseStatus::Error,
            data: (),
            error: Some("No user".into()),
        }),
    }
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub balance: Decimal,
    pub created_at: DateTime,
}
use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::{try_or_http_err, AppState};
use actix_web::web::Query;
use actix_web::{get, web, HttpResponse, Responder};
use entity::users;
use sea_orm::prelude::{DateTime, Decimal};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[utoipa::path(params(UserIdQuery), tag = "User")]
#[get("/api/v1/users/info")]
pub async fn user_info(state: web::Data<AppState>, query: Query<UserIdQuery>) -> impl Responder {
    match try_or_http_err!(
        users::Entity::find_by_id(query.user_id)
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

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct UserIdQuery {
    pub user_id: i32,
}

use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::{try_or_http_err, AppState};
use actix_web::{get, web, HttpResponse, Responder};
use entity::users;
use sea_orm::{
    ColumnTrait, EntityTrait, FromQueryResult, QueryFilter, QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[utoipa::path(params(TopUsersQuery), tag = "User")]
#[get("/api/v1/users/top")]
pub async fn top_users(
    state: web::Data<AppState>,
    query: web::Query<TopUsersQuery>,
) -> impl Responder {
    let data = try_or_http_err!(
        users::Entity::find()
            .filter(users::Column::IsBot.eq(false))
            .columns([users::Column::Id, users::Column::Username])
            .limit(query.limit)
            .order_by_desc(users::Column::Balance)
            .into_model::<TopUsers>()
            .all(state.db.as_ref())
            .await
    );

    HttpResponse::Ok().json(CommonResponse::<Vec<TopUsers>> {
        status: ResponseStatus::Ok,
        data,
        error: None,
    })
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct TopUsersQuery {
    pub limit: u64,
}

#[derive(Serialize, FromQueryResult)]
pub struct TopUsers {
    pub id: i32,
    pub username: String,
}

use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::{try_or_http_err, AppState};
use actix_web::{get, web, HttpResponse, Responder};
use entity::users;
use sea_orm::prelude::{DateTime, Decimal};
use sea_orm::QueryFilter;
use sea_orm::{ColumnTrait, QuerySelect};
use sea_orm::{EntityTrait, FromQueryResult};
use serde::Serialize;

#[utoipa::path(tag = "Market")]
#[get("/api/v1/bots")]
pub async fn get_bots(state: web::Data<AppState>) -> impl Responder {
    let bots_data = try_or_http_err!(
        users::Entity::find()
            .filter(users::Column::IsBot.eq(true))
            .column(users::Column::Id)
            .column(users::Column::Username)
            .column(users::Column::Email)
            .column(users::Column::Balance)
            .column(users::Column::CreatedAt)
            .into_model::<BotsResponse>()
            .all(state.db.as_ref())
            .await
    );

    HttpResponse::Ok().json(CommonResponse::<Vec<BotsResponse>> {
        status: ResponseStatus::Ok,
        data: bots_data,
        error: None,
    })
}

#[derive(Serialize, FromQueryResult)]
pub struct BotsResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub balance: Decimal,
    pub created_at: DateTime,
}

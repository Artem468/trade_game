use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::{try_or_http_err, AppState};
use actix_web::{get, web, HttpResponse, Responder};
use entity::events;
use sea_orm::prelude::DateTime;
use sea_orm::{EntityTrait, FromQueryResult, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[utoipa::path(params(EventsQuery), tag = "Market")]
#[get("/api/v1/events")]
pub async fn get_events(
    state: web::Data<AppState>,
    query: web::Query<EventsQuery>,
) -> impl Responder {
    let events_data = try_or_http_err!(
        events::Entity::find()
            .order_by_desc(events::Column::CreatedAt)
            .limit(query.limit)
            .offset(query.offset)
            .into_model::<EventsResponse>()
            .all(state.db.as_ref())
            .await
    );

    HttpResponse::Ok().json(CommonResponse::<Vec<EventsResponse>> {
        status: ResponseStatus::Ok,
        data: events_data,
        error: None,
    })
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct EventsQuery {
    pub limit: u64,
    pub offset: Option<u64>,
}

#[derive(Serialize, FromQueryResult)]
pub struct EventsResponse {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub created_at: DateTime,
}

use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::{try_or_http_err, AppState};
use actix_web::{post, web, HttpResponse, Responder};
use entity::events;
use entity::prelude::Events;
use sea_orm::{EntityTrait, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[utoipa::path(
    request_body = EventInput,
    tag="Market"
)]
#[post("/api/v1/events/create")]
pub async fn create_event(
    state: web::Data<AppState>,
    input: web::Json<EventInput>,
) -> impl Responder {
    let input = input.into_inner();

    let event = events::ActiveModel {
        title: Set(input.title),
        description: Set(input.description),
        ..Default::default()
    };
    let event_id = try_or_http_err!(Events::insert(event).exec(state.db.as_ref()).await);

    HttpResponse::Ok().json(CommonResponse::<EventResponse> {
        status: ResponseStatus::Ok,
        data: EventResponse {
            event_id: event_id.last_insert_id,
        },
        error: None,
    })
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct EventInput {
    title: String,
    description: String,
}

#[derive(Serialize)]
pub struct EventResponse {
    event_id: i32,
}

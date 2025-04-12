use crate::utils::jwt::AccessToken;
use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::{try_or_http_err, AppState};
use actix_web::{get, web, HttpResponse, Responder};
use entity::{messages, users};
use sea_orm::{ColumnTrait, Condition, FromQueryResult, QueryFilter};
use sea_orm::{EntityTrait, QuerySelect};
use serde::Serialize;

#[utoipa::path(
    tag = "User",
    security(
        ("bearer_token" = [])
    )
)]
#[get("/api/v1/chats/list")]
pub async fn get_chats(state: web::Data<AppState>, token: AccessToken) -> impl Responder {
    let user_id = token.claims.sub;

    let users_list = try_or_http_err!(
        messages::Entity::find()
            .distinct()
            .select_only()
            .column(messages::Column::FromId)
            .column(messages::Column::RecipientId)
            .filter(
                Condition::any()
                    .add(messages::Column::FromId.eq(user_id))
                    .add(messages::Column::RecipientId.eq(user_id))
            )
            .into_tuple::<(i32, i32)>()
            .all(state.db.as_ref())
            .await
    );

    let unique_ids: Vec<i32> = users_list
        .into_iter()
        .flat_map(|(from_id, recipient_id)| vec![from_id, recipient_id])
        .filter(|&id| id != user_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let data = try_or_http_err!(
        users::Entity::find()
            .filter(users::Column::Id.is_in(unique_ids))
            .select_only()
            .columns([users::Column::Id, users::Column::Username])
            .into_model::<ChatsResponse>()
            .all(state.db.as_ref())
            .await
    );

    HttpResponse::Ok().json(CommonResponse::<Vec<ChatsResponse>> {
        status: ResponseStatus::Ok,
        data,
        error: None,
    })
}

#[derive(Serialize, FromQueryResult)]
pub struct ChatsResponse {
    pub id: i32,
    pub username: String,
}

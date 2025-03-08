use crate::utils::jwt::AccessToken;
use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::{try_or_http_err, AppState};
use actix_web::{get, web, HttpResponse, Responder};
use entity::{messages, users};
use sea_orm::prelude::DateTime;
use sea_orm::{ColumnTrait, Condition, EntityTrait, FromQueryResult, QuerySelect};
use sea_orm::{QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[utoipa::path(params(HistoryQuery, HistoryParams))]
#[get("/api/v1/chat/getHistory/{user_id}")]
pub async fn chat_history(
    state: web::Data<AppState>,
    token: AccessToken,
    query: web::Path<HistoryQuery>,
    params: web::Query<HistoryParams>,
) -> impl Responder {
    let user_data = try_or_http_err!(
        users::Entity::find_by_id(token.0.claims.sub)
            .one(state.db.as_ref())
            .await
    );
    if let Some(user) = user_data {
        let data = try_or_http_err!(
            messages::Entity::find()
                .filter(
                    Condition::any()
                        .add(
                            Condition::all()
                                .add(messages::Column::FromId.eq(user.id))
                                .add(messages::Column::RecipientId.eq(query.user_id))
                        )
                        .add(
                            Condition::all()
                                .add(messages::Column::RecipientId.eq(user.id))
                                .add(messages::Column::FromId.eq(query.user_id))
                        ),
                )
                .column_as(messages::Column::Id, "message_id")
                .order_by_desc(messages::Column::CreatedAt)
                .limit(params.limit)
                .into_model::<ChatMsg>()
                .all(state.db.as_ref())
                .await
        );

        return HttpResponse::Ok().json(CommonResponse::<Vec<ChatMsg>> {
            status: ResponseStatus::Ok,
            data,
            error: None,
        });
    }

    HttpResponse::Ok().json(CommonResponse::<()> {
        status: ResponseStatus::Error,
        data: (),
        error: Some("No user".into()),
    })
}

#[derive(FromQueryResult, Serialize)]
pub struct ChatMsg {
    message_id: i32,
    from_id: i32,
    recipient_id: i32,
    text: String,
    created_at: DateTime,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct HistoryParams {
    pub limit: u64,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct HistoryQuery {
    pub user_id: i32,
}

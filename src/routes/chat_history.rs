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

#[utoipa::path(
    params(HistoryQuery, HistoryParams),
    tag="User",
    security(
        ("bearer_token" = [])
    )
)]
#[get("/api/v1/chat/history/{user_id}")]
pub async fn chat_history(
    state: web::Data<AppState>,
    token: AccessToken,
    params: web::Path<HistoryParams>,
    query: web::Query<HistoryQuery>,
) -> impl Responder {
    let user_data = try_or_http_err!(
        users::Entity::find_by_id(token.claims.sub)
            .one(state.db.as_ref())
            .await
    );
    if let Some(user) = user_data {
        let data = try_or_http_err!(
            messages::Entity::find()
                .filter(
                    Condition::any()
                        .add({
                            let cond = Condition::all()
                                .add(messages::Column::FromId.eq(user.id))
                                .add(messages::Column::RecipientId.eq(params.user_id));

                            if let Some(before) = query.before_message_id {
                                cond.add(messages::Column::Id.lt(before))
                            } else {
                                cond
                            }
                        })
                        .add({
                            let cond = Condition::all()
                                .add(messages::Column::RecipientId.eq(user.id))
                                .add(messages::Column::FromId.eq(params.user_id));
                    
                            if let Some(before) = query.before_message_id {
                                cond.add(messages::Column::Id.lt(before))
                            } else {
                                cond
                            }
                        })
                )
                .column_as(messages::Column::Id, "message_id")
                .order_by_asc(messages::Column::CreatedAt)
                .limit(query.limit)
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
pub struct HistoryQuery {
    pub limit: u64,
    pub before_message_id: Option<i32>,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct HistoryParams {
    pub user_id: i32,
}

use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::{try_or_http_err, AppState};
use actix_web::{web, HttpResponse, Responder};
use entity::{assets, trades, users};
use sea_orm::{FromQueryResult, JoinType, QueryFilter, QuerySelect, RelationTrait};
use sea_orm::{ColumnTrait, EntityTrait};
use sea_orm::prelude::{DateTime, Decimal};
use serde::{Deserialize, Serialize};

pub async fn trades_history(
    state: web::Data<AppState>,
    query: web::Query<TradeHistoryQuery>,
) -> impl Responder {
    let user_data = try_or_http_err!(users::Entity::find()
        .filter(users::Column::Username.eq(query.username.clone()))
        .one(state.db.as_ref())
        .await);

    if let Some(user) = user_data {
        let trade_history = trades::Entity::find()
            .filter(trades::Column::UserId.eq(user.id))
            .join(JoinType::InnerJoin, trades::Relation::Assets.def())
            .select_only()
            .column(trades::Column::Id)
            .column(trades::Column::TradeType)
            .column(trades::Column::Price)
            .column(trades::Column::Amount)
            .column(trades::Column::CreatedAt)
            .column(assets::Column::Name)
            .into_model::<TradeHistoryResponse>()
            .all(state.db.as_ref())
            .await;

        return match trade_history {
            Ok(trades) => 
                HttpResponse::Ok().json(CommonResponse::<Vec<TradeHistoryResponse>> {
                    status: ResponseStatus::Ok,
                    data: trades,
                    error: None,
                }),
            Err(_) => HttpResponse::Ok().json(CommonResponse::<()> {
                status: ResponseStatus::Error,
                data: (),
                error: Some("Failed to fetch trade history".into()),
            }),
        }
    }
    
    HttpResponse::Ok().json(CommonResponse::<()> {
        status: ResponseStatus::Error,
        data: (),
        error: Some("No user".into()),
    })
}

#[derive(Debug, FromQueryResult, Serialize)]
struct TradeHistoryResponse {
    id: i32,
    trade_type: String,
    name: String,
    price: Decimal,
    amount: Decimal,
    created_at: DateTime,
}

#[derive(Deserialize)]
pub struct TradeHistoryQuery {
    pub username: String,
}
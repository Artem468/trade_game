use crate::traits::redis::PriceInfo;
use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::{try_or_http_err, AppState};
use actix_web::{get, web, HttpResponse, Responder};
use entity::users;
use redis::AsyncCommands;
use sea_orm::prelude::{Decimal, Expr};
use sea_orm::{EntityTrait, FromQueryResult, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[utoipa::path(params(TopUsersQuery), tag = "User")]
#[get("/api/v1/users/top")]
pub async fn top_users(
    state: web::Data<AppState>,
    query: web::Query<TopUsersQuery>,
) -> impl Responder {
    let mut redis_conn = match state.cache.get_multiplexed_async_connection().await {
        Ok(conn) => conn,
        Err(err) => {
            return HttpResponse::InternalServerError().json(CommonResponse::<()> {
                status: ResponseStatus::Error,
                data: (),
                error: Some(err.to_string()),
            })
        }
    };

    let assets_keys: Vec<String> = try_or_http_err!(redis_conn.keys("asset_price:*").await);
    let mut asset_prices: Vec<(&str, PriceInfo)> = Vec::new();
    let assets_keys_clone = assets_keys.clone();
    for (i, key) in assets_keys.iter().enumerate() {
        asset_prices.push((
            assets_keys_clone[i]
                .strip_prefix("asset_price:")
                .unwrap_or_default(),
            try_or_http_err!(redis_conn.hgetall::<String, PriceInfo>(key.clone()).await)));
    }

    if asset_prices.is_empty() {
        return HttpResponse::InternalServerError().json(CommonResponse::<()> {
            status: ResponseStatus::Error,
            data: (),
            error: Some("No asset prices found in Redis".to_string()),
        });
    }

    let mut cases = String::new();
    for (asset_id, price) in &asset_prices {
        cases.push_str(&format!("WHEN user_balances.asset_id = {} THEN {} ", asset_id, price.price.unwrap_or_default()));
    }

    let query_string = format!(
        "ROUND(users.balance + COALESCE((SELECT SUM(user_balances.amount * (CASE {} ELSE 0 END)) FROM user_balances WHERE user_balances.user_id = users.id), 0), 3)",
        cases
    );

    let data = try_or_http_err!(users::Entity::find()
        .column(users::Column::Id)
        .column(users::Column::Username)
        .column_as(
            Expr::cust(&query_string),
            "total_balance",
        )
        .order_by_desc(Expr::cust("total_balance"))
        .limit(query.limit)
        .into_model::<TopUsers>()
        .all(state.db.as_ref())
        .await);

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
    pub total_balance: Decimal,
}

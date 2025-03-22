use crate::traits::redis::PriceInfo;
use crate::unwrap_or_http_err_with_opt_msg;
use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::{extract_db_response_or_http_err_with_opt_msg, try_or_http_err, AppState};
use actix_web::{get, web, HttpResponse, Responder};
use redis::AsyncCommands;
use sea_orm::prelude::Decimal;
use sea_orm::{DatabaseBackend, FromQueryResult, Statement};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[utoipa::path(params(UserPlaceQuery), tag = "User")]
#[get("/api/v1/users/place")]
pub async fn get_user_place(
    state: web::Data<AppState>,
    query: web::Query<UserPlaceQuery>,
) -> impl Responder {
    let mut redis_conn = try_or_http_err!(state.cache.get_multiplexed_async_connection().await);

    let assets_keys: Vec<String> = try_or_http_err!(redis_conn.keys("asset_price:*").await);

    if assets_keys.is_empty() {
        return HttpResponse::InternalServerError().json(CommonResponse::<()> {
            status: ResponseStatus::Error,
            data: (),
            error: Some("No asset prices found in Redis".to_string()),
        });
    }

    let mut cases = String::new();
    for key in &assets_keys {
        if let Some(asset_id) = key.strip_prefix("asset_price:") {
            if let Ok(price_info) = redis_conn.hgetall::<String, PriceInfo>(key.clone()).await {
                let price = price_info.price.unwrap_or_default();
                cases.push_str(&format!(
                    "WHEN user_balances.asset_id = '{}' THEN {} ",
                    asset_id, price
                ));
            }
        }
    }

    let case_expr = format!("CASE {} ELSE 0 END", cases);
    let query_string = format!(
        "ROUND(users.balance + COALESCE((
            SELECT SUM(user_balances.amount * {})
            FROM user_balances
            WHERE user_balances.user_id = users.id
        ), 0), 3)",
        case_expr
    );

    let rank_query = format!("RANK() OVER (ORDER BY {} DESC)", query_string);
    
    let sql = format!(
        "
        SELECT sub.total_balance, sub.place
        FROM (
            SELECT
                users.id,
                {query_string} AS total_balance,
                {rank_query} AS place
            FROM users
            WHERE users.is_bot = FALSE
        ) sub
        WHERE sub.id = $1
        "
    );
    
    let data = extract_db_response_or_http_err_with_opt_msg!(
        UserPlace::find_by_statement(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            &sql,
            [query.user_id.into()]
        ))
        .one(state.db.as_ref())
        .await,
        "User not found"
    );

    HttpResponse::Ok().json(CommonResponse::<UserPlace> {
        status: ResponseStatus::Ok,
        data,
        error: None,
    })
    
}


#[derive(Deserialize, ToSchema, IntoParams)]
pub struct UserPlaceQuery {
    pub user_id: i32,
}

#[derive(Serialize, FromQueryResult)]
pub struct UserPlace {
    pub place: i64,
    pub total_balance: Decimal,
}

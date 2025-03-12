use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::{try_or_http_err, AppState};
use actix_web::{get, web, HttpResponse, Responder};
use chrono::Utc;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[utoipa::path(params(PricePath), tag = "Market")]
#[get("/api/v1/price/history/{asset_id}")]
pub async fn price_history(
    state: web::Data<AppState>,
    path: web::Path<PricePath>,
) -> impl Responder {
    let day_ago = Utc::now().timestamp() - 86400;
    let mut redis_conn = try_or_http_err!(state.cache.get_multiplexed_async_connection().await);
    let history: Vec<(String, i64)> = try_or_http_err!(
        redis_conn
            .zrangebyscore_withscores(
                format!("asset_price_history:{}", path.asset_id),
                day_ago,
                "+inf",
            )
            .await
    );

    HttpResponse::Ok().json(CommonResponse::<Vec<PriceResponse>> {
        status: ResponseStatus::Ok,
        data: history
            .iter()
            .filter_map(|(item, _)| {
                if let Some((price, timestamp)) = item.split_once(":") {
                    Some(PriceResponse {
                        price: price.into(),
                        timestamp: timestamp.into(),
                    })
                } else {
                    None
                }
            })
            .collect(),
        error: None,
    })
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct PricePath {
    pub asset_id: i32,
}

#[derive(Serialize)]
pub struct PriceResponse {
    price: String,
    timestamp: String,
}

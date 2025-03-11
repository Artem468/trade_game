use crate::utils::jwt::AccessToken;
use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::{try_or_http_err, AppState};
use actix_web::{post, web, HttpResponse, Responder};
use entity::{user_balances, users};
use redis::AsyncCommands;
use sea_orm::prelude::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use utoipa::ToSchema;

#[utoipa::path(
    request_body = BuyMarketRequest,
    security(
        ("bearer_token" = [])
    )
)]
#[post("/api/v1/market/buy")]
pub async fn market_buy(
    state: web::Data<AppState>,
    input: web::Json<BuyMarketRequest>,
    token: AccessToken,
) -> impl Responder {
    let user_id = token.0.claims.sub;

    let mut redis_conn = try_or_http_err!(state.cache.get_multiplexed_async_connection().await);

    let price_key = format!("asset_price:{}", input.asset_id);
    let price_str: Option<String> = try_or_http_err!(redis_conn.hget(&price_key, "price").await);

    let current_price = if let Some(price_str) = price_str {
        try_or_http_err!(Decimal::from_str(&price_str))
    } else {
        return HttpResponse::BadRequest().json(CommonResponse::<()> {
            status: ResponseStatus::Error,
            data: (),
            error: Some("Can't get price".into()),
        });
    };

    let amount_to_buy = Decimal::from_f64_retain(input.amount).unwrap_or_default();

    let total_cost = current_price * amount_to_buy;

    let user_asset: user_balances::Model = if let Some(_data) = try_or_http_err!(
        user_balances::Entity::find()
            .filter(user_balances::Column::UserId.eq(user_id))
            .filter(user_balances::Column::AssetId.eq(input.asset_id))
            .one(state.db.as_ref())
            .await
    ) {
        _data
    } else {
        user_balances::Model {
            id: 0,
            user_id,
            asset_id: input.asset_id,
            amount: Decimal::from(0),
        }
    };

    let user: users::Model = match try_or_http_err!(
        users::Entity::find_by_id(user_id)
            .one(state.db.as_ref())
            .await
    ) {
        Some(us) => us,
        None => {
            return HttpResponse::Ok().json(CommonResponse::<()> {
                status: ResponseStatus::Error,
                data: (),
                error: Some("Can't find user".into()),
            })
        }
    };

    if user.balance < total_cost {
        return HttpResponse::Ok().json(CommonResponse::<()> {
            status: ResponseStatus::Error,
            data: (),
            error: Some("You don't have enough money".into()),
        });
    }
    let _balance = user.balance;
    let _user_amount = user_asset.amount;
    
    let mut active_user: users::ActiveModel = user.into_active_model();
    active_user.balance = Set((_balance - total_cost).round_dp(3));
    try_or_http_err!(active_user.update(state.db.as_ref()).await);

    let mut active_user_asset = user_asset.into_active_model();
    active_user_asset.amount = Set((_user_amount + amount_to_buy).round_dp(3));

    if let Err(_) = active_user_asset.update(state.db.as_ref()).await {
        let _asset = user_balances::ActiveModel {
            user_id: Set(user_id),
            asset_id: Set(input.asset_id),
            amount: Set(amount_to_buy.round_dp(3)),
            ..Default::default()
        };

        try_or_http_err!(_asset.insert(state.db.as_ref()).await);
    }
    

    HttpResponse::Ok().json(CommonResponse::<BuyMarketResponse> {
        status: ResponseStatus::Ok,
        data: BuyMarketResponse {},
        error: None,
    })
}

#[derive(Deserialize, ToSchema)]
pub struct BuyMarketRequest {
    pub asset_id: i32,
    pub amount: f64,
}

#[derive(Serialize)]
pub struct BuyMarketResponse {}

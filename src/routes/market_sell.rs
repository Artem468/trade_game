use crate::utils::get_price::get_price_by_asset_id;
use crate::utils::jwt::AccessToken;
use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::utils::take_commission::{take_commission};
use crate::{try_or_http_err, AppState, COMMISSION_MARKET_SELL};
use actix_web::{post, web, HttpResponse, Responder};
use entity::{trades, user_balances, users};
use sea_orm::prelude::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[utoipa::path(
    request_body = SellMarketRequest,
    tag="User",
    security(
        ("bearer_token" = [])
    )
)]
#[post("/api/v1/market/sell")]
pub async fn market_sell(
    state: web::Data<AppState>,
    input: web::Json<SellMarketRequest>,
    token: AccessToken,
) -> impl Responder {
    let user_id = token.0.claims.sub;
    let current_price = try_or_http_err!(get_price_by_asset_id(&state.cache, input.asset_id).await);
    let amount_to_sell = Decimal::from_f64_retain(input.amount).unwrap_or_default();
    let total_cost = current_price * amount_to_sell;

    let user_asset: user_balances::Model = if let Some(_data) = try_or_http_err!(
        user_balances::Entity::find()
            .filter(user_balances::Column::UserId.eq(user_id))
            .filter(user_balances::Column::AssetId.eq(input.asset_id))
            .one(state.db.as_ref())
            .await
    ) {
        _data
    } else {
        return HttpResponse::BadRequest().json(CommonResponse::<()> {
            status: ResponseStatus::Error,
            data: (),
            error: Some("Can't sell asset".into()),
        });
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

    if user_asset.amount < try_or_http_err!(Decimal::try_from(input.amount)) {
        return HttpResponse::Ok().json(CommonResponse::<()> {
            status: ResponseStatus::Error,
            data: (),
            error: Some("You don't have enough asset".into()),
        });
    }
    let _balance = user.balance;
    let _user_amount = user_asset.amount;

    let mut active_user: users::ActiveModel = user.into_active_model();
    let amount_commission = take_commission(total_cost, COMMISSION_MARKET_SELL.clone());
    active_user.balance = Set((_balance + amount_commission.amount).round_dp(3));
    try_or_http_err!(active_user.update(state.db.as_ref()).await);

    let mut active_user_asset = user_asset.into_active_model();
    let amount_data = (_user_amount - amount_to_sell).round_dp(3);
    active_user_asset.amount = Set(amount_data);

    try_or_http_err!(active_user_asset.update(state.db.as_ref()).await);
    
    let trade = trades::ActiveModel {
        user_id: Set(user_id),
        asset_id: Set(input.asset_id),
        trade_type: Set("sell".into()),
        price: Set(current_price),
        amount: Set(amount_data),
        ..Default::default()
    };
    try_or_http_err!(trade.insert(state.db.as_ref()).await);

    HttpResponse::Ok().json(CommonResponse::<BuyMarketResponse> {
        status: ResponseStatus::Ok,
        data: BuyMarketResponse {
            amount: amount_commission.amount,
            commission: amount_commission.commission,
        },
        error: None,
    })
}

#[derive(Deserialize, ToSchema)]
pub struct SellMarketRequest {
    pub asset_id: i32,
    pub amount: f64,
}

#[derive(Serialize)]
pub struct BuyMarketResponse {
    amount: Decimal,
    commission: Decimal
}

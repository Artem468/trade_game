use crate::utils::jwt::AccessToken;
use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::utils::take_commission::take_commission;
use crate::{
    extract_db_response_or_http_err_with_opt_msg, unwrap_or_http_err_with_opt_msg,
    COMMISSION_ORDER_SELL,
};
use crate::{try_or_http_err, AppState};
use actix_web::{post, web, HttpResponse, Responder};
use chrono::Utc;
use entity::{orders, trades, user_balances, users};
use sea_orm::prelude::Decimal;
use sea_orm::ColumnTrait;
use sea_orm::{ActiveModelTrait, QueryFilter, Set};
use sea_orm::{Condition, EntityTrait, IntoActiveModel};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[utoipa::path(
    request_body = OrderSellInput,
    tag="User",
    security(
        ("bearer_token" = [])
    )
)]
#[post("/api/v1/order/sell")]
pub async fn order_sell(
    state: web::Data<AppState>,
    input: web::Json<OrderSellInput>,
    token: AccessToken,
) -> impl Responder {
    let input = input.into_inner();
    let order = extract_db_response_or_http_err_with_opt_msg!(
        orders::Entity::find_by_id(input.order_id)
            .one(state.db.as_ref())
            .await,
        "No order with this ID"
    );

    if order.user_id == token.claims.sub {
        return HttpResponse::BadRequest().json(CommonResponse::<()> {
            status: ResponseStatus::Error,
            data: (),
            error: Some("Can't execute this order".into()),
        });
    }

    if order.order_type != "sell" {
        return HttpResponse::BadRequest().json(CommonResponse::<()> {
            status: ResponseStatus::Error,
            data: (),
            error: Some("Wrong order type".into()),
        });
    }

    if order.status != "pending" {
        return HttpResponse::BadRequest().json(CommonResponse::<()> {
            status: ResponseStatus::Error,
            data: (),
            error: Some("Can't execute order".into()),
        });
    }

    let buyer = extract_db_response_or_http_err_with_opt_msg!(
        users::Entity::find_by_id(token.claims.sub)
            .one(state.db.as_ref())
            .await,
        "Buyer is not exist"
    );

    if buyer.balance < order.price {
        return HttpResponse::BadRequest().json(CommonResponse::<()> {
            status: ResponseStatus::Error,
            data: (),
            error: Some("Not enough money".into()),
        });
    }

    let seller = extract_db_response_or_http_err_with_opt_msg!(
        users::Entity::find_by_id(order.user_id)
            .one(state.db.as_ref())
            .await,
        "Seller is not exist"
    );

    let new_balance = buyer.balance - order.price;
    let mut active_buyer = buyer.into_active_model();
    active_buyer.balance = Set(new_balance);
    try_or_http_err!(active_buyer.update(state.db.as_ref()).await);

    let new_balance = (seller.balance + take_commission(order.price, COMMISSION_ORDER_SELL.clone()).amount).round_dp(3);
    let mut active_seller = seller.into_active_model();
    active_seller.balance = Set(new_balance);
    try_or_http_err!(active_seller.update(state.db.as_ref()).await);

    let buyer_balance = match try_or_http_err!(
        user_balances::Entity::find()
            .filter(
                Condition::all()
                    .add(user_balances::Column::UserId.eq(token.claims.sub))
                    .add(user_balances::Column::AssetId.eq(order.asset_id))
            )
            .one(state.db.as_ref())
            .await
    ) {
        Some(_data) => {_data}
        None => {
            user_balances::Model {
                id: 0,
                user_id: token.claims.sub,
                asset_id: order.asset_id,
                amount: Decimal::from(0),
            }
        }
    };

    let _user_amount = buyer_balance.amount;
    let mut active_buyer_balance = buyer_balance.into_active_model();
    active_buyer_balance.amount = Set((order.amount + _user_amount).round_dp(3));
    if let Err(_) = active_buyer_balance.update(state.db.as_ref()).await {
        let _asset = user_balances::ActiveModel {
            user_id: Set(token.claims.sub),
            asset_id: Set(order.asset_id),
            amount: Set((order.amount + _user_amount).round_dp(3)),
            ..Default::default()
        };

        try_or_http_err!(_asset.insert(state.db.as_ref()).await);
    }

    let _ = trades::ActiveModel {
        user_id: Set(token.claims.sub),
        asset_id: Set(order.asset_id),
        trade_type: Set("buy".into()),
        price: Set(order.price),
        amount: Set(order.amount),
        ..Default::default()
    }.insert(state.db.as_ref()).await;

    let _ = trades::ActiveModel {
        user_id: Set(order.user_id),
        asset_id: Set(order.asset_id),
        trade_type: Set("sell".into()),
        price: Set(order.price),
        amount: Set(order.amount),
        ..Default::default()
    }.insert(state.db.as_ref()).await;
    
    let mut active_order = order.into_active_model();
    active_order.status = Set("done".into());
    active_order.updated_at = Set(Utc::now().naive_utc());
    try_or_http_err!(active_order.update(state.db.as_ref()).await);
    
    HttpResponse::Ok().json(CommonResponse::<SellOrderResponse> {
        status: ResponseStatus::Ok,
        data: SellOrderResponse {
            balance: new_balance
        },
        error: None,
    })
}

#[derive(Deserialize, ToSchema)]
pub struct OrderSellInput {
    order_id: i32,
}


#[derive(Serialize)]
pub struct SellOrderResponse {
    balance: Decimal
}

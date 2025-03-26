use crate::utils::jwt::AccessToken;
use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::utils::take_commission::take_commission;
use crate::{
    extract_db_response_or_http_err_with_opt_msg, try_or_http_err, AppState,
};
use crate::{unwrap_or_http_err_with_opt_msg, COMMISSION_ORDER_BUY};
use actix_web::{post, web, HttpResponse, Responder};
use chrono::Utc;
use entity::{orders, trades, user_balances, users};
use sea_orm::prelude::Decimal;
use sea_orm::QueryFilter;
use sea_orm::{ActiveModelTrait, ColumnTrait, IntoActiveModel};
use sea_orm::{Condition, EntityTrait, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[utoipa::path(
    request_body = OrderBuyInput,
    tag="User",
    security(
        ("bearer_token" = [])
    )
)]
#[post("/api/v1/order/buy")]
pub async fn order_buy(
    state: web::Data<AppState>,
    input: web::Json<OrderBuyInput>,
    token: AccessToken,
) -> impl Responder {
    let input = input.into_inner();
    let order = extract_db_response_or_http_err_with_opt_msg!(
        orders::Entity::find_by_id(input.order_id)
            .one(state.db.as_ref())
            .await,
        "No order with this ID"
    );

    if order.user_id == token.0.claims.sub {
        return HttpResponse::BadRequest().json(CommonResponse::<()> {
            status: ResponseStatus::Error,
            data: (),
            error: Some("Can't execute this order".into()),
        });
    }

    if order.order_type != "buy" {
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

    let seller_balance = extract_db_response_or_http_err_with_opt_msg!(
        user_balances::Entity::find()
            .filter(
                Condition::all()
                    .add(user_balances::Column::UserId.eq(token.0.claims.sub))
                    .add(user_balances::Column::AssetId.eq(order.asset_id))
            )
            .one(state.db.as_ref())
            .await,
        "No asset for this user"
    );
    
    if seller_balance.amount < order.amount {
        return HttpResponse::BadRequest().json(CommonResponse::<()> {
            status: ResponseStatus::Error,
            data: (),
            error: Some("Not enough asset amount".into()),
        });
    }

    let seller = extract_db_response_or_http_err_with_opt_msg!(
        users::Entity::find_by_id(token.0.claims.sub)
            .one(state.db.as_ref())
            .await,
        "Seller is not exist"
    );
        
    let new_balance = seller_balance.amount - order.amount;
    let mut active_seller_balance = seller_balance.into_active_model();
    active_seller_balance.amount = Set(new_balance);
    try_or_http_err!(active_seller_balance.update(state.db.as_ref()).await);
    
    let new_balance = (seller.balance + order.price).round_dp(3);
    let mut active_seller = seller.into_active_model();
    active_seller.balance = Set(new_balance);
    try_or_http_err!(active_seller.update(state.db.as_ref()).await);

    let buyer_balance = match try_or_http_err!(
        user_balances::Entity::find()
            .filter(
                Condition::all()
                    .add(user_balances::Column::UserId.eq(order.user_id))
                    .add(user_balances::Column::AssetId.eq(order.asset_id))
            )
            .one(state.db.as_ref())
            .await
    ) {
        Some(_data) => {_data}
        None => {
            user_balances::Model {
                id: 0,
                user_id: order.user_id,
                asset_id: order.asset_id,
                amount: Decimal::from(0),
            }
        }
    };

    let _user_amount = buyer_balance.amount;
    let mut active_buyer_balance = buyer_balance.into_active_model();
    active_buyer_balance.amount = Set(_user_amount + take_commission(order.amount, COMMISSION_ORDER_BUY.clone()).amount);
    if let Err(_) = active_buyer_balance.update(state.db.as_ref()).await {
        let _asset = user_balances::ActiveModel {
            user_id: Set(order.user_id),
            asset_id: Set(order.asset_id),
            amount: Set(_user_amount + take_commission(order.amount, COMMISSION_ORDER_BUY.clone()).amount),
            ..Default::default()
        };

        try_or_http_err!(_asset.insert(state.db.as_ref()).await);
    }

    let _ = trades::ActiveModel {
        user_id: Set(token.0.claims.sub),
        asset_id: Set(order.asset_id),
        trade_type: Set("sell".into()),
        price: Set(order.price),
        amount: Set(order.amount),
        ..Default::default()
    }.insert(state.db.as_ref()).await;

    let _ = trades::ActiveModel {
        user_id: Set(order.user_id),
        asset_id: Set(order.asset_id),
        trade_type: Set("buy".into()),
        price: Set(order.price),
        amount: Set(order.amount),
        ..Default::default()
    }.insert(state.db.as_ref()).await;

    let mut active_order = order.into_active_model();
    active_order.status = Set("done".into());
    active_order.updated_at = Set(Utc::now().naive_utc());
    try_or_http_err!(active_order.update(state.db.as_ref()).await);
    
    HttpResponse::Ok().json(CommonResponse::<BuyOrderResponse> {
        status: ResponseStatus::Ok,
        data: BuyOrderResponse{
            balance: new_balance,
        },
        error: None,
    })
}

#[derive(Deserialize, ToSchema)]
pub struct OrderBuyInput {
    order_id: i32,
}

#[derive(Serialize)]
pub struct BuyOrderResponse {
    balance: Decimal
}

use crate::utils::jwt::AccessToken;
use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::{try_or_http_err, AppState};
use actix_web::{post, web, HttpResponse, Responder};
use entity::prelude::Orders;
use entity::{orders, user_balances, users};
use sea_orm::prelude::Decimal;
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, DbConn, EntityTrait, IntoActiveModel, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::error::Error;
use utoipa::ToSchema;
use crate::structs::order_structs::OrderType;

#[utoipa::path(
    request_body = OrderCreateInput,
    tag="User",
    security(
        ("bearer_token" = [])
    )
)]
#[post("/api/v1/order/create")]
pub async fn order_create(
    state: web::Data<AppState>,
    input: web::Json<OrderCreateInput>,
    token: AccessToken,
) -> impl Responder {
    match input.order_type {
        OrderType::Buy => {
            let order_id = try_or_http_err!(
                __create_buy_order(
                    token.0.claims.sub,
                    input.asset_id,
                    try_or_http_err!(Decimal::from_f64_retain(input.amount).ok_or("Wrong amount")),
                    try_or_http_err!(Decimal::from_f64_retain(input.price).ok_or("Wrong price")),
                    state.db.as_ref()
                )
                .await
            );
            HttpResponse::Ok().json(CommonResponse::<OrderCreateResponse> {
                status: ResponseStatus::Ok,
                data: OrderCreateResponse { order_id },
                error: None,
            })
        }
        OrderType::Sell => {
            let order_id = try_or_http_err!(
                __create_sell_order(
                    token.0.claims.sub,
                    input.asset_id,
                    try_or_http_err!(Decimal::from_f64_retain(input.amount).ok_or("Wrong amount")),
                    try_or_http_err!(Decimal::from_f64_retain(input.price).ok_or("Wrong price")),
                    state.db.as_ref()
                )
                .await
            );
            HttpResponse::Ok().json(CommonResponse::<OrderCreateResponse> {
                status: ResponseStatus::Ok,
                data: OrderCreateResponse { order_id },
                error: None,
            })
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct OrderCreateInput {
    order_type: OrderType,
    asset_id: i32,
    amount: f64,
    price: f64,
}

#[derive(Serialize)]
pub struct OrderCreateResponse {
    order_id: i32,
}

async fn __create_buy_order(
    user_id: i32,
    asset_id: i32,
    amount: Decimal,
    price: Decimal,
    db: &DbConn,
) -> Result<i32, Box<dyn Error + Send + Sync>> {
    let user = match users::Entity::find_by_id(user_id).one(db).await? {
        Some(_user) => _user,
        None => return Err("No user".into()),
    };
    if user.balance < price {
        return Err("Not enough balance".into());
    }

    let order = orders::ActiveModel {
        user_id: Set(user_id),
        asset_id: Set(asset_id),
        order_type: Set("buy".into()),
        price: Set(price),
        amount: Set(amount),
        status: Set("pending".into()),
        ..Default::default()
    };
    let new_balance = (user.balance - price).round_dp(3);
    let mut active_user = user.into_active_model();
    active_user.balance = Set(new_balance);
    active_user.update(db).await?;
    let data = Orders::insert(order).exec(db).await?;

    Ok(data.last_insert_id)
}

async fn __create_sell_order(
    user_id: i32,
    asset_id: i32,
    amount: Decimal,
    price: Decimal,
    db: &DbConn,
) -> Result<i32, Box<dyn Error + Send + Sync>> {
    let user_balance = match user_balances::Entity::find()
        .filter(
            Condition::all()
                .add(user_balances::Column::UserId.eq(user_id))
                .add(user_balances::Column::AssetId.eq(asset_id)),
        )
        .one(db)
        .await? {
        Some(_balance) => { _balance }
        None => { return Err("No asset data for this user".into()) }
    };
    
    if user_balance.amount < amount {
        return Err("Not enough asset amount".into())
    }

    let order = orders::ActiveModel {
        user_id: Set(user_id),
        asset_id: Set(asset_id),
        order_type: Set("sell".into()),
        price: Set(price),
        amount: Set(amount),
        status: Set("pending".into()),
        ..Default::default()
    };
    let new_amount = (user_balance.amount - amount).round_dp(3);
    let mut active_user_balance = user_balance.into_active_model();
    active_user_balance.amount = Set(new_amount);
    active_user_balance.update(db).await?;
    let data = Orders::insert(order).exec(db).await?;

    Ok(data.last_insert_id)
}

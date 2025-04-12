use crate::utils::jwt::AccessToken;
use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::{try_or_http_err, AppState};
use actix_web::{post, web, HttpResponse, Responder};
use chrono::Utc;
use entity::{orders, user_balances, users};
use migration::Condition;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::{ActiveModelTrait, DbConn, EntityTrait, IntoActiveModel, Set};
use serde::Deserialize;
use std::error::Error;
use utoipa::ToSchema;

#[utoipa::path(
    request_body = OrderCancelInput,
    tag="User",
    security(
        ("bearer_token" = [])
    )
)]
#[post("/api/v1/order/cancel")]
pub async fn order_cancel(
    state: web::Data<AppState>,
    input: web::Json<OrderCancelInput>,
    token: AccessToken,
) -> impl Responder {
    match try_or_http_err!(
        orders::Entity::find_by_id(input.order_id)
            .one(state.db.as_ref())
            .await
    ) {
        Some(order) => {
            if order.user_id != token.claims.sub {
                return HttpResponse::BadRequest().json(CommonResponse::<()> {
                    status: ResponseStatus::Error,
                    data: (),
                    error: Some("No order with this ID".into()),
                });
            }
            if order.status != "pending" {
                return HttpResponse::BadRequest().json(CommonResponse::<()> {
                    status: ResponseStatus::Error,
                    data: (),
                    error: Some("Can't cancel this order".into()),
                });
            }
            match order.order_type.as_str() {
                "buy" => {
                    try_or_http_err!(__cancel_buy_order(order, state.db.as_ref()).await);
                    HttpResponse::Ok().json(CommonResponse::<()> {
                        status: ResponseStatus::Ok,
                        data: (),
                        error: None,
                    })
                }
                "sell" => {
                    try_or_http_err!(__cancel_sell_order(order, state.db.as_ref()).await);
                    HttpResponse::Ok().json(CommonResponse::<()> {
                        status: ResponseStatus::Ok,
                        data: (),
                        error: None,
                    })
                }
                _ => HttpResponse::InternalServerError().json(CommonResponse::<()> {
                    status: ResponseStatus::Error,
                    data: (),
                    error: Some("Unexpected order type".into()),
                }),
            }
        }
        None => HttpResponse::BadRequest().json(CommonResponse::<()> {
            status: ResponseStatus::Error,
            data: (),
            error: Some("No order with this ID".into()),
        }),
    }
}

#[derive(Deserialize, ToSchema)]
pub struct OrderCancelInput {
    order_id: i32,
}

async fn __cancel_buy_order(
    order: orders::Model,
    db: &DbConn,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let user = match users::Entity::find_by_id(order.user_id).one(db).await? {
        Some(_user) => _user,
        None => return Err("No user".into()),
    };

    let new_balance = (user.balance + order.price).round_dp(3);
    let mut active_user = user.into_active_model();
    active_user.balance = Set(new_balance);
    let mut active_order = order.into_active_model();
    active_order.status = Set("cancel".into());
    active_order.updated_at = Set(Utc::now().naive_utc());
    
    active_user.update(db).await?;
    active_order.update(db).await?;
    Ok(())
}

async fn __cancel_sell_order(
    order: orders::Model,
    db: &DbConn,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let user_balance = match user_balances::Entity::find()
        .filter(
            Condition::all()
                .add(user_balances::Column::UserId.eq(order.user_id))
                .add(user_balances::Column::AssetId.eq(order.asset_id)),
        )
        .one(db)
        .await? {
        Some(_balance) => { _balance }
        None => { return Err("No asset data for this user".into()) }
    };
    
    let new_amount = (user_balance.amount + order.amount).round_dp(3);
    let mut active_user_balance = user_balance.into_active_model();
    active_user_balance.amount = Set(new_amount);
    let mut active_order = order.into_active_model();
    active_order.status = Set("cancel".into());
    active_order.updated_at = Set(Utc::now().naive_utc());
    
    active_user_balance.update(db).await?;
    active_order.update(db).await?;

    Ok(())
}

use crate::unwrap_or_http_err_with_opt_msg;
use actix_web::{post, web, HttpResponse, Responder};
use chrono::Utc;
use sea_orm::prelude::Decimal;
use sea_orm::{EntityTrait, Set};
use serde::Deserialize;
use utoipa::ToSchema;
use entity::{orders, trades, user_balances, users};
use crate::{extract_db_response_or_http_err_with_opt_msg, try_or_http_err, AppState, COMMISSION_ORDER_SELL};
use crate::utils::jwt::AccessToken;
use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::utils::take_commission::take_commission;

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
    
    
    
    
    HttpResponse::Ok().json(CommonResponse::<()> {
        status: ResponseStatus::Ok,
        data: (),
        error: None,
    })
}


#[derive(Deserialize, ToSchema)]
pub struct OrderBuyInput {
    order_id: i32
}
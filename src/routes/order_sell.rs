use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use utoipa::ToSchema;
use crate::AppState;
use crate::routes::order_create::OrderCreateInput;
use crate::utils::jwt::AccessToken;

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
    input: web::Json<OrderCreateInput>,
    token: AccessToken,
) -> impl Responder {
    HttpResponse::Ok()
}


#[derive(Deserialize, ToSchema)]
pub struct OrderSellInput {
    order_id: i32
}
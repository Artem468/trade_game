use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use utoipa::ToSchema;
use crate::AppState;
use crate::routes::order_create::OrderCreateInput;
use crate::utils::jwt::AccessToken;


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
    input: web::Json<OrderCreateInput>,
    token: AccessToken,
) -> impl Responder {
    HttpResponse::Ok()
}


#[derive(Deserialize, ToSchema)]
pub struct OrderBuyInput {
    order_id: i32
}
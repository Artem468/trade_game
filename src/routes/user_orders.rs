use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::{try_or_http_err, AppState};
use actix_web::{get, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use entity::{orders, users};
use sea_orm::prelude::{Decimal, Expr};
use sea_orm::{ColumnTrait, Condition, EntityTrait, FromQueryResult, QueryFilter, QuerySelect};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[utoipa::path(params(OrderQuery, OrdersPath))]
#[get("/api/v1/orders/{asset_id}")]
pub async fn user_orders(
    state: web::Data<AppState>,
    path: web::Path<OrdersPath>,
    query: web::Query<OrderQuery>,
) -> impl Responder {
    let data = try_or_http_err!(
        orders::Entity::find()
            .left_join(users::Entity)
            .column_as(
                Expr::col((users::Entity, users::Column::Username)),
                "username"
            )
            .filter(orders::Column::Id.eq(path.asset_id))
            .limit(query.limit)
            .offset(query.offset)
            .into_model::<OrderResponse>()
            .all(state.db.as_ref())
            .await
    );

    HttpResponse::Ok().json(CommonResponse::<Vec<OrderResponse>> {
        status: ResponseStatus::Ok,
        data,
        error: None,
    })
}

#[utoipa::path(params(OrderQuery, OrdersPathByUser))]
#[get("/api/v1/orders/{asset_id}/{user_id}")]
pub async fn user_orders_by_user(
    state: web::Data<AppState>,
    path: web::Path<OrdersPathByUser>,
    query: web::Query<OrderQuery>,
) -> impl Responder {
    let data = try_or_http_err!(
        orders::Entity::find()
            .left_join(users::Entity)
            .column_as(
                Expr::col((users::Entity, users::Column::Username)),
                "username"
            )
            .filter(
                Condition::all()
                    .add(orders::Column::Id.eq(path.asset_id))
                    .add(users::Column::Id.eq(path.user_id))
            )
            .limit(query.limit)
            .offset(query.offset)
            .into_model::<OrderResponse>()
            .all(state.db.as_ref())
            .await
    );

    HttpResponse::Ok().json(CommonResponse::<Vec<OrderResponse>> {
        status: ResponseStatus::Ok,
        data,
        error: None,
    })
}

#[derive(Serialize, FromQueryResult)]
struct OrderResponse {
    id: i32,
    user_id: i32,
    username: String,
    asset_id: i32,
    order_type: String,
    order_side: String,
    price: Option<Decimal>,
    amount: Decimal,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct OrdersPath {
    pub asset_id: i32,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct OrdersPathByUser {
    pub user_id: i32,
    pub asset_id: i32,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct OrderQuery {
    pub limit: u64,
    pub offset: u64,
}

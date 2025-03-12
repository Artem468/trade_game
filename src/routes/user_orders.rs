use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::{try_or_http_err, AppState};
use actix_web::{get, web, HttpResponse, Responder};
use chrono::NaiveDateTime;
use entity::{orders, users};
use sea_orm::prelude::{Decimal, Expr};
use sea_orm::{ColumnTrait, Condition, EntityTrait, FromQueryResult, QueryFilter, QuerySelect};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use crate::structs::order_structs::{OrderStatus, OrderType};

#[utoipa::path(params(OrderQuery, OrdersPath), tag = "User")]
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
            .filter({
                let mut cond = Condition::all().add(orders::Column::AssetId.eq(path.asset_id));
            
                if let Some(status) = &query.status {
                    match status {
                        OrderStatus::Pending => {
                            cond = cond.add(orders::Column::Status.eq("pending"));
                        }
                        OrderStatus::Done => {
                            cond = cond.add(orders::Column::Status.eq("done"));
                        }
                        OrderStatus::Cancel => {
                            cond = cond.add(orders::Column::Status.eq("cancel"));
                        }
                    }
                }

                if let Some(order_type) = &query.order_type {
                    match order_type {
                        OrderType::Buy => {
                            cond = cond.add(orders::Column::OrderType.eq("buy"));
                        }
                        OrderType::Sell => {
                            cond = cond.add(orders::Column::OrderType.eq("sell"));
                        }
                    }
                }

                cond
            })
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

#[utoipa::path(params(OrderQuery, OrdersPathByUser), tag = "User")]
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
            .filter({
                let mut cond = Condition::all()
                    .add(orders::Column::AssetId.eq(path.asset_id))
                    .add(users::Column::Id.eq(path.user_id));

                if let Some(status) = &query.status {
                    match status {
                        OrderStatus::Pending => {
                            cond = cond.add(orders::Column::Status.eq("pending"));
                        }
                        OrderStatus::Done => {
                            cond = cond.add(orders::Column::Status.eq("done"));
                        }
                        OrderStatus::Cancel => {
                            cond = cond.add(orders::Column::Status.eq("cancel"));
                        }
                    }
                }

                if let Some(order_type) = &query.order_type {
                    match order_type {
                        OrderType::Buy => {
                            cond = cond.add(orders::Column::OrderType.eq("buy"));
                        }
                        OrderType::Sell => {
                            cond = cond.add(orders::Column::OrderType.eq("sell"));
                        }
                    }
                }

                cond
            })
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
    price: Option<Decimal>,
    amount: Decimal,
    status: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
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
    pub offset: Option<u64>,
    pub status: Option<OrderStatus>,
    pub order_type: Option<OrderType>,
}


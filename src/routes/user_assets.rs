use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::{try_or_http_err, AppState};
use actix_web::{get, web, HttpResponse, Responder};
use entity::{assets, user_balances, users};
use sea_orm::prelude::{Decimal, Expr};
use sea_orm::QueryFilter;
use sea_orm::{ColumnTrait, Condition, EntityTrait, FromQueryResult, QuerySelect};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[utoipa::path(params(AssetQuery))]
#[get("/api/v1/user/assets/{user_id}")]
pub async fn user_assets(state: web::Data<AppState>, path: web::Path<AssetQuery>) -> impl Responder {
    let user_data = try_or_http_err!(
        users::Entity::find_by_id(path.user_id)
            .one(state.db.as_ref())
            .await
    );
    if let Some(user) = user_data {
        let data = try_or_http_err!(
            assets::Entity::find()
                .left_join(user_balances::Entity)
                .column_as(
                    Expr::col((user_balances::Entity, user_balances::Column::Amount)).if_null(0),
                    "amount"
                )
                .filter(
                    Condition::any()
                        .add(user_balances::Column::UserId.eq(user.id))
                        .add(user_balances::Column::UserId.is_null()),
                )
                .into_model::<AssetWithBalance>()
                .all(state.db.as_ref())
                .await
        );

        return HttpResponse::Ok().json(CommonResponse::<Vec<AssetWithBalance>> {
            status: ResponseStatus::Ok,
            data,
            error: None,
        });
    }

    HttpResponse::Ok().json(CommonResponse::<()> {
        status: ResponseStatus::Error,
        data: (),
        error: Some("No user".into()),
    })
}

#[derive(Debug, FromQueryResult, Serialize)]
struct AssetWithBalance {
    id: i32,
    symbol: String,
    name: String,
    amount: Option<Decimal>,
}


#[derive(Deserialize, ToSchema, IntoParams)]
pub struct AssetQuery {
    pub user_id: i32,
}

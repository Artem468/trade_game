use crate::utils::jwt::AccessToken;
use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::{try_or_http_err, AppState};
use actix_web::{web, HttpResponse, Responder};
use entity::{assets, user_balances, users};
use sea_orm::{ColumnTrait, Condition, EntityTrait, FromQueryResult, QuerySelect};
use sea_orm::prelude::Decimal;
use sea_orm::QueryFilter;
use serde::Serialize;

pub async fn user_assets(
    state: web::Data<AppState>,
    token: AccessToken,
) -> impl Responder {
    let user_data = try_or_http_err!(users::Entity::find_by_id(token.0.claims.sub).one(state.db.as_ref()).await);
    if let Some(user) = user_data {
        let data = try_or_http_err!(
            assets::Entity::find()
            .left_join(user_balances::Entity)
            .column_as(user_balances::Column::Amount, "amount")
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
            error: None
        })
        
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

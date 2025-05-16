use crate::try_or_http_err;
use crate::unwrap_or_http_err_with_opt_msg;
use crate::utils::jwt::generate_access_token;
use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::{extract_db_response_or_http_err_with_opt_msg, AppState, RECOVERSTORAGE};
use actix_web::{post, web, HttpResponse, Responder};
use entity::users;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[utoipa::path(
    request_body = RecoverCodeInput,
    tag="Authorization"
)]
#[post("/api/v1/recover/check")]
pub async fn check_recover_code(
    state: web::Data<AppState>,
    input: web::Json<RecoverCodeInput>,
) -> impl Responder {
    let input = input.into_inner();
    let user = extract_db_response_or_http_err_with_opt_msg!(
        users::Entity::find()
            .filter(users::Column::Email.eq(&input.email))
            .one(state.db.as_ref())
            .await,
        "User not found"
    );

    let mut codes_storage = RECOVERSTORAGE.lock().await;
    match codes_storage.get_mut(&user.id) {
        Some(codes) => {
            if codes.all().await.contains(&input.code) {
                let access_token = try_or_http_err!(generate_access_token(
                    user.id,
                    user.username.as_str(),
                    user.email.as_str(),
                    state.jwt_secret.as_str()
                ));

                return HttpResponse::Ok().json(CommonResponse::<RecoverResponse> {
                    status: ResponseStatus::Ok,
                    data: RecoverResponse { access_token },
                    error: None,
                });
            }

            HttpResponse::Forbidden().json(CommonResponse::<()> {
                status: ResponseStatus::Error,
                data: (),
                error: Some("Wrong code".into()),
            })
        }
        None => HttpResponse::BadRequest().json(CommonResponse::<()> {
            status: ResponseStatus::Error,
            data: (),
            error: Some("No codes for user".into()),
        }),
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RecoverCodeInput {
    email: String,
    code: i32,
}

#[derive(Debug, Serialize)]
pub struct RecoverResponse {
    access_token: String,
}

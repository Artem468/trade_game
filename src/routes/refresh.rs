use crate::routes::login::LoginResponse;
use crate::utils::jwt::{generate_access_token, generate_refresh_token, Claims};
use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::{try_or_http_err, AppState};
use actix_web::{post, web, HttpResponse, Responder};
use entity::users;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[utoipa::path(request_body = RefreshInput)]
#[post("/api/v1/auth/refresh")]
pub async fn refresh(state: web::Data<AppState>, input: web::Json<RefreshInput>) -> impl Responder {
    let validation = Validation::new(Algorithm::HS256);
    let token_data = match decode::<Claims>(
        &input.refresh_token,
        &DecodingKey::from_secret(state.jwt_secret.as_ref()),
        &validation,
    ) {
        Ok(data) => data,
        Err(_) => {
            return HttpResponse::Unauthorized().json(CommonResponse::<()> {
                status: ResponseStatus::Error,
                data: (),
                error: Some("Invalid refresh token".to_string()),
            });
        }
    };

    let claims = token_data.claims;

    if claims.token_type != "refresh" {
        return HttpResponse::Unauthorized().json(CommonResponse::<()> {
            status: ResponseStatus::Error,
            data: (),
            error: Some("Invalid token type".to_string()),
        });
    }

    let user = try_or_http_err!(
        users::Entity::find_by_id(claims.sub)
            .one(state.db.as_ref())
            .await
    );

    if let Some(user) = user {
        let access_token = try_or_http_err!(generate_access_token(
            user.id,
            user.username.as_str(),
            user.email.as_str(),
            state.jwt_secret.as_str()
        ));
        let refresh_token = try_or_http_err!(generate_refresh_token(
            user.id,
            user.username.as_str(),
            user.email.as_str(),
            state.jwt_secret.as_str()
        ));

        return HttpResponse::Ok().json(CommonResponse {
            status: ResponseStatus::Ok,
            data: RefreshResponse {
                access_token,
                refresh_token,
                user_id: user.id,
                email: user.email,
                username: user.username,
            },
            error: None,
        });
    }

    HttpResponse::Unauthorized().json(CommonResponse::<Option<LoginResponse>> {
        status: ResponseStatus::Error,
        data: None,
        error: Some("Unauthorized user".into()),
    })
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshInput {
    refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    access_token: String,
    refresh_token: String,
    user_id: i32,
    email: String,
    username: String,
}

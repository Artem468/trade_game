use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::{try_or_http_err, AppState};
use actix_web::{post, web, HttpResponse, Responder};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use entity::prelude::Users;
use entity::users;
use rand_core::OsRng;
use sea_orm::{EntityTrait, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[utoipa::path(
    request_body = BotInput,
    tag="Market"
)]
#[post("/api/v1/bots/create")]
pub async fn create_bot(
    state: web::Data<AppState>,
    input: web::Json<BotInput>,
) -> impl Responder {
    let input = input.into_inner();

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(input.password.as_bytes(), &salt)
        .unwrap()
        .to_string();


    let bot = users::ActiveModel {
        username: Set(input.username),
        email: Set(input.email),
        hashed_password: Set(password_hash),
        is_bot: Set(true),
        ..Default::default()
    };
    let bot_id = try_or_http_err!(Users::insert(bot).exec(state.db.as_ref()).await);

    HttpResponse::Ok().json(CommonResponse::<BotResponse> {
        status: ResponseStatus::Ok,
        data: BotResponse {
            bot_id: bot_id.last_insert_id,
        },
        error: None,
    })
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BotInput {
    username: String,
    email: String,
    password: String
}

#[derive(Serialize)]
pub struct BotResponse {
    bot_id: i32,
}

use crate::utils::limited_list_with_timeout::LimitedListWithTimeout;
use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::{extract_db_response_or_http_err_with_opt_msg, try_or_http_err, AppState};
use crate::{unwrap_or_http_err_with_opt_msg, RECOVERSTORAGE};
use actix_web::{post, web, HttpResponse, Responder};
use entity::users;
use lettre::message::header::ContentType;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use rand::Rng;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use std::error::Error;
use utoipa::ToSchema;

#[utoipa::path(
    request_body = RecoverEmailInput,
    tag="Authorization"
)]
#[post("/api/v1/recover/send")]
pub async fn recover_account(
    state: web::Data<AppState>,
    input: web::Json<RecoverEmailInput>,
) -> impl Responder {
    let input = input.into_inner();

    let user = extract_db_response_or_http_err_with_opt_msg!(
        users::Entity::find()
            .filter(users::Column::Email.eq(&input.email))
            .one(state.db.as_ref())
            .await,
        "User not found"
    );

    let code = rand::rng().random_range(100_000..=999_999);
    {
        let mut codes_storage = RECOVERSTORAGE.lock().await;
        match codes_storage.get_mut(&user.id) {
            Some(codes) => {
                if codes.is_full().await {
                    return HttpResponse::Ok().json(CommonResponse::<()> {
                        status: ResponseStatus::Error,
                        data: (),
                        error: Some("Too many attempts".into()),
                    });
                }
                codes.add(code).await;
            }
            None => {
                let mut codes = LimitedListWithTimeout::new(3);
                codes.add(code).await;
                codes_storage.insert(user.id, codes);
            }
        }
    }

    match send_main(
        input.email.as_str(),
        code,
        state.recover_from.as_str(),
        state.recover_password.as_str(),
    ) {
        Ok(_) => HttpResponse::Ok().json(CommonResponse::<()> {
            status: ResponseStatus::Ok,
            data: (),
            error: None,
        }),
        Err(err) => HttpResponse::InternalServerError().json(CommonResponse::<()> {
            status: ResponseStatus::Error,
            data: (),
            error: Some(err.to_string()),
        })
    }
}

fn send_main(
    recover_to: &str,
    code: i32,
    recover_from: &str,
    recover_password: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let template = r#"
    <!DOCTYPE html>
    <html lang="ru">
    <head>
      <meta charset="UTF-8">
      <title>Восстановление пароля</title>
      <style>
        body {
          font-family: Arial, sans-serif;
          background-color: #f7f7f7;
          color: #333;
          padding: 20px;
        }
        .container {
          max-width: 600px;
          background-color: #ffffff;
          margin: 0 auto;
          padding: 30px;
          border-radius: 10px;
          box-shadow: 0 4px 12px rgba(0,0,0,0.1);
        }
        h1 {
          color: #1641b7;
        }
        p {
          font-size: 16px;
          line-height: 1.5;
        }
      </style>
    </head>
    <body>
      <div class="container">
        <h1>Сброс пароля</h1>
        <p>Здравствуйте!</p>
        <p>Мы получили запрос на сброс пароля для вашей учетной записи. Если вы не отправляли этот запрос, просто проигнорируйте это письмо.</p>
        <p>Чтобы сбросить пароль, введите этот код в приложение: {{reset_code}}</p>
        <p>Код действителен в течение 30 минут.</p>
      </div>
    </body>
    </html>
    "#;

    let email = Message::builder()
        .from(recover_from.parse::<Mailbox>()?)
        .to(recover_to.parse::<Mailbox>()?)
        .subject("Восстановление пароля")
        .header(ContentType::TEXT_HTML)
        .body(template.replace("{{reset_code}}", code.to_string().as_str()))
        .unwrap();

    let creds = Credentials::new(recover_from.to_string(), recover_password.to_string());

    let mailer = SmtpTransport::relay("smtp.mail.ru")
        .unwrap()
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Failed to send email: {:?}", email).into()),
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RecoverEmailInput {
    email: String,
}
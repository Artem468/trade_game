use crate::utils::jwt::AccessToken;
use crate::{AppState, CHAT_SESSIONS};
use actix::{Actor, AsyncContext, Handler, Message as ActixMessage, StreamHandler};
use actix_web::{error, get, web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use chrono::{DateTime, Utc};
use entity::messages::Entity as MessageEntity;
use entity::{messages, users};
use sea_orm::{EntityTrait, Set};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

impl AppState {
    async fn send_message(&self, from_id: i32, recipient_id: i32, text: String) {
        if from_id == recipient_id { return }
        
        let created_at = Utc::now();
        let message = messages::ActiveModel {
            from_id: Set(from_id),
            recipient_id: Set(recipient_id),
            text: Set(text.clone()),
            created_at: Set(created_at.naive_utc()),
            ..Default::default()
        };
        let message_id = match MessageEntity::insert(message).exec(self.db.as_ref()).await {
            Ok(msg) => {msg.last_insert_id}
            Err(_) => { return }
        };

        if let Some(addr) = CHAT_SESSIONS.read().await.get(&recipient_id) {
            let _ = addr.do_send(OutgoingClientMessage {
                from_id,
                message_id,
                text,
                created_at
            });
        }
    }
}

#[utoipa::path(
    request_body=IncomingClientMessage,
    tag="User",
    security(
        ("bearer_token" = [])
    )
)]
#[get("/api/v1/chat/private")]
pub async fn chat_ws(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
    token: AccessToken,
) -> Result<HttpResponse, Error> {
    match users::Entity::find_by_id(token.0.claims.sub).one(state.db.as_ref()).await
    {
        Ok(data) => match data {
            Some(user) => {
                let session = ChatSession {
                    id: user.id,
                    state: state.into_inner(),
                };
                ws::start(session, &req, stream)
            }
            None => { Err(error::ErrorUnauthorized("No user")) }
        },
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}


#[derive(ActixMessage, Serialize, Deserialize, Debug, ToSchema)]
#[rtype(result = "()")]
struct IncomingClientMessage {
    recipient_id: i32,
    text: String,
}

#[derive(ActixMessage, Serialize, Deserialize, Debug)]
#[rtype(result = "()")]
struct OutgoingClientMessage {
    from_id: i32,
    message_id: i32,
    text: String,
    created_at: DateTime<Utc>
}

pub(crate) struct ChatSession {
    id: i32,
    state: Arc<AppState>,
}

impl Actor for ChatSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        let session_id = self.id;
        tokio::spawn(async move {
            let mut sessions = CHAT_SESSIONS.write().await;
            sessions.insert(session_id, addr);
        });
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> actix::Running {
        let session_id = self.id;
        tokio::spawn(async move {
            let mut sessions = CHAT_SESSIONS.write().await;
            sessions.remove(&session_id);
        });
        actix::Running::Stop
    }
}
impl Handler<OutgoingClientMessage> for ChatSession {
    type Result = ();
    fn handle(&mut self, msg: OutgoingClientMessage, ctx: &mut Self::Context) {
        let message_json = serde_json::to_string(&msg).unwrap_or_default();
        ctx.text(message_json);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, _ctx: &mut Self::Context) {
        if let Ok(ws::Message::Text(text)) = msg {
            if let Ok(client_msg) = serde_json::from_str::<IncomingClientMessage>(&text) {
                let state = Arc::clone(&self.state);
                let from_id = self.id;
                tokio::spawn(async move {
                    state
                        .send_message(from_id, client_msg.recipient_id, client_msg.text)
                        .await;
                });
            }
        }
    }
}

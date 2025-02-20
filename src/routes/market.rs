use actix::prelude::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use rand_core::RngCore;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize)]
struct MarketData {
    symbol: String,
    price: f64,
}

struct MarketWs;

impl Actor for MarketWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_secs(1), move |_, ctx| {
            let data = MarketData {
                symbol: "BTC/USDT".to_string(),
                price: rand_core::OsRng.next_u64() as f64,
            };
            let _ = ctx.text(serde_json::to_string(&data).unwrap());
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MarketWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(ws::Message::Ping(msg)) = msg {
            ctx.pong(&msg);
        }
    }
}

pub(crate) async fn market(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    if ws::handshake(&req).is_ok() {
        // Если запрос идет как WebSocket, запускаем WS-соединение
        return Ok(ws::start(MarketWs, &req, stream)?);
    }

    // Если обычный GET, отдаем JSON-ответ
    let data = MarketData {
        symbol: "BTC/USDT".to_string(),
        price: rand_core::OsRng.next_u64() as f64,
    };

    Ok(HttpResponse::Ok().json(data))
}
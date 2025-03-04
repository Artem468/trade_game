use crate::utils::response::{CommonResponse, ResponseStatus};
use crate::AppState;
use actix::prelude::*;
use actix_web::{web, Error as ActixError, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use chrono::Utc;
use entity::assets;
use redis::{AsyncCommands, Client};
use sea_orm::prelude::Decimal;
use sea_orm::{DbConn, EntityTrait};
use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

#[derive(Serialize)]
struct MarketData {
    symbol: String,
    price: Decimal,
    trend: String,
    change_percent: Decimal,
}

pub(crate) struct MarketWs {
    state: Arc<AppState>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct WebSocketMessage {
    pub message: String,
}

impl Actor for MarketWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let db = self.state.db.clone();
        let cache = self.state.cache.clone();
        let addr = ctx.address();
        ctx.run_interval(Duration::from_secs(5), move |_, _ctx| {
            
            let db = db.clone();
            let cache = cache.clone();
            let addr = addr.clone();
            
            tokio::spawn(async move {
                let data = __get_price_changes(db, cache).await.unwrap_or_default();
                let message_json = serde_json::to_string(&data).unwrap_or_default();

                addr.do_send(WebSocketMessage { message: message_json });
            });
        });

    }
}

impl Handler<WebSocketMessage> for MarketWs {
    type Result = ();

    fn handle(&mut self, msg: WebSocketMessage, ctx: &mut Self::Context) {
        ctx.text(msg.message);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MarketWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(ws::Message::Ping(msg)) = msg {
            ctx.pong(&msg);
        }
    }
}

pub(crate) async fn market(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, ActixError> {
    if ws::handshake(&req).is_ok() {
        return Ok(ws::start(MarketWs { state: state.into_inner() }, &req, stream)?);
    }
    let data = __get_price_changes(state.as_ref().db.clone(), state.as_ref().cache.clone()).await;

    match data {
        Ok(prices) => Ok(HttpResponse::Ok().json(CommonResponse::<HashMap<String, MarketData>> {
            status: ResponseStatus::Ok,
            data: prices,
            error: None,
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(CommonResponse::<HashMap<String, MarketData>> {
            status: ResponseStatus::Error,
            data: HashMap::new(),
            error: Some(e.to_string()),
        })),
    }
}


async fn __get_price_changes(
    db: Arc<DbConn>,
    cache: Arc<Client>,
) -> Result<HashMap<String, MarketData>, Box<dyn Error + Sync + Send>> {
    let mut redis_conn = cache.get_multiplexed_async_connection().await?;
    let keys: Vec<String> = redis_conn.keys("asset_price_history:*").await?;

    let now = Utc::now().timestamp();
    let day_ago = now - 86400;
    let mut result = HashMap::new();

    for key in keys {
        let asset_id = key.split(':').nth(1).unwrap_or("unknown").to_string();
        let asset = match assets::Entity::find_by_id(asset_id.parse::<i32>()?)
            .one(db.as_ref())
            .await?
        {
            Some(data) => data,
            None => Err("can't find asset")?,
        };

        let prices: Vec<(String, f64)> = redis_conn
            .zrevrangebyscore_withscores(&key, now, day_ago)
            .await
            .expect("REASON");
        if prices.is_empty() {}
        let first_price = Decimal::from_str(&prices.first().unwrap().0).unwrap_or(Decimal::ZERO);
        let last_price = Decimal::from_str(&prices.last().unwrap().0).unwrap_or(Decimal::ZERO);

        let change = if first_price != Decimal::ZERO {
            ((last_price - first_price) / first_price) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        let trend = if last_price > first_price {
            "up"
        } else if last_price < first_price {
            "down"
        } else {
            "unchanged"
        };

        result.insert(
            asset_id,
            MarketData {
                symbol: asset.symbol,
                price: last_price.round_dp(3),
                trend: trend.to_string(),
                change_percent: change.round_dp(2),
            },
        );
    }
    Ok(result)
}

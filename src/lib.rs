mod macros;
mod routes;
mod traits;
mod utils;

use crate::routes::prelude::*;
use crate::routes::private_chat::ChatSession;
use crate::utils::establish_connection::establish_connection;
use crate::utils::init_assets::initialize_assets;
use crate::utils::price_calculation::calculate_asset_prices;
use crate::utils::prices_snapshot::save_prices_to_db;
use crate::utils::seed_assets::seed_assets;
use actix::Addr;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use lazy_static::lazy_static;
use redis::Client;
use sea_orm::DbConn;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

lazy_static! {
    static ref CHAT_SESSIONS: RwLock<HashMap<i32, Addr<ChatSession>>> = RwLock::new(HashMap::new());
}

struct AppState {
    db: Arc<DbConn>,
    cache: Arc<Client>,
    jwt_secret: String,
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // tracing_subscriber::fmt().with_max_level(tracing::Level::TRACE).init();
    dotenv().ok();

    let db = Arc::new(establish_connection().await?);
    let cache = Arc::new(Client::open(
        std::env::var("REDIS_URL").expect("REDIS_URL must be set"),
    )?);
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    initialize_assets(db.as_ref()).await?;
    seed_assets(db.as_ref()).await?;

    task::spawn(calculate_asset_prices(
        db.as_ref().clone(),
        cache.as_ref().clone(),
        10,
    ));
    task::spawn(save_prices_to_db(
        db.as_ref().clone(),
        cache.as_ref().clone(),
        10_800,
    ));

    let app_state = web::Data::new(AppState {
        db,
        cache,
        jwt_secret,
    });

    #[derive(OpenApi)]
    #[openapi(
        info(title = "Trade game", description = "Trade game api"),
        paths(
            register::register,
            login::login,
            refresh::refresh,
            user_assets::user_assets,
            trades_history::trades_history,
            market::market,
            private_chat::chat_ws,
            chat_history::chat_history,
            user_orders::user_orders,
            user_orders::user_orders_by_user,
            market_buy::market_buy,
            market_sell::market_sell,
        ),
        modifiers(&SecurityAddon)
    )]
    struct ApiDoc;

    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            let components = openapi.components.as_mut().unwrap();
            components.add_security_scheme(
                "bearer_token",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }

    let host: String = std::env::var("HOST").unwrap_or("127.0.0.1".to_string());
    let port: String = std::env::var("port").unwrap_or("8080".to_string());

    HttpServer::new(move || {
        let app = App::new()
            .app_data(app_state.clone())
            .service(register::register)
            .service(login::login)
            .service(refresh::refresh)
            .service(user_assets::user_assets)
            .service(trades_history::trades_history)
            .service(market::market)
            .service(private_chat::chat_ws)
            .service(chat_history::chat_history)
            .service(user_orders::user_orders)
            .service(user_orders::user_orders_by_user)
            .service(market_buy::market_buy)
            .service(market_sell::market_sell);
        // .route("/api/v1/orders/buy", web::post().to(buy_order))
        // .route("/api/v1/orders/sell", web::post().to(sell_order_))
        

        if cfg!(feature = "docs") {
            app.service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
        } else {
            app
        }
    })
    .bind(format!("{host}:{port}"))?
    .run()
    .await?;

    Ok(())
}

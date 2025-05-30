mod macros;
mod routes;
mod structs;
mod traits;
mod utils;

use crate::routes::prelude::*;
use crate::routes::private_chat::ChatSession;
use crate::utils::establish_connection::establish_connection;
use crate::utils::init_assets::initialize_assets;
use crate::utils::limited_list_with_timeout::LimitedListWithTimeout;
use crate::utils::price_calculation::calculate_asset_prices;
use crate::utils::prices_snapshot::save_prices_to_db;
use crate::utils::seed_assets::seed_assets;
use actix::Addr;
use actix_cors::Cors;
use actix_files;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use lazy_static::lazy_static;
use redis::Client;
use sea_orm::prelude::Decimal;
use sea_orm::DbConn;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::task;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

lazy_static! {
    static ref CHAT_SESSIONS: RwLock<HashMap<i32, Addr<ChatSession>>> = RwLock::new(HashMap::new());
    static ref COMMISSION_MARKET_BUY: Decimal = Decimal::from_f64_retain(0.1).unwrap();
    static ref COMMISSION_MARKET_SELL: Decimal = Decimal::from_f64_retain(0.1).unwrap();
    static ref COMMISSION_ORDER_BUY: Decimal = Decimal::from_f64_retain(0.1).unwrap();
    static ref COMMISSION_ORDER_SELL: Decimal = Decimal::from_f64_retain(0.1).unwrap();
    static ref RECOVERSTORAGE: Mutex<HashMap<i32, LimitedListWithTimeout<i32>>> =
        Mutex::new(HashMap::new());
}

struct AppState {
    db: Arc<DbConn>,
    cache: Arc<Client>,
    jwt_secret: String,
    recover_from: String,
    recover_password: String,
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let db = Arc::new(establish_connection().await?);
    let cache = Arc::new(Client::open(
        std::env::var("REDIS_URL").expect("REDIS_URL must be set"),
    )?);
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let recover_from = std::env::var("RECOVER_FROM").expect("RECOVER_FROM must be set");
    let recover_password = std::env::var("RECOVER_PASSWORD").expect("RECOVER_PASSWORD must be set");

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
        recover_from,
        recover_password,
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
            top_users::top_users,
            order_buy::order_buy,
            order_sell::order_sell,
            order_create::order_create,
            order_cancel::order_cancel,
            user_info::user_info,
            price_history::price_history,
            create_event::create_event,
            get_events::get_events,
            create_bot::create_bot,
            get_bots::get_bots,
            get_user_place::get_user_place,
            get_chats::get_chats,
            recover_account::recover_account,
            check_recover_code::check_recover_code,
            change_password::change_password,
        ),
        modifiers(&SecurityAddon),
        tags(
            (name="Authorization", description="Auth methods"),
            (name="User", description="User methods"),
            (name="Market", description="Market methods"),

        )
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
    let port: String = std::env::var("PORT").unwrap_or("8080".to_string());

    HttpServer::new(move || {
        let mut app = App::new()
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
            .service(market_sell::market_sell)
            .service(top_users::top_users)
            .service(order_buy::order_buy)
            .service(order_sell::order_sell)
            .service(order_create::order_create)
            .service(order_cancel::order_cancel)
            .service(user_info::user_info)
            .service(price_history::price_history)
            .service(create_event::create_event)
            .service(get_events::get_events)
            .service(create_bot::create_bot)
            .service(get_bots::get_bots)
            .service(get_user_place::get_user_place)
            .service(get_chats::get_chats)
            .service(recover_account::recover_account)
            .service(check_recover_code::check_recover_code)
            .service(change_password::change_password);

        if cfg!(feature = "docs") {
            app = app.service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            );
        }

        app.service(actix_files::Files::new("/", "./web_view/dist").index_file("index.html"))
            .wrap(Cors::permissive())
    })
    .bind(format!("{host}:{port}"))?
    .run()
    .await?;

    Ok(())
}

mod utils;
mod routes;
mod macros;
mod traits;

use std::sync::Arc;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use lazy_static::lazy_static;
use redis::Client;
use sea_orm::DbConn;
use tokio::sync::RwLock;
use tokio::task;
use crate::routes::market;
use crate::routes::prelude::*;
use crate::utils::establish_connection::establish_connection;
use crate::utils::init_assets::initialize_assets;
use crate::utils::price_calculation::calculate_asset_prices;
use crate::utils::seed_assets::seed_assets;

struct AppState {
    db: Arc<DbConn>,
    cache: Arc<Client>,
    jwt_secret: String,
}

pub struct GlobalData {
    db: Arc<DbConn>
}

lazy_static! {
    static ref GLOBALDATA: RwLock<Option<GlobalData>> = RwLock::new(None);
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // tracing_subscriber::fmt().with_max_level(tracing::Level::TRACE).init();
    dotenv().ok();
    
    let db = Arc::new(establish_connection().await?);
    let cache = Arc::new(Client::open(std::env::var("REDIS_URL").expect("REDIS_URL must be set"))?);
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    initialize_assets(db.as_ref()).await?;
    seed_assets(db.as_ref()).await?;

    task::spawn(calculate_asset_prices(db.as_ref().clone(), cache.as_ref().clone(), 10));
    
    init_global_data(db.clone()).await;
    
    let app_state = web::Data::new(AppState {
        db,
        cache,
        jwt_secret,
    });
    
    let host: String = std::env::var("HOST").unwrap_or("127.0.0.1".to_string());
    let port: String = std::env::var("port").unwrap_or("8080".to_string());
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/api/v1/auth/register", web::post().to(register::register))
            .route("/api/v1/auth/login", web::post().to(login::login))
            .route("/api/v1/auth/refresh", web::post().to(refresh::refresh))
            .route("/api/v1/user/assets", web::get().to(user_assets::user_assets))
            .route("/api/v1/market/data", web::get().to(market::market))
    })
        .bind(format!("{host}:{port}"))?
        .run()
        .await?;

    Ok(())
}

async fn init_global_data(db: Arc<DbConn>) {
    *GLOBALDATA.write().await = Some(GlobalData { db });
}

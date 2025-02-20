mod utils;
mod routes;
mod macros;

use std::sync::Arc;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use sea_orm::DbConn;
use crate::routes::market;
use crate::routes::prelude::*;
use crate::utils::establish_connection::establish_connection;
use crate::utils::init_assets::initialize_default_assets;

struct AppState {
    db: Arc<DbConn>,
    jwt_secret: String,
}


pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_max_level(tracing::Level::TRACE).init();
    dotenv().ok();
    
    let db = Arc::new(establish_connection().await?);
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    initialize_default_assets(db.as_ref()).await?;
    
    let app_state = web::Data::new(AppState {
        db,
        jwt_secret
    });

    let host: String = std::env::var("HOST").unwrap_or("127.0.0.1".to_string());
    let port: String = std::env::var("port").unwrap_or("8080".to_string());
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/api/v1/auth/register", web::post().to(register::register))
            .route("/api/v1/auth/login", web::post().to(login::login))
            .route("/api/v1/auth/refresh", web::post().to(refresh::refresh))
            .route("/api/v1/market/data", web::get().to(market::market))
    })
        .bind(format!("{host}:{port}"))?
        .run()
        .await?;

    Ok(())
}
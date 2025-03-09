use entity::{assets, price_snapshot};
use redis::AsyncCommands;
use sea_orm::prelude::Decimal;
use sea_orm::{
    DbConn, EntityTrait, Set,
};
use std::error::Error;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::interval;

pub async fn save_prices_to_db(db: DbConn, redis_client: redis::Client, n: u64) {
    let mut interval = interval(Duration::from_secs(n));
    loop {
        interval.tick().await;
        if let Err(err) = save_prices_executor(&db, &redis_client).await {
            eprintln!("Error updating asset prices: {err}");
        }
    }
}

pub async fn save_prices_executor(db: &DbConn, redis_client: &redis::Client) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut redis_conn = redis_client.get_multiplexed_async_connection().await?;
    
    let assets: Vec<assets::Model> = assets::Entity::find().all(db).await?;
    let mut snapshots = Vec::new();
    
    for asset in assets {
        let key = format!("asset_price:{}", asset.id);
        let price_str: Option<String> = redis_conn.hget(&key, "price").await?;

        if let Some(price_str) = price_str {
            if let Ok(price) = Decimal::from_str(&price_str) {
                snapshots.push(price_snapshot::ActiveModel {
                    asset_id: Set(asset.id),
                    price: Set(price),
                    ..Default::default()
                });
            }
        }
    }
    if !snapshots.is_empty() {
        price_snapshot::Entity::insert_many(snapshots)
            .exec(db)
            .await?;
    }
    Ok(())
}

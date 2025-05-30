use crate::traits::redis::PriceInfo;
use chrono::Utc;
use entity::prelude::Assets;
use entity::{assets, price_snapshot};
use lazy_static::lazy_static;
use redis::AsyncCommands;
use sea_orm::prelude::Decimal;
use sea_orm::{ColumnTrait, DbConn, DbErr, EntityTrait, QueryFilter, QueryOrder};
use std::error::Error;
use std::str::FromStr;
use tokio::time::{interval, Duration};

lazy_static! {
    static ref K: Decimal = Decimal::from_str("0.01").unwrap();
    static ref EPSILON: Decimal = Decimal::from_str("0.0001").unwrap();
}

pub async fn calculate_asset_prices(db: DbConn, redis_client: redis::Client, n: u64) {
    let mut interval = interval(Duration::from_secs(n));
    loop {
        interval.tick().await;
        if let Err(err) = update_all_asset_prices(&db, &redis_client).await {
            eprintln!("Error updating asset prices: {err}");
        }
    }
}

async fn update_all_asset_prices(db: &DbConn, redis_client: &redis::Client) -> Result<(), DbErr> {
    let assets: Vec<assets::Model> = Assets::find().all(db).await?;

    let mut handles = vec![];
    for asset in assets {
        let db = db.clone();
        let redis_client = redis_client.clone();
        let handle = tokio::spawn(async move {
            if let Err(err) = calculate_asset_price(&db, &redis_client, asset.id).await {
                eprintln!("Failed to update price for asset {}: {err}", asset.id);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }
    Ok(())
}

async fn calculate_asset_price(
    db: &DbConn,
    redis_client: &redis::Client,
    asset_id: i32,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut redis_conn = redis_client.get_multiplexed_async_connection().await?;

    let mut old_price: PriceInfo = redis_conn
        .hgetall(&format!("asset_price:{asset_id}"))
        .await?;
    if old_price.price.is_none() {
        if let Some(snapshot) = price_snapshot::Entity::find()
            .filter(price_snapshot::Column::AssetId.eq(asset_id))
            .order_by_desc(price_snapshot::Column::CreatedAt)
            .one(db)
            .await?
        {
            old_price.price = Some(Decimal::from(snapshot.price));
            old_price.created_at = Some(snapshot.created_at.and_utc());
        } else {
            old_price.price = Some(Decimal::from(1));
            old_price.created_at = Some(Utc::now());
        }
    }

    let old_price_value = old_price.price.unwrap();
    let max_change_percent = Decimal::from_f64_retain(0.01).ok_or("Can't parse")?; // ±1%
    let continue_trend_chance = 0.7;
    
    let mut current_trend_up = rand::random::<bool>();
    
    let trend_roll: f64 = rand::random();
    if trend_roll >= continue_trend_chance {
        current_trend_up = !current_trend_up;
    }

    let magnitude: f64 = rand::random();
    let direction = if current_trend_up { 1.0 } else { -1.0 };
    let rand_change = direction * magnitude;
    let change_factor = Decimal::from_f64_retain(rand_change).ok_or("Can't parse")? * max_change_percent;
    
    let final_price = (old_price_value * (Decimal::from(1) + change_factor))
        .max(Decimal::from_f64_retain(0.001).ok_or("Can't parse")?)
        .round_dp(3);


    let key = format!("asset_price:{}", asset_id);
    let history_key = format!("asset_price_history:{}", asset_id);
    let timestamp = Utc::now().timestamp();
    let minute_timestamp = timestamp / 60 * 60;
    
    let _: () = redis_conn
        .hset_multiple(
            &key,
            &[
                ("price", &final_price.round_dp(3).to_string()),
                ("created_at", &Utc::now().to_rfc3339()),
            ],
        )
        .await?;

    let last_history_entry: Option<String> = redis_conn
        .zrevrangebyscore_limit::<_, _, _, Vec<String>>(&history_key, "+inf", "-inf", 0, 1)
        .await?
        .into_iter()
        .next();
    
    let should_update = match last_history_entry {
        Some(entry) => {
            let parts: Vec<&str> = entry.split(':').collect();
            if parts.len() == 2 {
                parts[1].parse::<i64>()? < minute_timestamp
            } else {
                true
            }
        }
        None => true,
    };

    if should_update {
        let _: () = redis_conn
            .zadd(
                &history_key,
                &format!("{}:{}", final_price.round_dp(3), timestamp),
                minute_timestamp,
            )
            .await?;
    }
    let day_ago = Utc::now().timestamp() - 86400;
    let _: () = redis_conn
        .zrembyscore(&history_key, "-inf", day_ago)
        .await?;

    Ok(())
}

use crate::traits::redis::PriceInfo;
use chrono::Utc;
use entity::prelude::{Assets, Orders};
use entity::{assets, orders, price_snapshot, user_balances};
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
    
    let buy_orders: Vec<orders::Model> = Orders::find()
        .filter(orders::Column::AssetId.eq(asset_id))
        .filter(orders::Column::OrderType.eq("buy"))
        .filter(orders::Column::Status.eq("pending"))
        .all(db)
        .await?;

    let sell_orders: Vec<orders::Model> = Orders::find()
        .filter(orders::Column::AssetId.eq(asset_id))
        .filter(orders::Column::OrderType.eq("sell"))
        .filter(orders::Column::Status.eq("pending"))
        .all(db)
        .await?;
    
    let user_balances: Vec<user_balances::Model> = user_balances::Entity::find()
        .filter(user_balances::Column::AssetId.eq(asset_id))
        .all(db)
        .await?;

    let total_held_balance: Decimal = user_balances.iter().map(|b| b.amount).sum();
    
    let v_buy: Decimal = buy_orders.iter().map(|o| o.amount).sum::<Decimal>();
    let v_sell: Decimal = sell_orders.iter().map(|o| o.amount).sum::<Decimal>();
    
    let total_supply = v_buy + total_held_balance;

    let price_change = K.clone() * (v_buy - v_sell) / (total_supply + EPSILON.clone());
    let new_price = (old_price_value * (Decimal::from(1) + price_change)).round_dp(3);
    
    let key = format!("asset_price:{}", asset_id);
    let history_key = format!("asset_price_history:{}", asset_id);
    let timestamp = Utc::now().timestamp();

    let _: () = redis_conn
        .hset_multiple(
            &key,
            &[("price", &new_price.to_string()), ("created_at", &Utc::now().to_rfc3339())],
        )
        .await?;

    let _: () = redis_conn.zadd(&history_key, &new_price.to_string(), timestamp).await?;

    let day_ago = Utc::now().timestamp() - 86400;
    let _: () = redis_conn.zrembyscore(&history_key, "-inf", day_ago).await?;

    Ok(())
}

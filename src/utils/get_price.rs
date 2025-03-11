use redis::{AsyncCommands, Client};
use sea_orm::prelude::Decimal;
use std::error::Error;
use std::str::FromStr;

pub async fn get_price_by_asset_id(cache: &Client, asset_id: i32) -> Result<Decimal, Box<dyn Error + Send + Sync>> {
    let mut redis_conn = cache.get_multiplexed_async_connection().await?;

    let price_key = format!("asset_price:{}", asset_id);
    let price_str: Option<String> = redis_conn.hget(&price_key, "price").await?;

    if let Some(price_str) = price_str {
        Ok(Decimal::from_str(&price_str)?)
    } else {
        Err("Can't get price".into())
    }

}
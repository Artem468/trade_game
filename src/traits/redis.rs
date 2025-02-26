use std::collections::HashMap;
use std::str::FromStr;
use chrono::{DateTime, Utc};
use sea_orm::prelude::Decimal;

#[derive(Debug)]
pub struct PriceInfo {
    pub price: Option<Decimal>,
    pub created_at: Option<DateTime<Utc>>,
}

impl redis::FromRedisValue for PriceInfo {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        let map: redis::RedisResult<HashMap<String, String>> = redis::from_redis_value(v);
        match map {
            Ok(m) => Ok({
                PriceInfo {
                    price: m.get("price").and_then(|p| Decimal::from_str(p).ok()),
                    created_at: m
                        .get("created_at")
                        .and_then(|t| DateTime::parse_from_rfc3339(t).ok())
                        .map(|dt| dt.with_timezone(&Utc)),
                }
            }),
            Err(e) => Err(e),
        }
    }
}

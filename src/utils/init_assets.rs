use entity::assets;
use sea_orm::EntityTrait;
use sea_orm::{DbConn, Set};

pub async fn initialize_default_assets(db: &DbConn) -> Result<(), sea_orm::DbErr> {
    let assets = vec![
        assets::ActiveModel {
            symbol: Set("AAPL".to_string()),
            name: Set("Apple".to_string()),
            ..Default::default()
        },
        assets::ActiveModel {
            symbol: Set("BTC".to_string()),
            name: Set("Bitcoin".to_string()),
            ..Default::default()
        },
        assets::ActiveModel {
            symbol: Set("TSLA".to_string()),
            name: Set("Tesla".to_string()),
            ..Default::default()
        },
        assets::ActiveModel {
            symbol: Set("META".to_string()),
            name: Set("Meta".to_string()),
            ..Default::default()
        },
        assets::ActiveModel {
            symbol: Set("NVDA".to_string()),
            name: Set("Nvidia".to_string()),
            ..Default::default()
        },
        assets::ActiveModel {
            symbol: Set("MSFT".to_string()),
            name: Set("Microsoft".to_string()),
            ..Default::default()
        },
        assets::ActiveModel {
            symbol: Set("ETH".to_string()),
            name: Set("Ethereum".to_string()),
            ..Default::default()
        },
        assets::ActiveModel {
            symbol: Set("AMZN".to_string()),
            name: Set("Amazon".to_string()),
            ..Default::default()
        },
        assets::ActiveModel {
            symbol: Set("COLA".to_string()),
            name: Set("Cola".to_string()),
            ..Default::default()
        },
        assets::ActiveModel {
            symbol: Set("SOL".to_string()),
            name: Set("Solana".to_string()),
            ..Default::default()
        },
        assets::ActiveModel {
            symbol: Set("GOOGL".to_string()),
            name: Set("Google".to_string()),
            ..Default::default()
        },
        assets::ActiveModel {
            symbol: Set("YNDX".to_string()),
            name: Set("Yandex".to_string()),
            ..Default::default()
        },
    ];
    
    for asset in assets {
        let res = assets::Entity::insert(asset).exec(db).await;
        if res.is_err() {
            eprintln!("Error inserting asset: {:?}", res);
        }
    }

    Ok(())
}

use sea_orm::PaginatorTrait;
use entity::assets;
use sea_orm::{DbConn, Set};
use sea_orm::EntityTrait;

pub async fn initialize_assets(db: &DbConn) -> Result<(), sea_orm::DbErr> {
    let count = assets::Entity::find().count(db).await?;
    if count > 0 {
        println!("Assets table is not empty. Skipping seeding.");
        return Ok(());
    }
    
    let assets = vec![
        assets::ActiveModel {
            symbol: Set("AAPL".to_string()),
            name: Set("Apple".to_string()),
            ..Default::default()
        },
        assets::ActiveModel {
            symbol: Set("QSR".to_string()),
            name: Set("Burger King".to_string()),
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
        match assets::Entity::insert(asset).exec(db).await {
            Ok(_) => {}
            Err(_) => {}
        };
    }

    Ok(())
}

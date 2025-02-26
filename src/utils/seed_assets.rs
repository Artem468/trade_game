use entity::price_snapshot;
use sea_orm::{DbConn, EntityTrait};
use sea_orm::{PaginatorTrait, Set};

pub async fn seed_assets(db: &DbConn) -> Result<(), sea_orm::DbErr> {
    let count = price_snapshot::Entity::find().count(db).await?;
    if count > 0 {
        println!("Price Snapshot table is not empty. Skipping seeding.");
        return Ok(());
    }

    let assets_data = vec![
        price_snapshot::ActiveModel{
            asset_id: Set(1),
            price: Set(246),
            .. Default::default()
        },
        price_snapshot::ActiveModel{
            asset_id: Set(2),
            price: Set(96_530),
            .. Default::default()
        },
        price_snapshot::ActiveModel{
            asset_id: Set(3),
            price: Set(338),
            .. Default::default()
        },
        price_snapshot::ActiveModel{
            asset_id: Set(4),
            price: Set(684),
            .. Default::default()
        },
        price_snapshot::ActiveModel{
            asset_id: Set(5),
            price: Set(134),
            .. Default::default()
        },
        price_snapshot::ActiveModel{
            asset_id: Set(6),
            price: Set(408),
            .. Default::default()
        },
        price_snapshot::ActiveModel{
            asset_id: Set(7),
            price: Set(2737),
            .. Default::default()
        },
        price_snapshot::ActiveModel{
            asset_id: Set(8),
            price: Set(217),
            .. Default::default()
        },
        price_snapshot::ActiveModel{
            asset_id: Set(9),
            price: Set(71),
            .. Default::default()
        },
        price_snapshot::ActiveModel{
            asset_id: Set(10),
            price: Set(174),
            .. Default::default()
        },
        price_snapshot::ActiveModel{
            asset_id: Set(11),
            price: Set(182),
            .. Default::default()
        },
        price_snapshot::ActiveModel{
            asset_id: Set(12),
            price: Set(53),
            .. Default::default()
        },
    ];

    for active_asset in assets_data {
        price_snapshot::Entity::insert(active_asset).exec(db).await?;
    }

    Ok(())
}

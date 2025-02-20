use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Users::Id).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Users::Username).string().not_null().unique_key())
                    .col(ColumnDef::new(Users::Email).string().not_null().unique_key())
                    .col(ColumnDef::new(Users::HashedPassword).string().not_null())
                    .col(ColumnDef::new(Users::CreatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Assets::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Assets::Id).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Assets::Symbol).string().not_null().unique_key())
                    .col(ColumnDef::new(Assets::Name).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(UserBalances::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(UserBalances::Id).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(UserBalances::UserId).integer().not_null())
                    .col(ColumnDef::new(UserBalances::AssetId).integer().not_null())
                    .col(ColumnDef::new(UserBalances::Amount).decimal().not_null().default(0))
                    .foreign_key(ForeignKey::create().from(UserBalances::Table, UserBalances::UserId).to(Users::Table, Users::Id).on_delete(ForeignKeyAction::Cascade))
                    .foreign_key(ForeignKey::create().from(UserBalances::Table, UserBalances::AssetId).to(Assets::Table, Assets::Id).on_delete(ForeignKeyAction::Cascade))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Trades::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Trades::Id).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Trades::UserId).integer().not_null())
                    .col(ColumnDef::new(Trades::AssetId).integer().not_null())
                    .col(ColumnDef::new(Trades::TradeType).string().not_null())
                    .col(ColumnDef::new(Trades::Price).decimal().not_null())
                    .col(ColumnDef::new(Trades::Amount).decimal().not_null())
                    .col(ColumnDef::new(Trades::CreatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .foreign_key(ForeignKey::create().from(Trades::Table, Trades::UserId).to(Users::Table, Users::Id).on_delete(ForeignKeyAction::Cascade))
                    .foreign_key(ForeignKey::create().from(Trades::Table, Trades::AssetId).to(Assets::Table, Assets::Id).on_delete(ForeignKeyAction::Cascade))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Orders::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Orders::Id).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Orders::UserId).integer().not_null())
                    .col(ColumnDef::new(Orders::AssetId).integer().not_null())
                    .col(ColumnDef::new(Orders::OrderType).string().not_null())
                    .col(ColumnDef::new(Orders::OrderSide).string().not_null())
                    .col(ColumnDef::new(Orders::Price).decimal().null()) // NULL для рыночных ордеров
                    .col(ColumnDef::new(Orders::Amount).decimal().not_null())
                    .col(ColumnDef::new(Orders::Status).string().not_null())
                    .col(ColumnDef::new(Orders::CreatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Orders::UpdatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .foreign_key(ForeignKey::create().from(Orders::Table, Orders::UserId).to(Users::Table, Users::Id).on_delete(ForeignKeyAction::Cascade))
                    .foreign_key(ForeignKey::create().from(Orders::Table, Orders::AssetId).to(Assets::Table, Assets::Id).on_delete(ForeignKeyAction::Cascade))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Orders::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Trades::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(UserBalances::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Assets::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Users::Table).to_owned()).await?;
        Ok(())
    }
}


#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Username,
    Email,
    HashedPassword,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Assets {
    Table,
    Id,
    Symbol,
    Name,
}

#[derive(DeriveIden)]
enum UserBalances {
    Table,
    Id,
    UserId,
    AssetId,
    Amount,
}

#[derive(DeriveIden)]
enum Trades {
    Table,
    Id,
    UserId,
    AssetId,
    TradeType,
    Price,
    Amount,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Orders {
    Table,
    Id,
    UserId,
    AssetId,
    OrderType,
    OrderSide,
    Price,
    Amount,
    Status,
    CreatedAt,
    UpdatedAt,
}
use feel_sea_orm::user::entities::user::{Column as UserColumn, Entity as User};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(User)
            .if_not_exists()
            .col(
                ColumnDef::new(UserColumn::Id)
                    .big_integer()
                    .not_null()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(UserColumn::Uid)
                    .string()
                    .not_null()
                    .unique_key(),
            )
            .col(ColumnDef::new(UserColumn::Name).string().not_null())
            .col(ColumnDef::new(UserColumn::Avatar).string().not_null())
            .col(ColumnDef::new(UserColumn::Slogan).string().not_null())
            .col(ColumnDef::new(UserColumn::Enabled).boolean().not_null())
            .col(ColumnDef::new(UserColumn::CreatedAt).timestamp().not_null())
            .col(ColumnDef::new(UserColumn::UpdatedAt).timestamp().not_null())
            .to_owned();

        manager.create_table(table).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User).to_owned())
            .await
    }
}

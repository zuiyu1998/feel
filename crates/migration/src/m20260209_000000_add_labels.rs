use feel_sea_orm::label::entities::label::{Column as LabelColumn, Entity as Label};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(Label)
            .if_not_exists()
            .col(
                ColumnDef::new(LabelColumn::Id)
                    .big_integer()
                    .not_null()
                    .primary_key()
                    .auto_increment(),
            )
            .col(
                ColumnDef::new(LabelColumn::Name)
                    .string()
                    .not_null()
                    .unique_key(),
            )
            .col(
                ColumnDef::new(LabelColumn::Description)
                    .string()
                    .not_null()
                    .default(""),
            )
            .col(
                ColumnDef::new(LabelColumn::Remark)
                    .string()
                    .not_null()
                    .default(""),
            )
            .col(
                ColumnDef::new(LabelColumn::Influence)
                    .big_integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(LabelColumn::Enabled)
                    .boolean()
                    .not_null()
                    .default(true),
            )
            .col(
                ColumnDef::new(LabelColumn::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(LabelColumn::UpdatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .to_owned();

        manager.create_table(table).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Label).to_owned())
            .await
    }
}

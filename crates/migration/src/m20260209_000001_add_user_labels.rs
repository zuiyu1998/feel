use feel_sea_orm::label::entities::user_label::{Column as UserLabelColumn, Entity as UserLabel};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(UserLabel)
            .if_not_exists()
            .col(
                ColumnDef::new(UserLabelColumn::Id)
                    .big_integer()
                    .not_null()
                    .primary_key()
                    .auto_increment(),
            )
            .col(
                ColumnDef::new(UserLabelColumn::UserId)
                    .big_integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(UserLabelColumn::LabelId)
                    .big_integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(UserLabelColumn::Enabled)
                    .boolean()
                    .not_null()
                    .default(true),
            )
            .col(
                ColumnDef::new(UserLabelColumn::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(UserLabelColumn::UpdatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .index(
                &mut Index::create()
                    .name("idx_user_label_user_id_label_id")
                    .col(UserLabelColumn::UserId)
                    .col(UserLabelColumn::LabelId)
                    .unique()
                    .to_owned(),
            )
            .index(
                &mut Index::create()
                    .name("idx_user_label_label_id")
                    .col(UserLabelColumn::LabelId)
                    .to_owned(),
            )
            .to_owned();

        manager.create_table(table).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserLabel).to_owned())
            .await
    }
}

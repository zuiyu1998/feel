use feel_sea_orm::content::entities::content::{Column as ContentColumn, Entity as Content};
use sea_orm_migration::prelude::*;

/// 为 `content` 表增加浏览量字段（feed 热度排序依据）。
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Content)
                    .add_column(
                        ColumnDef::new(ContentColumn::ViewCount)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_content_view_created")
                    .table(Content)
                    .col(ContentColumn::ViewCount)
                    .col(ContentColumn::CreatedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_content_view_created")
                    .table(Content)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Content)
                    .drop_column(ContentColumn::ViewCount)
                    .to_owned(),
            )
            .await
    }
}

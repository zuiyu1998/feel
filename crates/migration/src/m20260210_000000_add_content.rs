use feel_sea_orm::content::entities::{
    content::{Column as ContentColumn, Entity as Content},
    content_article::{Column as ContentArticleColumn, Entity as ContentArticle},
    content_video::{Column as ContentVideoColumn, Entity as ContentVideo},
};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // -- content 基表（公共字段），id 由数据库序列自增 --
        let content = Table::create()
            .table(Content)
            .if_not_exists()
            .col(
                ColumnDef::new(ContentColumn::Id)
                    .big_integer()
                    .not_null()
                    .primary_key()
                    .auto_increment(),
            )
            .col(
                ColumnDef::new(ContentColumn::UserId)
                    .big_integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(ContentColumn::ContentType)
                    .string()
                    .not_null(),
            )
            .col(ColumnDef::new(ContentColumn::Title).string().not_null())
            .col(
                ColumnDef::new(ContentColumn::Enabled)
                    .boolean()
                    .not_null()
                    .default(true),
            )
            .col(
                ColumnDef::new(ContentColumn::IsDelete)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .col(
                ColumnDef::new(ContentColumn::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(ContentColumn::UpdatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .to_owned();

        manager.create_table(content).await?;

        // 非唯一索引不能作为表内约束(CONSTRAINT 必须指定类型,如 UNIQUE),
        // 需在建表后单独创建 CREATE INDEX
        manager
            .create_index(
                Index::create()
                    .name("idx_content_user_created")
                    .table(Content)
                    .col(ContentColumn::UserId)
                    .col(ContentColumn::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_content_type_created")
                    .table(Content)
                    .col(ContentColumn::ContentType)
                    .col(ContentColumn::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // -- 文章明细表（1:1 关联 content.id）--
        let article = Table::create()
            .table(ContentArticle)
            .if_not_exists()
            .col(
                ColumnDef::new(ContentArticleColumn::ContentId)
                    .big_integer()
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new(ContentArticleColumn::Body).text().not_null())
            .to_owned();

        manager.create_table(article).await?;

        // -- 视频明细表（1:1 关联 content.id）--
        let video = Table::create()
            .table(ContentVideo)
            .if_not_exists()
            .col(
                ColumnDef::new(ContentVideoColumn::ContentId)
                    .big_integer()
                    .not_null()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(ContentVideoColumn::VideoUrl)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(ContentVideoColumn::CoverUrl)
                    .string()
                    .not_null()
                    .default(""),
            )
            .col(
                ColumnDef::new(ContentVideoColumn::DurationSecs)
                    .integer()
                    .not_null()
                    .default(0),
            )
            .to_owned();

        manager.create_table(video).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ContentVideo).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ContentArticle).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Content).to_owned())
            .await
    }
}

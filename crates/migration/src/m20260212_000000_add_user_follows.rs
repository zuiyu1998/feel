use feel_sea_orm::follow::entities::user_follow::{
    Column as UserFollowColumn, Entity as UserFollow,
};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // -- user_follow 关注关系表（follower → followee）--
        let table = Table::create()
            .table(UserFollow)
            .if_not_exists()
            .col(
                ColumnDef::new(UserFollowColumn::Id)
                    .big_integer()
                    .not_null()
                    .primary_key()
                    .auto_increment(),
            )
            .col(
                ColumnDef::new(UserFollowColumn::UserId)
                    .big_integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(UserFollowColumn::FollowUserId)
                    .big_integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(UserFollowColumn::Enabled)
                    .boolean()
                    .not_null()
                    .default(true),
            )
            .col(
                ColumnDef::new(UserFollowColumn::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(UserFollowColumn::UpdatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .to_owned();

        manager.create_table(table).await?;

        // 非唯一索引不能作为表内约束(CONSTRAINT 必须指定类型,如 UNIQUE),
        // 需在建表后单独创建 CREATE INDEX
        manager
            .create_index(
                Index::create()
                    .name("idx_user_follow_pair")
                    .table(UserFollow)
                    .col(UserFollowColumn::UserId)
                    .col(UserFollowColumn::FollowUserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // 粉丝列表按 follow_user_id 查询
        manager
            .create_index(
                Index::create()
                    .name("idx_user_follow_follow")
                    .table(UserFollow)
                    .col(UserFollowColumn::FollowUserId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserFollow).to_owned())
            .await
    }
}

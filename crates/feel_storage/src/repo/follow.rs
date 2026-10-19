use crate::{Result, repo::UserFollowRepo};
use async_trait::async_trait;
use chrono::Local;
use feel_entity::follow::{UserFollow, UserFollowCreate};
use feel_sea_orm::follow::entities::prelude::*;
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{NotSet, Set},
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use std::collections::HashMap;

pub struct SeaOrmUserFollowRepo {
    /// Sea-orm database connection
    pub conn: DatabaseConnection,
}

impl SeaOrmUserFollowRepo {
    pub fn new(conn: DatabaseConnection) -> Self {
        SeaOrmUserFollowRepo { conn }
    }
}

#[async_trait]
impl UserFollowRepo for SeaOrmUserFollowRepo {
    async fn create_user_follow(&self, create: &UserFollowCreate) -> Result<UserFollow> {
        let now = Local::now();

        let active = UserFollowActiveModel {
            id: NotSet, // database auto-increment
            user_id: Set(create.user_id),
            follow_user_id: Set(create.follow_user_id),
            enabled: Set(true),
            created_at: Set(now.to_utc()),
            updated_at: Set(now.to_utc()),
        };

        let model: UserFollowModel = active.insert(&self.conn).await?;
        Ok(model.into())
    }

    async fn find_user_follow(
        &self,
        user_id: i64,
        follow_user_id: i64,
    ) -> Result<Option<UserFollow>> {
        let model = UserFollowEntity::find()
            .filter(UserFollowColumn::UserId.eq(user_id))
            .filter(UserFollowColumn::FollowUserId.eq(follow_user_id))
            .one(&self.conn)
            .await?;
        Ok(model.map(Into::into))
    }

    async fn delete_user_follow(&self, user_id: i64, follow_user_id: i64) -> Result<()> {
        UserFollowEntity::delete_many()
            .filter(UserFollowColumn::UserId.eq(user_id))
            .filter(UserFollowColumn::FollowUserId.eq(follow_user_id))
            .exec(&self.conn)
            .await?;
        Ok(())
    }

    async fn find_followings(
        &self,
        user_id: i64,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<UserFollow>> {
        let models = UserFollowEntity::find()
            .filter(UserFollowColumn::UserId.eq(user_id))
            .order_by_desc(UserFollowColumn::CreatedAt)
            .offset(offset)
            .limit(limit)
            .all(&self.conn)
            .await?;
        Ok(models.into_iter().map(Into::into).collect())
    }

    async fn find_followers(
        &self,
        user_id: i64,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<UserFollow>> {
        let models = UserFollowEntity::find()
            .filter(UserFollowColumn::FollowUserId.eq(user_id))
            .order_by_desc(UserFollowColumn::CreatedAt)
            .offset(offset)
            .limit(limit)
            .all(&self.conn)
            .await?;
        Ok(models.into_iter().map(Into::into).collect())
    }

    async fn count_followings(&self, user_id: i64) -> Result<u64> {
        let count = UserFollowEntity::find()
            .filter(UserFollowColumn::UserId.eq(user_id))
            .count(&self.conn)
            .await?;
        Ok(count)
    }

    async fn count_followers(&self, user_id: i64) -> Result<u64> {
        let count = UserFollowEntity::find()
            .filter(UserFollowColumn::FollowUserId.eq(user_id))
            .count(&self.conn)
            .await?;
        Ok(count)
    }

    async fn find_follower_ids(&self, user_id: i64) -> Result<Vec<i64>> {
        let models = UserFollowEntity::find()
            .filter(UserFollowColumn::FollowUserId.eq(user_id))
            .all(&self.conn)
            .await?;
        Ok(models.into_iter().map(|model| model.user_id).collect())
    }

    async fn find_following_ids(&self, user_id: i64) -> Result<Vec<i64>> {
        let models = UserFollowEntity::find()
            .filter(UserFollowColumn::UserId.eq(user_id))
            .all(&self.conn)
            .await?;
        Ok(models
            .into_iter()
            .map(|model| model.follow_user_id)
            .collect())
    }

    async fn count_followers_bulk(&self, user_ids: &[i64]) -> Result<HashMap<i64, u64>> {
        let mut counts: HashMap<i64, u64> = HashMap::new();
        if user_ids.is_empty() {
            return Ok(counts);
        }

        let models = UserFollowEntity::find()
            .filter(UserFollowColumn::FollowUserId.is_in(user_ids.to_vec()))
            .all(&self.conn)
            .await?;
        for model in models {
            *counts.entry(model.follow_user_id).or_insert(0) += 1;
        }
        Ok(counts)
    }
}

use async_trait::async_trait;

use crate::database::{DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE};
use crate::{Error, Result, repo::UserFollowRepo};
use feel_entity::prelude::*;

pub struct CommonUserFollowDataBase {
    follow_repo: Box<dyn UserFollowRepo>,
}

impl CommonUserFollowDataBase {
    pub fn new<T: UserFollowRepo>(follow_repo: T) -> Self {
        CommonUserFollowDataBase {
            follow_repo: Box::new(follow_repo),
        }
    }

    /// 将可选分页参数归一化为 (offset, page, page_size)。
    fn paginate(page: Option<u64>, page_size: Option<u64>) -> (u64, u64, u64) {
        let page = page.unwrap_or(1).max(1);
        let size = page_size
            .unwrap_or(DEFAULT_PAGE_SIZE)
            .clamp(1, MAX_PAGE_SIZE);
        ((page - 1) * size, page, size)
    }
}

#[async_trait]
pub trait UserFollowDataBase: 'static + Send + Sync {
    /// 关注（幂等：已关注时返回既有关系；不允许关注自己）
    async fn follow(&self, follower_id: i64, followee_id: i64) -> Result<UserFollow>;

    /// 取关（幂等：未关注时静默成功）
    async fn unfollow(&self, follower_id: i64, followee_id: i64) -> Result<()>;

    /// 关注统计（`viewer_id` 为 Some 且非本人时附带 `is_following`）
    async fn get_stats(&self, user_id: i64, viewer_id: Option<i64>) -> Result<FollowStats>;

    /// 分页查询某用户的关注列表（按创建时间倒序）
    async fn list_followings(
        &self,
        user_id: i64,
        page: Option<u64>,
        page_size: Option<u64>,
    ) -> Result<FollowPage>;

    /// 分页查询某用户的粉丝列表（按创建时间倒序）
    async fn list_followers(
        &self,
        user_id: i64,
        page: Option<u64>,
        page_size: Option<u64>,
    ) -> Result<FollowPage>;
}

#[async_trait]
impl UserFollowDataBase for CommonUserFollowDataBase {
    async fn follow(&self, follower_id: i64, followee_id: i64) -> Result<UserFollow> {
        if follower_id == followee_id {
            return Err(Error::Business(
                "Cannot follow your own account".to_string(),
            ));
        }

        // 幂等：已存在则直接返回既有关系
        if let Some(existing) = self
            .follow_repo
            .find_user_follow(follower_id, followee_id)
            .await?
        {
            return Ok(existing);
        }

        let create = UserFollowCreate {
            user_id: follower_id,
            follow_user_id: followee_id,
        };
        self.follow_repo.create_user_follow(&create).await
    }

    async fn unfollow(&self, follower_id: i64, followee_id: i64) -> Result<()> {
        // 幂等：未关注时静默成功
        self.follow_repo
            .delete_user_follow(follower_id, followee_id)
            .await
    }

    async fn get_stats(&self, user_id: i64, viewer_id: Option<i64>) -> Result<FollowStats> {
        let follower_count = self.follow_repo.count_followers(user_id).await?;
        let following_count = self.follow_repo.count_followings(user_id).await?;

        let is_following = match viewer_id {
            Some(viewer) if viewer != user_id => Some(
                self.follow_repo
                    .find_user_follow(viewer, user_id)
                    .await?
                    .is_some(),
            ),
            _ => None,
        };

        Ok(FollowStats {
            user_id,
            follower_count,
            following_count,
            is_following,
        })
    }

    async fn list_followings(
        &self,
        user_id: i64,
        page: Option<u64>,
        page_size: Option<u64>,
    ) -> Result<FollowPage> {
        let (offset, page, page_size) = Self::paginate(page, page_size);
        let items = self
            .follow_repo
            .find_followings(user_id, offset, page_size)
            .await?;
        let total = self.follow_repo.count_followings(user_id).await?;
        Ok(FollowPage {
            items,
            total,
            page,
            page_size,
        })
    }

    async fn list_followers(
        &self,
        user_id: i64,
        page: Option<u64>,
        page_size: Option<u64>,
    ) -> Result<FollowPage> {
        let (offset, page, page_size) = Self::paginate(page, page_size);
        let items = self
            .follow_repo
            .find_followers(user_id, offset, page_size)
            .await?;
        let total = self.follow_repo.count_followers(user_id).await?;
        Ok(FollowPage {
            items,
            total,
            page,
            page_size,
        })
    }
}

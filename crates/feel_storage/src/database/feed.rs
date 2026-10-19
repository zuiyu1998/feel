use async_trait::async_trait;

use std::collections::HashMap;

use crate::Result;
use crate::cache::feed::UserFeedCache;
use crate::database::{DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE};
use crate::repo::{ContentRepo, UserFollowRepo};
use feel_entity::content::Content;
use feel_entity::feed::FollowingFeedPage;

/// 粉丝数 <= 该值的作者采用「推」模式（写扩散：扇出到粉丝收件箱）。
pub const PUSH_FANOUT_LIMIT: u64 = 5_000;
/// 读「推」窗口大小（收件箱最新内容条数）。
pub const INBOX_READ_LIMIT: u64 = 300;
/// 拉模式合并的大 V 作者上限。
const PULL_AUTHOR_LIMIT: usize = 50;
/// 拉模式拉取的大 V 内容窗口。
const PULL_CONTENT_LIMIT: u64 = 200;

pub struct CommonUserFeedDataBase {
    feed_cache: Box<dyn UserFeedCache>,
    follow_repo: Box<dyn UserFollowRepo>,
    content_repo: Box<dyn ContentRepo>,
}

impl CommonUserFeedDataBase {
    pub fn new<C: UserFeedCache, F: UserFollowRepo, R: ContentRepo>(
        feed_cache: C,
        follow_repo: F,
        content_repo: R,
    ) -> Self {
        CommonUserFeedDataBase {
            feed_cache: Box::new(feed_cache),
            follow_repo: Box::new(follow_repo),
            content_repo: Box::new(content_repo),
        }
    }

    fn paginate(page: Option<u64>, page_size: Option<u64>) -> (u64, u64, u64) {
        let page = page.unwrap_or(1).max(1);
        let size = page_size
            .unwrap_or(DEFAULT_PAGE_SIZE)
            .clamp(1, MAX_PAGE_SIZE);
        ((page - 1) * size, page, size)
    }
}

#[async_trait]
pub trait UserFeedDataBase: 'static + Send + Sync {
    /// 内容发布扇出（推拉结合）：
    /// 粉丝数未超 `PUSH_FANOUT_LIMIT` → 写入所有粉丝的 Redis 收件箱；
    /// 超过阈值（大 V）→ 跳过推送，由读模式拉取。
    async fn publish(&self, author_id: i64, content_id: i64) -> Result<()>;

    /// 内容删除后从粉丝收件箱回收（best-effort）。
    async fn remove_content(&self, author_id: i64, content_id: i64) -> Result<()>;

    /// 获取「我关注的人」的 Feed（推 + 拉合并，按内容 id 倒序）。
    async fn get_following_feed(
        &self,
        user_id: i64,
        page: Option<u64>,
        page_size: Option<u64>,
    ) -> Result<FollowingFeedPage>;
}

#[async_trait]
impl UserFeedDataBase for CommonUserFeedDataBase {
    async fn publish(&self, author_id: i64, content_id: i64) -> Result<()> {
        let follower_count = self.follow_repo.count_followers(author_id).await?;

        // 大 V 或无人关注：不写扩散（读时拉取 / 无读者）
        if follower_count == 0 || follower_count > PUSH_FANOUT_LIMIT {
            return Ok(());
        }

        let follower_ids = self.follow_repo.find_follower_ids(author_id).await?;
        self.feed_cache
            .push_to_inboxes(&follower_ids, content_id)
            .await
    }

    async fn remove_content(&self, author_id: i64, content_id: i64) -> Result<()> {
        let follower_ids = self.follow_repo.find_follower_ids(author_id).await?;
        if follower_ids.is_empty() {
            return Ok(());
        }
        self.feed_cache
            .remove_from_inboxes(&follower_ids, content_id)
            .await
    }

    async fn get_following_feed(
        &self,
        user_id: i64,
        page: Option<u64>,
        page_size: Option<u64>,
    ) -> Result<FollowingFeedPage> {
        let (offset, page, page_size) = Self::paginate(page, page_size);

        // 1. 推模式：读自己的收件箱（内容 id 倒序）
        let mut ids = self
            .feed_cache
            .read_inbox(user_id, INBOX_READ_LIMIT)
            .await?;

        // 2. 拉模式：找出我关注的「大 V」作者，拉取其最近内容
        let following_ids = self.follow_repo.find_following_ids(user_id).await?;
        let pulled: Vec<Content> = if following_ids.is_empty() {
            Vec::new()
        } else {
            let counts = self
                .follow_repo
                .count_followers_bulk(&following_ids)
                .await?;
            let mut celebrities: Vec<i64> = following_ids
                .iter()
                .filter(|id| counts.get(*id).copied().unwrap_or(0) > PUSH_FANOUT_LIMIT)
                .copied()
                .collect();
            celebrities.sort_unstable();
            celebrities.truncate(PULL_AUTHOR_LIMIT);

            if celebrities.is_empty() {
                Vec::new()
            } else {
                self.content_repo
                    .find_recent_by_users(&celebrities, 0, PULL_CONTENT_LIMIT)
                    .await?
            }
        };

        // 3. 推拉合并（内容 id 单调递增 ≈ 时间序，倒序去重）
        let mut pulled_map: HashMap<i64, Content> = pulled
            .into_iter()
            .map(|content| (content.base.id, content))
            .collect();
        for id in pulled_map.keys() {
            ids.push(*id);
        }
        ids.sort_unstable_by(|a, b| b.cmp(a));
        ids.dedup();

        // 4. 分页切片
        let start = offset as usize;
        let end = (start + page_size as usize).min(ids.len());
        let has_more = end < ids.len();

        let mut items: Vec<Content> = Vec::new();
        if start < ids.len() {
            for &id in &ids[start..end] {
                if let Some(content) = pulled_map.remove(&id) {
                    items.push(content);
                } else if let Some(content) = self.content_repo.find_by_id(id).await? {
                    // 已删除/禁用的内容从流中过滤
                    if content.base.enabled && !content.base.is_delete {
                        items.push(content);
                    }
                }
            }
        }

        Ok(FollowingFeedPage {
            items,
            page,
            page_size,
            has_more,
        })
    }
}

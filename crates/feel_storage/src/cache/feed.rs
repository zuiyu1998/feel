use crate::Result;
use async_trait::async_trait;
use redis::{Client, RedisResult};

/// 每个用户收件箱保留的最大条目数（按内容 id 倒序保留最新部分）。
pub const FEED_INBOX_MAX: u64 = 500;
/// 单次 pipeline 扇出的粉丝数批量大小。
const FANOUT_BATCH: usize = 500;
/// 收件箱键前缀。
const REDIS_KEY_INBOX_PREFIX: &str = "feel:feed:inbox:";

pub struct CommonUserFeedCache {
    client: Client,
}

impl CommonUserFeedCache {
    pub fn new(client: Client) -> Self {
        CommonUserFeedCache { client }
    }

    async fn get_connection(&self) -> RedisResult<redis::aio::MultiplexedConnection> {
        self.client.get_multiplexed_tokio_connection().await
    }
}

/// 收件箱键：`feel:feed:inbox:{user_id}`（ZSET，score = content_id）。
fn inbox_key(user_id: i64) -> String {
    format!("{REDIS_KEY_INBOX_PREFIX}{user_id}")
}

#[async_trait]
impl UserFeedCache for CommonUserFeedCache {
    async fn push_to_inboxes(&self, follower_ids: &[i64], content_id: i64) -> Result<()> {
        if follower_ids.is_empty() {
            return Ok(());
        }

        let mut conn = self.get_connection().await?;
        for chunk in follower_ids.chunks(FANOUT_BATCH) {
            let mut pipe = redis::pipe();
            for &user_id in chunk {
                let key = inbox_key(user_id);
                // ZADD key score member（score = content_id：BIGSERIAL 单调递增，天然时间序）
                pipe.cmd("ZADD")
                    .arg(&key)
                    .arg(content_id)
                    .arg(content_id)
                    .ignore();
                // 仅保留最新 FEED_INBOX_MAX 条，控制内存
                pipe.cmd("ZREMRANGEBYRANK")
                    .arg(&key)
                    .arg(0)
                    .arg(-(FEED_INBOX_MAX as i64 + 1))
                    .ignore();
            }
            pipe.query_async::<_, ()>(&mut conn).await?;
        }
        Ok(())
    }

    async fn read_inbox(&self, user_id: i64, limit: u64) -> Result<Vec<i64>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let mut conn = self.get_connection().await?;
        let members: Vec<String> = redis::cmd("ZREVRANGE")
            .arg(inbox_key(user_id))
            .arg(0)
            .arg(limit as i64 - 1)
            .query_async(&mut conn)
            .await?;
        Ok(members
            .into_iter()
            .filter_map(|member| member.parse::<i64>().ok())
            .collect())
    }

    async fn remove_from_inboxes(&self, follower_ids: &[i64], content_id: i64) -> Result<()> {
        if follower_ids.is_empty() {
            return Ok(());
        }

        let mut conn = self.get_connection().await?;
        for chunk in follower_ids.chunks(FANOUT_BATCH) {
            let mut pipe = redis::pipe();
            for &user_id in chunk {
                pipe.cmd("ZREM")
                    .arg(inbox_key(user_id))
                    .arg(content_id)
                    .ignore();
            }
            pipe.query_async::<_, ()>(&mut conn).await?;
        }
        Ok(())
    }
}

#[async_trait]
pub trait UserFeedCache: 'static + Send + Sync {
    /// 将内容 id 推入一批粉丝的收件箱（pipeline 批量，按 inbox 上限裁剪）。
    async fn push_to_inboxes(&self, follower_ids: &[i64], content_id: i64) -> Result<()>;

    /// 读取某用户收件箱的最新内容 id（按 id 倒序，最多 `limit` 条）。
    async fn read_inbox(&self, user_id: i64, limit: u64) -> Result<Vec<i64>>;

    /// 从一批粉丝的收件箱中移除内容 id（内容删除后回收，best-effort）。
    async fn remove_from_inboxes(&self, follower_ids: &[i64], content_id: i64) -> Result<()>;
}

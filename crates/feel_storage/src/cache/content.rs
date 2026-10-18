use crate::Result;
use async_trait::async_trait;
use feel_entity::content::{ContentPage, FeedSort};
use redis::{Client, RedisResult};
use serde_json;

/// Feed 分页缓存 TTL（秒）。
pub const FEED_CACHE_TTL_SECS: u64 = 60;

/// Feed 缓存版本号键：内容写操作时 INCR，使全部旧缓存逻辑上失效（O(1)，无需 SCAN 删除）。
const REDIS_KEY_FEED_VERSION: &str = "feel:content:feed:version";
/// 单条 feed 分页缓存键前缀（含版本号，格式：`prefix{version}:{type}:{sort}:{page}:{size}`）。
const REDIS_KEY_FEED_PREFIX: &str = "feel:content:feed:v";

pub struct CommonContentCache {
    client: Client,
}

impl CommonContentCache {
    pub fn new(client: Client) -> Self {
        CommonContentCache { client }
    }

    async fn get_connection(&self) -> RedisResult<redis::aio::MultiplexedConnection> {
        self.client.get_multiplexed_tokio_connection().await
    }

    /// 读取当前 feed 缓存版本号（无则视为 0）。
    async fn feed_version(&self, conn: &mut redis::aio::MultiplexedConnection) -> RedisResult<u64> {
        let version: Option<u64> = redis::cmd("GET")
            .arg(REDIS_KEY_FEED_VERSION)
            .query_async(conn)
            .await?;
        Ok(version.unwrap_or(0))
    }

    /// 构造 feed 分页缓存键。
    fn feed_key(
        content_type: Option<&str>,
        sort: FeedSort,
        page: u64,
        page_size: u64,
        version: u64,
    ) -> String {
        let type_key = content_type.unwrap_or("all");
        let sort_key = match sort {
            FeedSort::Latest => "latest",
            FeedSort::Popular => "popular",
        };
        format!("{REDIS_KEY_FEED_PREFIX}{version}:{type_key}:{sort_key}:{page}:{page_size}")
    }
}

#[async_trait]
impl ContentCache for CommonContentCache {
    async fn get_feed(
        &self,
        content_type: Option<&str>,
        sort: FeedSort,
        page: u64,
        page_size: u64,
    ) -> Result<Option<ContentPage>> {
        let mut conn = self.get_connection().await?;
        let version = self.feed_version(&mut conn).await?;
        let key = Self::feed_key(content_type, sort, page, page_size, version);

        let json: Option<String> = redis::cmd("GET").arg(&key).query_async(&mut conn).await?;
        match json {
            Some(json) => Ok(Some(serde_json::from_str(&json)?)),
            None => Ok(None),
        }
    }

    async fn set_feed(
        &self,
        content_type: Option<&str>,
        sort: FeedSort,
        page: u64,
        page_size: u64,
        content_page: &ContentPage,
    ) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let version = self.feed_version(&mut conn).await?;
        let key = Self::feed_key(content_type, sort, page, page_size, version);

        let json = serde_json::to_string(content_page)?;
        redis::cmd("SET")
            .arg(&key)
            .arg(json)
            .arg("EX")
            .arg(FEED_CACHE_TTL_SECS)
            .query_async::<_, ()>(&mut conn)
            .await?;
        Ok(())
    }

    async fn invalidate_feed(&self) -> Result<()> {
        let mut conn = self.get_connection().await?;
        redis::cmd("INCR")
            .arg(REDIS_KEY_FEED_VERSION)
            .query_async::<_, ()>(&mut conn)
            .await?;
        Ok(())
    }
}

#[async_trait]
pub trait ContentCache: 'static + Send + Sync {
    /// 获取缓存的 feed 分页（命中返回 `Some`，未命中返回 `None`）。
    async fn get_feed(
        &self,
        content_type: Option<&str>,
        sort: FeedSort,
        page: u64,
        page_size: u64,
    ) -> Result<Option<ContentPage>>;

    /// 写入 feed 分页缓存（键含版本号，TTL = `FEED_CACHE_TTL_SECS`）。
    async fn set_feed(
        &self,
        content_type: Option<&str>,
        sort: FeedSort,
        page: u64,
        page_size: u64,
        content_page: &ContentPage,
    ) -> Result<()>;

    /// 使全部 feed 缓存失效（内容写操作后调用；INCR 版本号，O(1)）。
    async fn invalidate_feed(&self) -> Result<()>;
}

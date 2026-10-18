# ContentCache — Feed 分页缓存接口

## 概述

`ContentCache` 是 `feel_storage` crate 中定义的 **Feed 分页缓存层 trait**，位于 `crates/feel_storage/src/cache/content.rs`。

它抽象了对全站 feed 分页结果（`ContentPage`）的 Redis 缓存读写与失效，使 `CommonContentDataBase::list_feed` 可以走 Cache-Aside 模式：先查缓存、未命中回源数据库并回填。默认实现 `CommonContentCache` 基于 Redis 字符串键 + **版本号失效**策略。

## 模块位置

```
crates/feel_storage/src/cache/
├── mod.rs         # 模块声明（user / content）
├── user.rs        # UserCache trait + CommonUserCache
└── content.rs     # ContentCache trait + CommonContentCache ←
```

## Trait 定义

```rust
#[async_trait]
pub trait ContentCache: 'static + Send + Sync {
    /// 获取缓存的 feed 分页（命中返回 Some，未命中返回 None）
    async fn get_feed(&self, content_type: Option<&str>, sort: FeedSort, page: u64, page_size: u64)
        -> Result<Option<ContentPage>>;

    /// 写入 feed 分页缓存（键含版本号，TTL = FEED_CACHE_TTL_SECS）
    async fn set_feed(&self, content_type: Option<&str>, sort: FeedSort, page: u64, page_size: u64,
        content_page: &ContentPage) -> Result<()>;

    /// 使全部 feed 缓存失效（内容写操作后调用；INCR 版本号，O(1)）
    async fn invalidate_feed(&self) -> Result<()>;
}
```

| 方法 | 说明 |
|------|------|
| `get_feed` | 读取 `(type, sort, page, size)` 维度的分页缓存 |
| `set_feed` | 写入分页缓存（`SET key value EX 60`） |
| `invalidate_feed` | 写操作后使全部旧缓存失效（版本号 +1） |

## 缓存键与失效设计

```
键：feel:content:feed:v{version}:{type}:{sort}:{page}:{size}
   version  — 当前 feed 缓存版本号（feel:content:feed:version，GET/INCR）
   type     — "article" | "video" | "all"
   sort     — "latest" | "popular"
值：JSON(ContentPage)（ContentBase + ContentKind 均可 serde 序列化）
TTL：FEED_CACHE_TTL_SECS = 60 秒
```

**失效策略（版本号，O(1)）**：

- 内容写操作（create / update / remove）成功后执行 `INCR feel:content:feed:version`
- 新请求按新版本构造键，旧版本缓存**逻辑上失效**，无需 `SCAN` 删除，由 TTL 自然过期回收
- **浏览量（record_view）不主动失效**：热门排序的微小漂移由 60s TTL 收敛，避免高频浏览量写放大缓存

## 默认实现：CommonContentCache

```rust
pub struct CommonContentCache {
    client: redis::Client,
}

impl CommonContentCache {
    pub fn new(client: Client) -> Self
    async fn get_connection(&self) -> RedisResult<redis::aio::MultiplexedConnection>
    async fn feed_version(&self, conn: &mut MultiplexedConnection) -> RedisResult<u64>
    fn feed_key(content_type: Option<&str>, sort: FeedSort, page: u64, page_size: u64, version: u64) -> String
}
```

### 读流程（get_feed）

```
1. 获取 Redis 异步连接
2. GET feel:content:feed:version → 当前版本 v（无则 0）
3. GET feed_key(..., v) → 命中则 JSON 反序列化为 ContentPage 返回 Some
4. 未命中返回 None
```

### 写流程（set_feed）

```
1. 获取连接，读当前版本 v
2. serde_json::to_string(&ContentPage)
3. SET feed_key(..., v) <json> EX 60
```

## 架构关系

```
Handler（feel_api::content::content_feed）
    ↓
ContentDataBase（CommonContentDataBase::list_feed —— Cache-Aside）
    ├─ 命中 → 直接返回缓存 ContentPage
    └─ 未命中 → ContentRepo 回源 → set_feed 回填
    ↓
ContentCache（CommonContentCache）── Redis（版本号 + 分页键，TTL 60s）
```

### 依赖关系

| 依赖 | 用途 |
|------|------|
| `redis` crate | Redis 客户端 + 异步连接 |
| `serde_json` | `ContentPage` 序列化/反序列化 |
| `feel_entity` | `ContentPage` / `FeedSort` 数据模型 |

## 错误处理

方法返回 `crate::Result<T>`（`Error::Redis` / `Error::Json`）。**降级策略**（由 `CommonContentDataBase` 实施）：

- `get_feed` 失败 → 视为未命中回源数据库（不影响请求成功）
- `set_feed` / `invalidate_feed` 失败 → best-effort 忽略（旧缓存按 TTL 自然过期）

## 使用示例

```rust
use feel_storage::cache::content::{CommonContentCache, FEED_CACHE_TTL_SECS};
use feel_storage::database::{CommonContentDataBase, ContentDataBase};

let cache = CommonContentCache::new(redis_client);
let db = CommonContentDataBase::new(SeaOrmContentRepo::new(conn), cache);

// list_feed 内部：缓存命中返回 / 未命中回源并回填（详见 ContentDataBase）
let page = db.list_feed(None, FeedSort::Latest, Some(1), Some(20)).await?;
```

## 当前状态

| 方法 | 实现状态 | 说明 |
|------|----------|------|
| `get_feed` | ✅ 完整 | 版本号键 + JSON 反序列化 |
| `set_feed` | ✅ 完整 | `SET ... EX 60` |
| `invalidate_feed` | ✅ 完整 | `INCR` 版本号（O(1)） |
| 接入 `list_feed` | ✅ 完整 | Cache-Aside + 降级 |

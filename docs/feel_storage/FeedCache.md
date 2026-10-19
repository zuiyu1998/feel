# FeedCache — 关注流收件箱缓存（Redis ZSET）

## 概述

`UserFeedCache` 是 `feel_storage` 中定义的**关注流收件箱缓存接口**（位于 `crates/feel_storage/src/cache/feed.rs`），实现推拉结合的「推」侧：把内容 id 扇出写入粉丝的 Redis ZSET 收件箱，并在读取时取出最新内容 id。

默认实现 `CommonUserFeedCache` 基于 Redis ZSET（score = content_id，member = content_id）。

## 缓存键与结构

```
键：feel:feed:inbox:{user_id}      （ZSET）
  score  = content_id              （BIGSERIAL 单调递增 ≈ 时间序）
  member = content_id
  TTL    ：无（由裁剪控制内存，每箱最多 FEED_INBOX_MAX = 500 条）
读：ZREVRANGE key 0 (limit-1) → 最新内容 id（倒序）
写：ZADD key content_id content_id + ZREMRANGEBYRANK key 0 -(FEED_INBOX_MAX+1)
```

## Trait 定义

```rust
#[async_trait]
pub trait UserFeedCache: 'static + Send + Sync {
    /// 将内容 id 推入一批粉丝的收件箱（pipeline 批量，按上限裁剪）
    async fn push_to_inboxes(&self, follower_ids: &[i64], content_id: i64) -> Result<()>;

    /// 读取某用户收件箱的最新内容 id（倒序，最多 limit 条）
    async fn read_inbox(&self, user_id: i64, limit: u64) -> Result<Vec<i64>>;

    /// 从一批粉丝收件箱移除内容 id（内容删除回收，best-effort）
    async fn remove_from_inboxes(&self, follower_ids: &[i64], content_id: i64) -> Result<()>;
}
```

## 实现要点

- `push_to_inboxes`：粉丝分块（500/批），`redis::pipe()` 管道批量执行 ZADD + ZREMRANGEBYRANK，降低 RTT
- `read_inbox`：ZREVRANGE 取最新 N 条（字符串解析为 i64）
- `remove_from_inboxes`：管道批量 ZREM
- 所有错误以 `crate::Result` 返回，由 `UserFeedDataBase` 按 best-effort 处理（不影响发布主流程）

## 架构关系

```
UserFeedDataBase（推拉合并）
    ├─ publish / remove_content → UserFeedCache（写收件箱）
    └─ get_following_feed      → UserFeedCache.read_inbox（读收件箱）
                                   ↓
                              Redis ZSET：feel:feed:inbox:{user_id}
```

## 当前状态

| 方法 | 状态 | 说明 |
|------|------|------|
| `push_to_inboxes` | ✅ | pipeline 扇出 + 500 条裁剪 |
| `read_inbox` | ✅ | ZREVRANGE 倒序读取 |
| `remove_from_inboxes` | ✅ | ZREM 批量回收 |

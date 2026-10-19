# UserFeedDataBase — 关注流数据访问接口（推拉结合）

## 概述

`UserFeedDataBase` 是关注流的核心服务接口（位于 `crates/feel_storage/src/database/feed.rs`），组合 **关注关系 + 内容 + 收件箱缓存**，实现推拉结合：

- **推**（`publish`）：内容发布时扇出到粉丝收件箱（仅粉丝数未超阈值的作者）
- **拉**（`get_following_feed`）：读时合并收件箱与大 V 作者最近内容
- **回收**（`remove_content`）：内容删除后从收件箱移除（best-effort）

默认实现 `CommonUserFeedDataBase { feed_cache, follow_repo, content_repo }`。

## Trait 定义

```rust
#[async_trait]
pub trait UserFeedDataBase: 'static + Send + Sync {
    async fn publish(&self, author_id: i64, content_id: i64) -> Result<()>;
    async fn remove_content(&self, author_id: i64, content_id: i64) -> Result<()>;
    async fn get_following_feed(&self, user_id: i64, page: Option<u64>, page_size: Option<u64>)
        -> Result<FollowingFeedPage>;
}
```

## 推拉结合逻辑

| 方法 | 逻辑 |
|------|------|
| `publish` | 查作者粉丝数：`0` 或无粉丝 → 跳过；`≤ PUSH_FANOUT_LIMIT(5000)` → `find_follower_ids` 后 `push_to_inboxes` 扇出；`> 5000` → 跳过（读时拉取） |
| `remove_content` | 查作者粉丝 → `remove_from_inboxes` 回收（best-effort） |
| `get_following_feed` | 1) 读收件箱（`INBOX_READ_LIMIT=300`）<br>2) 找出我关注的大 V（`find_following_ids` + `count_followers_bulk` 过滤 > 5000）<br>3) 拉取大 V 最近内容（`find_recent_by_users`，窗口 200，作者上限 50）<br>4) 合并去重、按 id 倒序、分页切片（`page/page_size`，默认 20/上限 50）<br>5) 批量加载 Content，过滤已删除/禁用，返回 `FollowingFeedPage { items, page, page_size, has_more }` |

## 参数常量

| 常量 | 默认值 | 说明 |
|------|--------|------|
| `PUSH_FANOUT_LIMIT` | 5000 | 推/拉分界（粉丝数） |
| `INBOX_READ_LIMIT` | 300 | 推模式读窗口 |
| `PULL_AUTHOR_LIMIT` | 50 | 拉模式大 V 作者上限 |
| `PULL_CONTENT_LIMIT` | 200 | 拉模式内容窗口 |

## 一致性说明

- 扇出/回收为 best-effort：失败只影响该内容的后续可见性，主流程不受影响
- 读取侧兜底：内容加载时过滤 `is_delete` / `!enabled`
- 取关后旧内容可能残留在收件箱（流语义保留），后续不再推送
- 冷启动：关注后历史内容不在收件箱（Phase 2 可「新关注补拉」）

## 架构关系

```
Handler（feel_api::feed::following_feed）
    ↓
UserFeedDataBase（推拉合并）
    ├─ ContentRepo（内容读取/拉取）
    ├─ UserFollowRepo（粉丝/关注/粉丝数）
    └─ UserFeedCache（Redis 收件箱）
```

## 当前状态

| 项 | 状态 | 说明 |
|----|------|------|
| `publish` 扇出 | ✅ | 阈值判定 + 批量写收件箱 |
| `remove_content` 回收 | ✅ | best-effort |
| `get_following_feed` | ✅ | 推拉合并 + 分页 + has_more |

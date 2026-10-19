# Feed — 关注流（推拉结合）

## 概述

关注流（Following Feed）是"我关注的人发布的内容"时间线，是关注系统与内容系统的组合应用。采用**推拉结合（hybrid fan-out）**：

- **推（fan-out on write）**：普通作者发布内容时，把内容 id 扇出写入所有粉丝的 Redis 收件箱（写扩散）
- **拉（fan-out on read）**：粉丝数超阈值（默认 5000）的「大 V」作者**不推送**，读取时按需拉取其最近内容并合并

阈值 5000 来自经验权衡：粉丝数可控时写扩散保证读性能；大 V 粉丝过多时写扩散成本过高（一次发布扇出百万级写入），改为读时合并以摊薄成本。

## 设计目标

1. **读性能优先**：普通作者的内容对粉丝是 O(1) 读取（直接读收件箱）
2. **写成本可控**：大 V 发布不产生扇出风暴
3. **窗口语义**：关注流保证最近窗口完整性（推窗口 300 + 拉窗口 200 条），窗口外不保证（`has_more=false` 时客户端停止翻页）
4. **最佳努力一致性**：扇出/回收失败不阻塞主流程；读取时过滤已删除/禁用内容
5. **零新表**：全部基于 Redis ZSET 收件箱 + 现有 `content` / `user_follow` 数据

## 数据流

```
发布内容（POST /content/create）
  → ContentDataBase.create（落库）
  → UserFeedDataBase.publish(author_id, content_id)      [best-effort]
      ├─ 粉丝数 ≤ PUSH_FANOUT_LIMIT → ZADD 到每个粉丝收件箱（pipeline 批量，裁剪 500 条）
      └─ 粉丝数 >  PUSH_FANOUT_LIMIT → 跳过（读时拉取）

读取关注流（GET /feed/following）
  → 推：ZREVRANGE 自己的收件箱（最新 300 条）
  → 拉：找出我关注的大 V（follower_count > 阈值）→ 一次查询拉取最近内容（200 条）
  → 合并去重 → 按内容 id 倒序 → 分页切片 → 批量加载 Content → 过滤已删除/禁用
```

## 关键参数

| 常量 | 默认值 | 位置 | 说明 |
|------|--------|------|------|
| `PUSH_FANOUT_LIMIT` | 5000 | `database/feed.rs` | 粉丝数 ≤ 该值 → 推模式；更大 → 拉模式 |
| `INBOX_READ_LIMIT` | 300 | `database/feed.rs` | 读推窗口（收件箱最新条数） |
| `PULL_AUTHOR_LIMIT` | 50 | `database/feed.rs` | 拉模式合并的大 V 作者上限 |
| `PULL_CONTENT_LIMIT` | 200 | `database/feed.rs` | 拉模式内容窗口 |
| `FEED_INBOX_MAX` | 500 | `cache/feed.rs` | 每个收件箱保留的最大条目数 |

## 领域模型（feel_entity::feed）

```rust
pub struct FollowingFeedPage {
    pub items: Vec<Content>,
    pub page: u64,
    pub page_size: u64,
    pub has_more: bool,   // 合并窗口内是否还有更多
}
```

## API 设计

| 方法 | 路径 | 鉴权 | 说明 |
|------|------|------|------|
| GET | `/api/v1/feed/following` | ✅ Bearer | 我关注的人的流（推拉结合） |

```json
// 请求
{ "page": 1, "page_size": 20 }

// 响应
{
    "code": 0,
    "message": "ok",
    "data": {
        "items": [ { "id": 1, "content_type": "article", "view_count": 12,
                     "author": { "user_id": 42, "name": "张三", "avatar": "…" },
                     "payload": { "body": "…" }, "…": "…" } ],
        "page": 1,
        "page_size": 20,
        "has_more": true
    }
}
```

> `has_more=false` 表示合并窗口已耗尽（推+拉窗口），客户端可停止翻页。

## 业务规则

1. 仅显示 `enabled` 且未删除、且作者的公开内容
2. 读取时过滤已被删除/禁用的收件箱条目（扇出后删除的脏数据）
3. 取关后旧内容可能残留收件箱（流语义保留已读内容），后续不再推送
4. 冷启动说明：关注某人后，其**历史内容**不在收件箱，需等待下一次发布或从作者主页查看（推拉结合的正常取舍，Phase 2 可加"新关注补拉"）

## 分层落点

| 层 | 落点 | 说明 |
|----|------|------|
| `migration` | — | **无需新表**（纯 Redis） |
| `feel_entity::feed` | `crates/feel_entity/src/feed/` | `FollowingFeedPage` |
| `feel_storage::cache::feed` | `crates/feel_storage/src/cache/feed.rs` | `UserFeedCache` / `CommonUserFeedCache`（ZSET 收件箱） |
| `feel_storage::repo` | `crates/feel_storage/src/repo/{follow,content}.rs` | 新增 `find_follower_ids` / `find_following_ids` / `count_followers_bulk` / `find_recent_by_users` |
| `feel_storage::database::feed` | `crates/feel_storage/src/database/feed.rs` | `UserFeedDataBase` / `CommonUserFeedDataBase`（推拉合并） |
| `feel_api::model::feed` | `cmd/feel_api/src/model/feed.rs` | DTO |
| `feel_api::feed` | `cmd/feel_api/src/feed/mod.rs` | handler + 路由 |
| 发布/删除挂钩 | `cmd/feel_api/src/content/mod.rs` | create → publish；remove → 回收收件箱 |

## 实施计划

**Phase 1 — MVP** ✅ 已实现

- 推：发布扇出 + 收件箱裁剪
- 拉：大 V 作者识别 + 内容合并
- `GET /feed/following`（推拉合并 + 分页 + has_more）
- 内容删除回收收件箱（best-effort）

**Phase 2 — 增强**

- 新关注者补拉（关注时把作者最近内容写入收件箱，解决冷启动）
- 关注流排序加权（时间 + 互动热度）
- 收件箱分片/批量扇出并发优化（大粉丝量场景）
- 话题/推荐混合流
- 互动（点赞/评论）后内容回推

## 与现有实现的关系

| 模块 | 关系 |
|------|------|
| `user_follow`（关注系统） | 提供粉丝/关注关系与粉丝数（推拉判定） |
| `content`（内容系统） | 提供内容发布/删除事件与内容读取 |
| `ContentCache`（全站 feed 缓存） | 并行存在：全站 feed 用 Cache-Aside；关注流用收件箱 + 拉合并 |

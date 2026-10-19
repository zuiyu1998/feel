# Feed API — 关注流接口

## 概述

关注流（推拉结合）的 HTTP 接口，设计见 [features/feed.md](../features/feed.md)。实现位于 `cmd/feel_api/src/feed/mod.rs`（handler 与路由）与 `cmd/feel_api/src/model/feed.rs`（DTO）。

## 接口一览

| 方法 | 路径 | 鉴权 | Handler | 说明 |
|------|------|------|---------|------|
| GET | `/api/v1/feed/following` | ✅ Bearer | `following_feed` | 我关注的人的流（推拉结合） |

## GET `/api/v1/feed/following` — 关注流

**请求：**

```json
{ "page": 1, "page_size": 20 }
```

**响应 `ApiResponse<FollowingFeedResponse>`：**

```json
{
    "code": 0,
    "message": "ok",
    "data": {
        "items": [ { "id": 1, "content_type": "article", "title": "…", "view_count": 12,
                     "author": { "user_id": 42, "name": "张三", "avatar": "…" },
                     "payload": { "body": "…" }, "…": "…" } ],
        "page": 1,
        "page_size": 20,
        "has_more": true
    }
}
```

- `items`：合并后的内容卡片（基表字段 + `payload` + `author`），按内容 id 倒序
- `has_more`：**合并窗口**内是否还有更多；`false` 时客户端应停止翻页（窗口外内容不保证返回）
- 分页：`page` 从 1 开始，`page_size` 默认 20、上限 50

## 数据来源（推拉结合）

```
GET /feed/following
  → 推：ZREVRANGE feel:feed:inbox:{user_id}（最新 300 条内容 id）
  → 拉：我关注的大 V（粉丝数 > 5000）→ 最近 200 条内容
  → 合并去重 → 分页切片 → 批量加载 → 过滤已删除/禁用
```

- **推**：普通作者（粉丝 ≤ 5000）发布内容时扇出写入粉丝收件箱（内容创建接口自动触发）
- **拉**：大 V 作者不推送，读取时按需拉取
- 内容删除后收件箱自动回收（best-effort）；读取时同样会过滤已删除内容

## 错误映射

复用 `from_storage_error`（见 [model/response.md](model/response.md)）：认证失败 → `10005`，数据库异常 → `10003`。

## 路由挂载与 AppState

```rust
// cmd/feel_api/src/lib.rs
Route::new().nest("/feed", feed::router())
```

`AppState` 提供 `feed_database: Arc<dyn UserFeedDataBase>`（`CommonUserFeedDataBase`，内含 `CommonUserFeedCache` + `SeaOrmUserFollowRepo` + `SeaOrmContentRepo`）。

## 涉及的 crate 依赖

| crate / 模块 | 作用 |
|--------------|------|
| `feel_api::model::feed` | `FollowingFeedRequest` / `FollowingFeedResponse` |
| `feel_entity::feed` | `FollowingFeedPage` |
| `feel_storage` | `UserFeedDataBase`（推拉合并）+ `UserFeedCache`（收件箱） |
| `feel_sea_orm` | 复用 `content` / `user_follow` 实体 |
| `migration` | 无新增迁移（纯 Redis） |

# Follow DTO — 关注 API 请求/响应模型

## 概述

关注系统的 API 请求/响应 DTO 定义在 `cmd/feel_api/src/model/follow.rs` 中，通过 `serde` 实现 JSON 序列化与反序列化。各接口的完整说明见 [feel_api/follow.md](../follow.md)，领域设计见 [features/follow.md](../../features/follow.md)。

## 请求 DTO

### AddFollowRequest

关注请求体（`POST /follow/add`）：`{ "target_uid": "uid_abc123" }`

### RemoveFollowRequest

取关请求体（`DELETE /follow/remove`）：`{ "target_uid": "uid_abc123" }`

### FollowStatsRequest

关注统计请求体（`GET /follow/stats`）：`{ "user_uid": "uid_abc123" }`

### FollowListRequest

关注 / 粉丝列表请求体（`GET /follow/followers`、`GET /follow/followings`）：

```rust
#[derive(Debug, Deserialize)]
pub struct FollowListRequest {
    pub user_uid: String,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}
```

## 响应 DTO

### FollowResponse

关注关系响应（`From<UserFollow>` 转换）：

```rust
#[derive(Debug, Serialize)]
pub struct FollowResponse {
    pub id: i64,
    pub user_id: i64,
    pub follow_user_id: i64,
    pub enabled: bool,
    pub created_at: String,   // RFC3339
    pub updated_at: String,   // RFC3339
}
```

### FollowStatsResponse

关注统计响应（`From<FollowStats>`）：

```rust
pub struct FollowStatsResponse {
    pub user_id: i64,
    pub follower_count: u64,
    pub following_count: u64,
    pub is_following: Option<bool>,   // 匿名/本人时为 null
}
```

### FollowUserBrief / FollowPageResponse

关注 / 粉丝列表项（内嵌对端用户摘要）与分页响应：

```rust
pub struct FollowUserBrief {
    pub user_id: i64,
    pub uid: String,
    pub name: String,
    pub avatar: String,
}

pub struct FollowPageResponse {
    pub items: Vec<FollowUserBrief>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

impl FollowPageResponse {
    pub fn with_items(items: Vec<FollowUserBrief>, page: &FollowPage) -> Self
}
```

> 对端用户摘要由 handler 层按关联行补齐（同一用户只查一次，缺失用户跳过）。

## 与领域模型的对应关系

| DTO | 领域模型 | 转换方式 |
|-----|----------|----------|
| `FollowResponse` | `feel_entity::follow::UserFollow` | `From<UserFollow>` |
| `FollowStatsResponse` | `feel_entity::follow::FollowStats` | `From<FollowStats>` |
| `FollowPageResponse` | `feel_entity::follow::FollowPage` | `with_items(items, &page)` |
| `AddFollowRequest` / `RemoveFollowRequest` | — | handler 内按 `target_uid` 解析用户后组装 |

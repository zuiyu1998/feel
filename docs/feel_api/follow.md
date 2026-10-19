# Follow API — 关注接口

## 概述

用户关注系统的 HTTP 接口，设计见 [features/follow.md](../features/follow.md)。实现位于 `cmd/feel_api/src/follow/mod.rs`（handler 与路由）与 `cmd/feel_api/src/model/follow.rs`（DTO）。

参数统一走 `Json` 请求体（不使用 Path/Query），响应统一为 `ApiResponse<T>`。

## 接口一览

| 方法 | 路径 | 鉴权 | Handler | 说明 |
|------|------|------|---------|------|
| POST | `/api/v1/follow/add` | ✅ Bearer | `add_follow` | 关注某人 |
| DELETE | `/api/v1/follow/remove` | ✅ Bearer | `remove_follow` | 取关某人 |
| GET | `/api/v1/follow/stats` | ❌（可选 Bearer） | `follow_stats` | 关注统计 + 可选 `is_following` |
| GET | `/api/v1/follow/followers` | ❌ | `list_followers` | 粉丝列表（分页） |
| GET | `/api/v1/follow/followings` | ❌ | `list_followings` | 关注列表（分页） |

## POST `/api/v1/follow/add` — 关注

**请求：**

```json
{ "target_uid": "uid_abc123" }
```

**响应 `ApiResponse<FollowResponse>`：**

```json
{
    "code": 0,
    "message": "ok",
    "data": {
        "id": 10,
        "user_id": 1,
        "follow_user_id": 2,
        "enabled": true,
        "created_at": "2026-02-11T00:00:00Z",
        "updated_at": "2026-02-11T00:00:00Z"
    }
}
```

> 幂等：重复关注返回既有关系；不能关注自己（`10006`）；目标不存在（`10004`）。

## DELETE `/api/v1/follow/remove` — 取关

**请求：**

```json
{ "target_uid": "uid_abc123" }
```

**响应：** `ApiResponse<()>`（`data: null`）。幂等：未关注时静默成功。

## GET `/api/v1/follow/stats` — 关注统计

**请求：**

```json
{ "user_uid": "uid_abc123" }
```

**响应 `ApiResponse<FollowStatsResponse>`：**

```json
{
    "code": 0,
    "message": "ok",
    "data": {
        "user_id": 2,
        "follower_count": 3,
        "following_count": 5,
        "is_following": true
    }
}
```

> 请求头带合法 `Authorization: Bearer <token>` 时 `is_following` 表示当前登录用户是否关注该用户；匿名访问或查看本人时为 `null`。

## GET `/api/v1/follow/followers | /followings` — 粉丝 / 关注列表

**请求：**

```json
{ "user_uid": "uid_abc123", "page": 1, "page_size": 20 }
```

**响应 `ApiResponse<FollowPageResponse>`：**

```json
{
    "code": 0,
    "message": "ok",
    "data": {
        "items": [ { "user_id": 2, "uid": "uid_abc123", "name": "张三", "avatar": "https://example.com/avatar.png" } ],
        "total": 3,
        "page": 1,
        "page_size": 20
    }
}
```

- `followers`：关注了该用户的人；`followings`：该用户关注的人
- 按创建时间倒序；`page_size` 默认 20、上限 50

## 业务校验

| 校验项 | 规则 | 错误 |
|--------|------|------|
| 自关注 | `follower == followee` | `10006` |
| 目标存在 | 目标 uid 必须存在 | `10004` |
| 分页参数 | `page ≥ 1`，`page_size` 1~50 | 自动归一化 |

## 错误映射

复用 `from_storage_error`（见 [model/response.md](model/response.md)）：业务校验失败 → `10006`，用户不存在 → `10004`，数据库异常 → `10003`，认证失败 → `10005`。

## 路由挂载与 AppState

```rust
// cmd/feel_api/src/lib.rs
Route::new()
    .nest("/users", user::router())
    .nest("/labels", label::router())
    .nest("/content", content::router())
    .nest("/follow", follow::router())
```

`AppState` 提供 `user_follow_database: Arc<dyn UserFollowDataBase>`（`CommonUserFollowDataBase`，内含 `SeaOrmUserFollowRepo`）。

## 涉及的 crate 依赖

| crate / 模块 | 作用 |
|--------------|------|
| `feel_api::model::follow` | 请求/响应 DTO（`AddFollowRequest`、`FollowStatsResponse` 等） |
| `feel_entity::follow` | 领域模型（`UserFollow`、`FollowStats`、`FollowPage`） |
| `feel_storage` | `UserFollowDataBase` / `UserFollowRepo` 及其实现 |
| `feel_sea_orm::follow` | `user_follow` ORM 实体 |
| `migration` | `m20260212_000000_add_user_follows` 建表迁移 |

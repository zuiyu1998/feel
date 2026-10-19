# UserFollowDataBase — 关注关系数据访问接口

## 概述

`UserFollowDataBase` 是面向业务层的**关注关系数据访问接口**（位于 `crates/feel_storage/src/database/follow.rs`），封装 `UserFollowRepo`，负责幂等关注/取关、防自关注、分页归一化与统计口径；默认实现 `CommonUserFollowDataBase`。

## 模块位置

```
crates/feel_storage/src/database/
├── mod.rs        # 模块声明（re-export）
├── content.rs    # ContentDataBase
├── follow.rs     # UserFollowDataBase trait + CommonUserFollowDataBase ←
├── label.rs      # LabelDataBase
└── user.rs       # UserDataBase
```

## Trait 定义

```rust
#[async_trait]
pub trait UserFollowDataBase: 'static + Send + Sync {
    async fn follow(&self, follower_id: i64, followee_id: i64) -> Result<UserFollow>;
    async fn unfollow(&self, follower_id: i64, followee_id: i64) -> Result<()>;
    async fn get_stats(&self, user_id: i64, viewer_id: Option<i64>) -> Result<FollowStats>;
    async fn list_followings(&self, user_id: i64, page: Option<u64>, page_size: Option<u64>) -> Result<FollowPage>;
    async fn list_followers(&self, user_id: i64, page: Option<u64>, page_size: Option<u64>) -> Result<FollowPage>;
}
```

| 方法 | 说明 |
|------|------|
| `follow` | 幂等关注：已存在返回既有关系；自关注 → `10006` |
| `unfollow` | 幂等取关：未关注时静默成功 |
| `get_stats` | 粉丝数/关注数；`viewer_id` 为 Some 且非本人时附带 `is_following` |
| `list_followings` / `list_followers` | 分页归一化（默认 20、上限 50）后委托 Repo |

## 业务规则

| 规则 | 实现 |
|------|------|
| 防自关注 | `follower_id == followee_id` → `Error::Business` |
| 幂等关注 | 先 `find_user_follow`，命中即返回 |
| 幂等取关 | `delete_user_follow` 静默执行 |
| 统计口径 | 粉丝 = count(follow_user_id)；关注 = count(user_id) |

> 分页常量复用 `DEFAULT_PAGE_SIZE = 20`、`MAX_PAGE_SIZE = 50`（定义于 `database/content.rs`）。

## 默认实现：CommonUserFollowDataBase

```rust
pub struct CommonUserFollowDataBase {
    follow_repo: Box<dyn UserFollowRepo>,
}

impl CommonUserFollowDataBase {
    pub fn new<T: UserFollowRepo>(follow_repo: T) -> Self { /* ... */ }
}
```

## 架构关系

```
Handler（feel_api::follow）
    ↓
UserFollowDataBase（幂等/防自关注/分页）
    ↓ 委托
CommonUserFollowDataBase ──► UserFollowRepo（SeaOrmUserFollowRepo）
    ↓
user_follow
```

## 当前状态

| 项 | 状态 | 说明 |
|----|------|------|
| 幂等关注 / 取关 | ✅ | `follow` / `unfollow` |
| 防自关注 | ✅ | `10006` |
| 统计 | ✅ | 粉丝数 / 关注数 / `is_following` |
| 分页归一化 | ✅ | 默认 20 / 上限 50 |

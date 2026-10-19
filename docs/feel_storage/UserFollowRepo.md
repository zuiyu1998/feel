# UserFollowRepo — 关注关系持久化层接口

## 概述

`UserFollowRepo` 是 `feel_storage` 中定义的**关注关系持久化接口**（trait 位于 `crates/feel_storage/src/repo/mod.rs`），由 `crates/feel_storage/src/repo/follow.rs` 的 `SeaOrmUserFollowRepo` 实现。

负责 `user_follow` 表的读写：关注/取关、关系查询、粉丝/关注列表与计数。

## 模块位置

```
crates/feel_storage/src/repo/
├── mod.rs        # UserRepo / LabelRepo / ContentRepo / UserFollowRepo trait
├── user.rs
├── label.rs
├── content.rs
└── follow.rs     # SeaOrmUserFollowRepo ←
```

## Trait 定义

```rust
#[async_trait]
pub trait UserFollowRepo: 'static + Send + Sync {
    async fn create_user_follow(&self, create: &UserFollowCreate) -> Result<UserFollow>;
    async fn find_user_follow(&self, user_id: i64, follow_user_id: i64) -> Result<Option<UserFollow>>;
    async fn delete_user_follow(&self, user_id: i64, follow_user_id: i64) -> Result<()>;
    async fn find_followings(&self, user_id: i64, offset: u64, limit: u64) -> Result<Vec<UserFollow>>;
    async fn find_followers(&self, user_id: i64, offset: u64, limit: u64) -> Result<Vec<UserFollow>>;
    async fn count_followings(&self, user_id: i64) -> Result<u64>;
    async fn count_followers(&self, user_id: i64) -> Result<u64>;
}
```

| 方法 | 说明 |
|------|------|
| `create_user_follow` | 插入关注关系（幂等去重由唯一索引 + DataBase 层保证） |
| `find_user_follow` | 查询 follower 是否关注 followee |
| `delete_user_follow` | 删除关注关系（不存在时静默成功） |
| `find_followings` | 某用户的关注列表（`user_id` 过滤，创建时间倒序，offset/limit） |
| `find_followers` | 某用户的粉丝列表（`follow_user_id` 过滤，创建时间倒序） |
| `count_followings` / `count_followers` | 关注数 / 粉丝数统计 |

## 默认实现：SeaOrmUserFollowRepo

- `new(conn)` 注入共享 `DatabaseConnection`
- 写入：`UserFollowActiveModel` 插入（`id: NotSet` 交由数据库序列生成）
- 查询：按 `user_id` / `follow_user_id` 过滤 + `order_by_desc(created_at)` 分页
- 删除：`delete_many` 双条件过滤（幂等）

## 架构关系

```
UserFollowDataBase（业务层：幂等/防自关注/分页）
    ↓ 委托
UserFollowRepo（持久化接口）
    ↓ 实现
SeaOrmUserFollowRepo ── sea-orm ──► user_follow（唯一索引 idx_user_follow_pair）
```

## 当前状态

| 项 | 状态 | 说明 |
|----|------|------|
| `create_user_follow` | ✅ | 序列自增主键 |
| `find_user_follow` | ✅ | 双条件查询 |
| `delete_user_follow` | ✅ | 幂等删除 |
| `find_followings` / `find_followers` | ✅ | 分页（offset/limit） |
| `count_followings` / `count_followers` | ✅ | 统计 |

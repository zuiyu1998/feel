# UserFollowEntity — user_follow 表 ORM 实体

## 概述

`UserFollowEntity` 是 `feel_sea_orm` crate 中定义的 **Sea-ORM 实体**，对应数据库中的 `user_follow` 表（用户关注关系：follower → followee），位于 `crates/feel_sea_orm/src/follow/entities/user_follow.rs`。提供 `UserFollowModel → UserFollow` 的类型转换，供 `SeaOrmUserFollowRepo` 使用。

## 模块位置

```
crates/feel_sea_orm/src/follow/entities/
├── mod.rs        # 模块声明
├── prelude.rs    # 重新导出（UserFollowActiveModel / Column / Entity / Model）
└── user_follow.rs # user_follow 表实体 ←
```

## 数据模型（Model）

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_follow")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    /// 关注者
    #[sea_orm(index)]
    pub user_id: i64,
    /// 被关注者
    #[sea_orm(index)]
    pub follow_user_id: i64,
    /// 关联是否启用
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

## DDL 对应（PostgreSQL，由 migration 创建）

```sql
CREATE TABLE user_follow (
    id             BIGSERIAL    PRIMARY KEY,
    user_id        BIGINT       NOT NULL,
    follow_user_id BIGINT       NOT NULL,
    enabled        BOOLEAN      NOT NULL DEFAULT true,
    created_at     TIMESTAMPTZ  NOT NULL,
    updated_at     TIMESTAMPTZ  NOT NULL
);

-- 建表后单独创建（m20260212_000000_add_user_follows）
CREATE UNIQUE INDEX idx_user_follow_pair   ON user_follow (user_id, follow_user_id);
CREATE INDEX        idx_user_follow_follow ON user_follow (follow_user_id);
```

## 关系定义（Relation）

```rust
pub enum Relation {}
```

**未定义 ORM 关系 / 外键**：`user_id`、`follow_user_id` 均为逻辑关联 `users.id`，由业务层保证一致性。

## 类型转换：UserFollowModel → UserFollow

```rust
impl From<Model> for UserFollow {
    // 字段一一映射
}
```

## 当前状态

| 项 | 状态 | 说明 |
|----|------|------|
| 字段定义 | ✅ | 6 个字段 |
| 唯一索引 | ✅ | `(user_id, follow_user_id)` 联合唯一（幂等去重） |
| 粉丝索引 | ✅ | `follow_user_id` 索引（粉丝列表查询） |
| Relation / 外键 | 🚧 逻辑关联 | 无 ORM 关系、无 FK 约束 |
| From 转换 | ✅ | UserFollowModel → UserFollow |
| 迁移建表 | ✅ | `m20260212_000000_add_user_follows` |

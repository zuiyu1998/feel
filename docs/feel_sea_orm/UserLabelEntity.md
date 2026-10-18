# UserLabelEntity — user_label 表 ORM 实体

## 概述

`UserLabelEntity` 是 `feel_sea_orm` crate 中定义的 **Sea-ORM 实体**，对应数据库中的 `user_label` 表（用户×标签多对多关联），位于 `crates/feel_sea_orm/src/label/entities/user_label.rs`。

该实体提供 `UserLabelModel → UserLabel` 的类型转换，供 `feel_storage` 的 `SeaOrmLabelRepo` 使用；"用户标签列表"与"标签下用户列表"的关联查询均基于此表。

## 模块位置

```
crates/feel_sea_orm/src/label/entities/
├── mod.rs        # 模块声明
├── prelude.rs    # 重新导出（UserLabelActiveModel / UserLabelColumn / UserLabelEntity / UserLabelModel）
├── label.rs      # label 表实体
└── user_label.rs # user_label 表实体 ←
```

### 重新导出路径

```rust
use feel_sea_orm::label::entities::prelude::*;  // 或 feel_sea_orm::prelude::*

// 可用的别名：
//   UserLabelActiveModel  — ActiveModel
//   UserLabelColumn       — Column
//   UserLabelEntity       — Entity
//   UserLabelModel        — Model
```

## 数据模型（Model）

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_label")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    #[sea_orm(index)]
    pub user_id: i64,
    #[sea_orm(index)]
    pub label_id: i64,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 字段说明

| 字段 | 类型 | Sea-ORM 属性 | 说明 |
|------|------|--------------|------|
| `id` | `i64` | `primary_key, auto_increment = false` | 主键，写入时 `NotSet` 交由数据库序列生成 |
| `user_id` | `i64` | `index` | 归属用户（逻辑关联 `users.id`） |
| `label_id` | `i64` | `index` | 关联标签（逻辑关联 `label.id`） |
| `enabled` | `bool` | — | 关联是否启用（默认 true） |
| `created_at` | `DateTime<Utc>` | — | 创建时间（`TIMESTAMPTZ`） |
| `updated_at` | `DateTime<Utc>` | — | 最后更新时间（`TIMESTAMPTZ`） |

### DDL 对应（PostgreSQL，由 migration 创建）

```sql
CREATE TABLE user_label (
    id         BIGSERIAL    PRIMARY KEY,
    user_id    BIGINT       NOT NULL,
    label_id   BIGINT       NOT NULL,
    enabled    BOOLEAN      NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ  NOT NULL,
    updated_at TIMESTAMPTZ  NOT NULL
);

-- 一对用户与标签只能关联一次
CREATE UNIQUE INDEX idx_user_label_user_id_label_id
    ON user_label (user_id, label_id);
-- 反向查询（标签下用户）辅助索引
CREATE INDEX idx_user_label_label_id ON user_label (label_id);
```

## 关系定义（Relation）

```rust
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
```

**未定义 ORM 关系**，迁移中也未建 `FOREIGN KEY` 约束：

- `user_id` / `label_id` 仅为逻辑外键，由业务层（`feel_storage`）保证一致性
- "查看用户标签"通过 `find_user_labels(user_id)` 二次查询 `find_label_by_id` 组装标签信息
- "查看标签下的用户"通过 `find_users_by_label(label_id)` 再查用户主体

## 类型转换：UserLabelModel → UserLabel

```rust
impl From<Model> for UserLabel {
    fn from(value: Model) -> Self {
        UserLabel {
            id: value.id,
            user_id: value.user_id,
            label_id: value.label_id,
            enabled: value.enabled,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
```

## 使用示例

```rust
use feel_sea_orm::prelude::*;
use sea_orm::*;

// 建立关联（id 由数据库序列生成；user_id 为 UserBase.id）
let link = UserLabelActiveModel {
    id: NotSet,
    user_id: Set(1),
    label_id: Set(1),
    enabled: Set(true),
    created_at: Set(Utc::now()),
    updated_at: Set(Utc::now()),
}.insert(&db).await?;

// 用户的标签关联（按创建时间排序）
let links = UserLabelEntity::find()
    .filter(UserLabelColumn::UserId.eq(1))
    .order_by_asc(UserLabelColumn::CreatedAt)
    .all(&db)
    .await?;

// 标签下的用户关联
let links = UserLabelEntity::find()
    .filter(UserLabelColumn::LabelId.eq(1))
    .all(&db)
    .await?;

let user_label: UserLabel = links[0].clone().into();
```

## 依赖关系

| 依赖 | 用途 |
|------|------|
| `sea-orm` | ORM 框架（实体派生、查询） |
| `chrono` | 日期时间类型 `DateTime<Utc>` |
| `feel_entity` | 领域模型 `UserLabel` |

## 当前状态

| 项 | 状态 | 说明 |
|----|------|------|
| 字段定义 | ✅ | 6 个字段完整定义 |
| 唯一索引 | ✅ | `(user_id, label_id)` 联合唯一 |
| Relation / 外键 | 🚧 逻辑关联 | 无 ORM 关系、无 FK 约束，由业务层保证 |
| From 转换 | ✅ | UserLabelModel → UserLabel |
| 迁移建表 | ✅ | `m20260209_000001_add_user_labels` |

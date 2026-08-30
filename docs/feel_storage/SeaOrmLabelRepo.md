# SeaOrmLabelRepo — 标签持久化 Sea-ORM 实现

## 概述

`SeaOrmLabelRepo` 是 `LabelRepo` trait 的 **Sea-ORM 数据库实现**，位于 `crates/feel_storage/src/repo/label.rs`。

它通过 Sea-ORM 操作 `label`（标签本体）和 `user_label`（用户标签关联）两张数据库表，完成标签数据的持久化存储，统一承载 `LabelRepo` 中 `LabelBase` 与 `UserLabel` 的全部操作。

> **当前状态：** `SeaOrmLabelRepo` 已实现于 `crates/feel_storage/src/repo/label.rs`，本文档描述当前实现。

## 模块位置

```
crates/feel_storage/src/repo/
├── mod.rs         # UserRepo / LabelRepo trait 定义
├── user.rs        # SeaOrmUserRepo 实现
└── label.rs       # SeaOrmLabelRepo 实现 ←
```

## 结构体定义

```rust
pub struct SeaOrmLabelRepo {
    /// Sea-ORM 数据库连接
    pub conn: DatabaseConnection,
}
```

### 构造方法

```rust
impl SeaOrmLabelRepo {
    pub fn new(conn: DatabaseConnection) -> Self
}
```

接收一个 `sea_orm::DatabaseConnection`，直接持有以备后续数据库操作。

---

## 依赖项

| 依赖 | 用途 |
| --- | --- |
| `sea_orm` | ORM 框架（`DatabaseConnection`、`ActiveModelTrait`、`EntityTrait`、`QueryFilter`、`ColumnTrait`） |
| `chrono::Local` | 生成当前时间戳 |
| `feel_entity::label::{LabelBase, UserLabel}` | 业务模型 |
| `feel_sea_orm::label::entities::prelude::*` | ORM 实体（`LabelModel`、`LabelActiveModel`、`LabelEntity`、`LabelColumn`、`UserLabelModel` 等） |

---

## 操作的表

### label 表（ORM 实体：`LabelModel` / `LabelActiveModel`）

| 字段 | 类型 | 约束/说明 |
| --- | --- | --- |
| `id` | `i64` | 主键，自增 |
| `name` | `String` | 唯一约束，标签名称（全局唯一） |
| `description` | `String` | 公共描述（默认空串） |
| `remark` | `String` | 标签备注，所有用户共享（默认空串） |
| `influence` | `i64` | 标签影响力，创建时确立，之后不变 |
| `enabled` | `bool` | 是否启用（默认 true） |
| `created_at` | `DateTime<Utc>` | 创建时间 |
| `updated_at` | `DateTime<Utc>` | 更新时间 |

### user_label 表（ORM 实体：`UserLabelModel` / `UserLabelActiveModel`）

| 字段 | 类型 | 约束/说明 |
| --- | --- | --- |
| `id` | `i64` | 主键，自增 |
| `user_id` | `i64` | 索引，归属用户（关联 `users.id`） |
| `label_id` | `i64` | 索引，关联标签（关联 `label.id`） |
| `enabled` | `bool` | 是否启用（默认 true） |
| `created_at` | `DateTime<Utc>` | 创建时间 |
| `updated_at` | `DateTime<Utc>` | 更新时间 |

> 复合唯一约束：`UNIQUE(user_id, label_id)`，一个用户对同一标签只能关联一次。

### 模型转换

`LabelModel` 实现了 `From<Model> for LabelBase`，`UserLabelModel` 实现了 `From<Model> for UserLabel`，可直接转换为业务模型：

```rust
impl From<Model> for LabelBase {
    fn from(value: Model) -> Self {
        LabelBase {
            id: value.id,
            name: value.name,
            description: value.description,
            remark: value.remark,
            influence: value.influence,
            enabled: value.enabled,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

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

---

## 标签本体方法（LabelBase）

### find_label_by_id / find_label_by_name

按主键或唯一名称查找标签本体，未找到时返回 `None`：

```rust
async fn find_label_by_id(&self, label_id: i64) -> Result<Option<LabelBase>> {
    let label = LabelEntity::find_by_id(label_id).one(&self.conn).await?;
    Ok(label.map(Into::into))
}

async fn find_label_by_name(&self, name: &str) -> Result<Option<LabelBase>> {
    let label = LabelEntity::find()
        .filter(LabelColumn::Name.eq(name))
        .one(&self.conn)
        .await?;
    Ok(label.map(Into::into))
}
```

### find_all_labels

查询全部标签本体，按创建时间排序：

```rust
async fn find_all_labels(&self) -> Result<Vec<LabelBase>> {
    let list = LabelEntity::find()
        .order_by_asc(LabelColumn::CreatedAt)
        .all(&self.conn)
        .await?;
    Ok(list.into_iter().map(Into::into).collect())
}
```

### create_label

创建标签本体，参数为请求结构体 `LabelCreate`（`id`、`enabled`、时间戳由存储层管理），`id` 交由数据库自增：

```rust
async fn create_label(&self, create: &LabelCreate) -> Result<LabelBase> {
    let now = Local::now();

    let active = LabelActiveModel {
        id: NotSet,                              // 数据库自增
        name: Set(create.name.clone()),
        description: Set(create.description.clone()),
        remark: Set(create.remark.clone()),
        influence: Set(create.influence),
        enabled: Set(true),
        created_at: Set(now.to_utc()),
        updated_at: Set(now.to_utc()),
    };

    let model: LabelModel = active.insert(&self.conn).await?;   // 写入 label 表
    Ok(model.into())
}
```

> 名称重复时依赖 `label.name` 的唯一约束，返回 `sea_orm::DbErr`（唯一约束冲突）。

### update_label

按 ID 查找后更新字段（如修改备注、禁用/启用），参数为请求结构体 `LabelUpdate`（含 `id` 与可写字段），未找到时返回 `RecordNotFound`：

```rust
async fn update_label(&self, update: &LabelUpdate) -> Result<LabelBase> {
    let now = Local::now();

    // 1. 查找标签本体
    let active = LabelEntity::find_by_id(update.id)
        .one(&self.conn)
        .await?
        .ok_or_else(|| {
            sea_orm::DbErr::RecordNotFound(format!("Label {} not found", update.id))
        })?
        .into_active_model();

    // 2. 更新字段
    let mut active = active;
    active.name = Set(update.name.clone());
    active.description = Set(update.description.clone());
    active.remark = Set(update.remark.clone());
    active.influence = Set(update.influence);
    active.enabled = Set(update.enabled);
    active.updated_at = Set(now.to_utc());

    // 3. 执行更新
    let model: LabelModel = active.update(&self.conn).await?;
    Ok(model.into())
}
```

---

## 用户关联方法（UserLabel）

### find_user_labels / find_users_by_label

按 `user_id` 或 `label_id` 过滤关联列表，`find_user_labels` 按创建时间排序：

```rust
async fn find_user_labels(&self, user_id: i64) -> Result<Vec<UserLabel>> {
    let list = UserLabelEntity::find()
        .filter(UserLabelColumn::UserId.eq(user_id))
        .order_by_asc(UserLabelColumn::CreatedAt)
        .all(&self.conn)
        .await?;
    Ok(list.into_iter().map(Into::into).collect())
}

async fn find_users_by_label(&self, label_id: i64) -> Result<Vec<UserLabel>> {
    let list = UserLabelEntity::find()
        .filter(UserLabelColumn::LabelId.eq(label_id))
        .all(&self.conn)
        .await?;
    Ok(list.into_iter().map(Into::into).collect())
}
```

> 返回的是关联记录（`UserLabel`）；如需内嵌标签本体或用户信息，由上层（数据库/API 层）按 `label_id` / `user_id` 二次查询组装。

### count_user_labels

统计某用户的关联数量，用于"单个用户关联上限 20 个"的校验：

```rust
async fn count_user_labels(&self, user_id: i64) -> Result<u64> {
    let count = UserLabelEntity::find()
        .filter(UserLabelColumn::UserId.eq(user_id))
        .count(&self.conn)
        .await?;
    Ok(count)
}
```

### create_user_label

创建用户标签关联，参数为请求结构体 `UserLabelCreate`（`id`、`enabled`、时间戳由存储层管理），`id` 交由数据库自增：

```rust
async fn create_user_label(&self, create: &UserLabelCreate) -> Result<UserLabel> {
    let now = Local::now();

    let active = UserLabelActiveModel {
        id: NotSet,                              // 数据库自增
        user_id: Set(create.user_id),
        label_id: Set(create.label_id),
        enabled: Set(true),
        created_at: Set(now.to_utc()),
        updated_at: Set(now.to_utc()),
    };

    let model: UserLabelModel = active.insert(&self.conn).await?;  // 写入 user_label 表
    Ok(model.into())
}
```

> 同一用户重复关联同一标签时，依赖 `UNIQUE(user_id, label_id)` 复合唯一约束，返回 `sea_orm::DbErr`。数量上限校验（`count_user_labels`）由上层在调用前执行。

### delete_user_label / delete_user_label_by_ids

按关联 ID 或 用户 + 标签 解除关联（仅删关联，不删除标签本体）：

```rust
async fn delete_user_label(&self, user_label_id: i64) -> Result<()> {
    UserLabelEntity::delete_by_id(user_label_id).exec(&self.conn).await?;
    Ok(())
}

async fn delete_user_label_by_ids(&self, user_id: i64, label_id: i64) -> Result<()> {
    UserLabelEntity::delete_many()
        .filter(UserLabelColumn::UserId.eq(user_id))
        .filter(UserLabelColumn::LabelId.eq(label_id))
        .exec(&self.conn)
        .await?;
    Ok(())
}
```

---

## 关键实现要点

- **唯一约束冲突**：`create_label`（`name` 唯一）与 `create_user_label`（`UNIQUE(user_id, label_id)`）的冲突以 `sea_orm::DbErr` 返回，由上层转换为业务错误提示
- **数量上限**：添加关联前调用 `count_user_labels` 校验上限 20 个，超限由上层拒绝
- **本体只增不删**：不提供删除标签本体的方法；禁用通过 `update_label` 将 `enabled` 置为 `false`
- **主键自增**：两张表的 `id` 均使用 `NotSet` 交由数据库自增，无需额外 ID 生成器
- **无数据库外键**：与 `user_credentials` 表一致，`user_id` / `label_id` 为逻辑关联（仅索引），不建数据库级 FK

---

## 架构关系

```
┌──────────────────────────────────────────────┐
│            LabelRepo (trait)                  │
│        feel_storage::repo::label              │
└──────────────────┬───────────────────────────┘
                   │ 实现
                   ▼
┌──────────────────────────────────────────────┐
│        SeaOrmLabelRepo (struct)               │
│       持有 DatabaseConnection                 │
│                                              │
│  ┌──────────────────┐   ┌──────────────────┐ │
│  │   label 表        │   │  user_label 表   │ │
│  │  (LabelModel)     │   │ (UserLabelModel) │ │
│  └────────┬─────────┘   └──────┬───────────┘ │
│           │                    │             │
│           └─────────┬──────────┘             │
│     唯一约束: name / UNIQUE(user_id, label_id)│
└──────────────────────────────────────────────┘
```

---

## 使用示例

```rust
use sea_orm::DatabaseConnection;
use feel_storage::repo::{LabelRepo, SeaOrmLabelRepo};

let conn: DatabaseConnection = /* 获取数据库连接 */;
let repo = SeaOrmLabelRepo::new(conn);

// 按名称查找标签本体,不存在则创建
let label = match repo.find_label_by_name("Rust 开发者").await? {
    Some(l) => l,
    None => {
        repo.create_label(&LabelCreate {
            name: "Rust 开发者".into(),
            description: "使用 Rust 进行开发的人".into(),
            remark: String::new(),
            influence: 50,
        })
        .await?
    }
};

// 校验数量上限后为用户添加关联
if repo.count_user_labels(1).await? < 20 {
    let user_label = repo
        .create_user_label(&UserLabelCreate {
            user_id: 1,
            label_id: label.id,
        })
        .await?;
    println!("已添加关联: user {} -> label {}", user_label.user_id, user_label.label_id);
}
```

---

## 当前状态

| 方法 | 实现状态 | 说明 |
| --- | --- | --- |
| `find_label_by_id` | ✅ 已实现 | `crates/feel_storage/src/repo/label.rs` |
| `find_label_by_name` | ✅ 已实现 | 同上 |
| `find_all_labels` | ✅ 已实现 | 同上 |
| `create_label` | ✅ 已实现 | 同上 |
| `update_label` | ✅ 已实现 | 同上 |
| `find_user_labels` | ✅ 已实现 | 同上 |
| `find_users_by_label` | ✅ 已实现 | 同上 |
| `count_user_labels` | ✅ 已实现 | 同上 |
| `create_user_label` | ✅ 已实现 | 同上 |
| `delete_user_label` | ✅ 已实现 | 同上 |
| `delete_user_label_by_ids` | ✅ 已实现 | 同上 |

> `SeaOrmLabelRepo` 已实现于 `crates/feel_storage/src/repo/label.rs`，`LabelRepo` trait 定义于 `crates/feel_storage/src/repo/mod.rs`。

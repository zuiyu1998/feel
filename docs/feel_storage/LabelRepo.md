# LabelRepo — 标签持久化层接口

## 概述

`LabelRepo` 是 `feel_storage` crate 中规划的**标签数据持久化层 trait**，统一负责 `LabelBase`(标签本体)与 `UserLabel`(用户标签关联)的数据库操作，位于 `crates/feel_storage/src/repo/mod.rs`(待实现)。

它抽象了对标签数据的数据库操作——标签本体的查找/创建/更新、用户标签关联的增删查——使得上层业务无需关心具体存储实现。默认的 Sea-ORM 实现为 `SeaOrmLabelRepo`，位于 `crates/feel_storage/src/repo/label.rs`。

对应关系：

| 层 | 位置 | 说明 |
| --- | --- | --- |
| 特性设计 | `docs/features/label.md` | 标签特性设计文档 |
| 领域模型 | `feel_entity::label::{LabelBase, UserLabel}` | 纯数据结构 |
| ORM 实体 | `feel_sea_orm::label::entities::{label, user_label}` | 表实体，含 `From<Model>` 转换 |
| 持久化接口 | `feel_storage::repo::LabelRepo`(本文档) | 本 trait |

> **当前状态：** `LabelRepo` trait 与 `SeaOrmLabelRepo` 均已实现于 `crates/feel_storage/src/repo/`，本文档描述当前实现。

## 模块位置

```
crates/feel_storage/src/repo/
├── mod.rs         # UserRepo / LabelRepo trait 定义 + 模块重新导出
├── user.rs        # SeaOrmUserRepo 实现
└── label.rs       # SeaOrmLabelRepo 实现
```

## Trait 定义

```rust
use crate::Result;
use async_trait::async_trait;
use feel_entity::label::{
    LabelBase, LabelCreate, LabelUpdate, UserLabel, UserLabelCreate,
};

#[async_trait]
pub trait LabelRepo: 'static + Send + Sync {
    /// 按 ID 查找标签本体
    async fn find_label_by_id(&self, label_id: i64) -> Result<Option<LabelBase>>;

    /// 按名称查找标签本体(名称全局唯一)
    async fn find_label_by_name(&self, name: &str) -> Result<Option<LabelBase>>;

    /// 创建标签本体
    async fn create_label(&self, create: &LabelCreate) -> Result<LabelBase>;

    /// 更新标签本体(如修改备注、禁用/启用)
    async fn update_label(&self, update: &LabelUpdate) -> Result<LabelBase>;

    /// 查询某用户的全部标签关联(按创建时间排序)
    async fn find_user_labels(&self, user_id: i64) -> Result<Vec<UserLabel>>;

    /// 查询某标签下的全部用户关联
    async fn find_users_by_label(&self, label_id: i64) -> Result<Vec<UserLabel>>;

    /// 统计某用户的关联数量(用于数量上限校验)
    async fn count_user_labels(&self, user_id: i64) -> Result<u64>;

    /// 创建用户标签关联(同一用户对同一标签只能关联一次)
    async fn create_user_label(&self, create: &UserLabelCreate) -> Result<UserLabel>;

    /// 按关联 ID 解除用户标签关联
    async fn delete_user_label(&self, user_label_id: i64) -> Result<()>;

    /// 按 用户 + 标签 解除关联
    async fn delete_user_label_by_ids(&self, user_id: i64, label_id: i64) -> Result<()>;
}
```

### Trait 约束

| 约束      | 说明                                   |
| --------- | -------------------------------------- |
| `'static` | 不包含非静态引用，可安全持有           |
| `Send`    | 可跨线程传递所有权                     |
| `Sync`    | 可被多线程共享引用                     |

### 方法说明

| 方法 | 参数 | 返回 | 说明 |
| --- | --- | --- | --- |
| `find_label_by_id` | `label_id: i64` | `Result<Option<LabelBase>>` | 按 ID 查标签本体 |
| `find_label_by_name` | `name: &str` | `Result<Option<LabelBase>>` | 按名称查标签本体（名称全局唯一） |
| `create_label` | `&LabelCreate` | `Result<LabelBase>` | 创建标签本体（名称需唯一） |
| `update_label` | `&LabelUpdate` | `Result<LabelBase>` | 更新标签本体（如修改备注、禁用/启用） |
| `find_user_labels` | `user_id: i64` | `Result<Vec<UserLabel>>` | 查某用户的全部关联（按创建时间排序） |
| `find_users_by_label` | `label_id: i64` | `Result<Vec<UserLabel>>` | 查某标签下的全部用户关联 |
| `count_user_labels` | `user_id: i64` | `Result<u64>` | 统计关联数量（数量上限校验） |
| `create_user_label` | `&UserLabelCreate` | `Result<UserLabel>` | 添加关联（`UNIQUE(user_id, label_id)` 约束） |
| `delete_user_label` | `user_label_id: i64` | `Result<()>` | 按关联 ID 解除关联 |
| `delete_user_label_by_ids` | `user_id: i64, label_id: i64` | `Result<()>` | 按 用户 + 标签 解除关联 |

> **注意：** 创建/更新操作使用专用请求结构体 `LabelCreate`、`LabelUpdate`、`UserLabelCreate`（定义于 `feel_entity::label`），与 `UserRegister` / `UserLogin` 的模式一致：`id`、`enabled`（默认 true）、时间戳由存储层管理，调用方无需提供。

> **注意：** `LabelRepo` 不包含**删除标签本体**的方法——标签本体是公有资源，删除会影响其他关联用户，按设计只支持创建/禁用，不支持物理删除。`UserLabel` 本身也无需"修改"方法，因为关联除 `enabled` 外无可变字段，个性化内容（备注）位于 `LabelBase` 上。

---

## 默认实现：SeaOrmLabelRepo

```rust
pub struct SeaOrmLabelRepo {
    /// Sea-ORM 数据库连接
    pub conn: DatabaseConnection,
}
```

`SeaOrmLabelRepo` 是 `LabelRepo` trait 的基于 [Sea-ORM](https://www.sea-ql.org/SeaORM/) 的实现。详细实现见 [`SeaOrmLabelRepo.md`](SeaOrmLabelRepo.md)。

### 构造方法

```rust
impl SeaOrmLabelRepo {
    pub fn new(conn: DatabaseConnection) -> Self
}
```

接收一个 Sea-ORM 的 `DatabaseConnection` 实例。

### 关键实现要点

- **标签名唯一**：`create_label` 依赖 `label.name` 的唯一约束，名称已存在时返回数据库错误
- **关联唯一**：`create_user_label` 依赖 `user_label` 表的复合唯一约束 `UNIQUE(user_id, label_id)`，重复关联时返回数据库错误
- **数量上限**：添加关联前调用 `count_user_labels` 校验业务规则"单个用户关联上限 20 个"
- **本体只增不删**：不实现删除标签本体的方法；禁用通过 `update_label` 将 `enabled` 置为 `false`
- **按用户过滤**：`find_user_labels` 按 `user_id` 过滤 `user_label` 表（由唯一索引左前缀覆盖）

---

## 架构关系

```
┌──────────────────────────────────────────────┐
│            LabelRepo (trait)                  │
│          feel_storage::repo::label            │
│       持久化层抽象(标签本体 + 用户关联)        │
└──────────────────┬───────────────────────────┘
                   │ 实现
                   ▼
┌──────────────────────────────────────────────┐
│         SeaOrmLabelRepo (struct)              │
│          Sea-ORM 数据库操作实现               │
├──────────────────────────────────────────────┤
│  label 表            user_label 表            │
│  ┌──────────────┐   ┌──────────────────┐     │
│  │ id           │   │ id               │     │
│  │ name         │   │ user_id          │     │
│  │ description  │   │ label_id         │     │
│  │ remark       │   │ enabled          │     │
│  │ influence    │   │ created_at       │     │
│  │ enabled      │   │ updated_at       │     │
│  │ created_at   │   └──────────────────┘     │
│  │ updated_at   │                            │
│  └──────────────┘                            │
└──────────────────────────────────────────────┘
```

### 依赖关系

- **数据库 ORM**：Sea-ORM（`sea_orm` crate）
- **实体模型**：`feel_sea_orm::label::entities`（定义 `label` 和 `user_label` 表的 ORM 实体，含 `From<Model>` 转换）
- **上层数据模型**：`feel_entity::label::models`（`LabelBase`、`UserLabel`，及请求结构体 `LabelCreate` / `LabelUpdate` / `UserLabelCreate`）
- **错误类型**：`crate::Result` / `crate::Error`（定义于 `crates/feel_storage/src/error.rs`）

---

## 错误处理

`LabelRepo` 中的方法均返回 `crate::Result<T>`，其定义为：

```rust
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Db(#[from] sea_orm::DbErr),

    #[error("Authentication error: {0}")]
    Authentication(String),
}
```

标签操作中可能返回的错误：

- `sea_orm::DbErr` — 数据库操作失败（读写 `label` / `user_label` 表）
- 唯一约束冲突（`label.name`、`UNIQUE(user_id, label_id)`）以 `DbErr` 形式返回，由上层转换为业务错误
- 通过 `?` 操作符自动向上传播

---

## 使用示例

```rust
use sea_orm::DatabaseConnection;
use feel_storage::repo::{LabelRepo, SeaOrmLabelRepo};

let conn: DatabaseConnection = /* 获取数据库连接 */;
let repo = SeaOrmLabelRepo::new(conn);

// 按名称查找标签本体,不存在则创建
let label = match repo.find_label_by_name("Rust 开发者").await.unwrap() {
    Some(l) => l,
    None => {
        repo.create_label(&LabelCreate {
            name: "Rust 开发者".into(),
            description: "使用 Rust 进行开发的人".into(),
            remark: String::new(),
            influence: 50,
        })
        .await
        .unwrap()
    }
};

// 为用户添加关联(校验数量上限后)
let user_label = repo
    .create_user_label(&UserLabelCreate {
        user_id: 1,
        label_id: label.id,
    })
    .await
    .unwrap();

// 解除关联
repo.delete_user_label(user_label.id).await.unwrap();
```

---

## 与业务规则的对应

| `label.md` 业务规则 | 对应方法 |
| --- | --- |
| 标签名全局唯一 | `find_label_by_name` / `create_label`（唯一约束） |
| 备注随本体（所有用户共享） | `update_label`（修改备注） |
| 单个用户关联上限 20 个 | `count_user_labels` + `create_user_label` |
| 一个用户对同一标签只能关联一次 | `create_user_label`（`UNIQUE(user_id, label_id)`） |
| 本体只增不删（公有不属于任何人） | 无删除标签本体方法，`update_label` 支持禁用 |

---

## 与 LabelDataBase 的对比

| 维度 | LabelDataBase | LabelRepo |
| --- | --- | --- |
| 所属模块 | `database::label` | `repo::label` |
| 职责 | 数据访问接口（业务层使用） | 持久化实现接口（存储层） |
| 方法 | create_label / update_label / get_label / get_label_by_name / get_user_labels / get_users_by_label / count_user_labels / add_label / remove_label | find_* / create_* / update_* / delete_* |
| 数量上限 | `add_label` 内置 20 个上限校验 | 不校验，由上层负责 |
| 实现方式 | 委托 LabelRepo | 直接操作 Sea-ORM |

设计见 [`LabelDataBase.md`](LabelDataBase.md)。

---

## 当前状态

| 方法 | 实现状态 | 说明 |
| --- | --- | --- |
| `find_label_by_id` | ✅ 已实现 | `crates/feel_storage/src/repo/label.rs` |
| `find_label_by_name` | ✅ 已实现 | 同上 |
| `create_label` | ✅ 已实现 | 同上 |
| `update_label` | ✅ 已实现 | 同上 |
| `find_user_labels` | ✅ 已实现 | 同上 |
| `find_users_by_label` | ✅ 已实现 | 同上 |
| `count_user_labels` | ✅ 已实现 | 同上 |
| `create_user_label` | ✅ 已实现 | 同上 |
| `delete_user_label` | ✅ 已实现 | 同上 |
| `delete_user_label_by_ids` | ✅ 已实现 | 同上 |

> `LabelRepo` trait 定义于 `crates/feel_storage/src/repo/mod.rs`，`SeaOrmLabelRepo` 实现于 `crates/feel_storage/src/repo/label.rs`，两者均已随 `repo` 模块导出。

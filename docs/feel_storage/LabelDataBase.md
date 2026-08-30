# LabelDataBase — 标签数据访问接口

## 概述

`LabelDataBase` 是 `feel_storage` crate 中定义的**标签数据访问层 trait**，位于 `crates/feel_storage/src/database/label.rs`。

它抽象了对标签相关对象（`LabelBase` 标签本体、`UserLabel` 用户标签关联）的数据操作——标签本体的创建/更新/查询、用户标签关联的添加/解除/查询——使得业务层（`feel_api`）无需关心底层存储实现。默认实现为 `CommonLabelDataBase`，内部委托给 `LabelRepo` trait 完成实际持久化。

对应关系：

| 层 | 位置 | 说明 |
| --- | --- | --- |
| 特性设计 | `docs/features/label.md` | 标签特性设计文档 |
| 领域模型 | `feel_entity::label` | `LabelBase` / `UserLabel` 及请求结构体 |
| 持久化接口 | `feel_storage::repo::LabelRepo` | 见 `docs/feel_storage/LabelRepo.md` |
| 持久化实现 | `feel_storage::repo::SeaOrmLabelRepo` | 见 `docs/feel_storage/SeaOrmLabelRepo.md` |
| 数据访问接口 | `feel_storage::database::LabelDataBase`（本文档） | 本 trait |

> **当前状态：** `LabelDataBase` trait 与 `CommonLabelDataBase` 均已实现于 `crates/feel_storage/src/database/label.rs`，本文档描述当前实现。

## 模块位置

```
crates/feel_storage/src/database/
├── mod.rs          # 重新导出 user / label 模块
├── user.rs         # UserDataBase trait + CommonUserDataBase 实现
└── label.rs        # LabelDataBase trait + CommonLabelDataBase 实现 ←
```

## Trait 定义

```rust
use crate::Result;
use async_trait::async_trait;
use feel_entity::label::{LabelBase, LabelCreate, LabelUpdate, UserLabel};

#[async_trait]
pub trait LabelDataBase: 'static + Send + Sync {
    /// 创建标签本体
    async fn create_label(&self, create: &LabelCreate) -> Result<LabelBase>;

    /// 更新标签本体（如修改备注、禁用/启用）
    async fn update_label(&self, update: &LabelUpdate) -> Result<LabelBase>;

    /// 根据 ID 获取标签本体
    async fn get_label(&self, label_id: i64) -> Result<Option<LabelBase>>;

    /// 根据名称获取标签本体（名称全局唯一）
    async fn get_label_by_name(&self, name: &str) -> Result<Option<LabelBase>>;

    /// 获取全部标签本体
    async fn get_all_labels(&self) -> Result<Vec<LabelBase>>;

    /// 获取某用户的全部标签关联（按创建时间排序）
    async fn get_user_labels(&self, user_id: i64) -> Result<Vec<UserLabel>>;

    /// 获取某标签下的全部用户关联
    async fn get_users_by_label(&self, label_id: i64) -> Result<Vec<UserLabel>>;

    /// 统计某用户的关联数量（用于数量上限校验）
    async fn count_user_labels(&self, user_id: i64) -> Result<u64>;

    /// 为用户添加标签关联（同一用户对同一标签只能关联一次）
    async fn add_label(&self, user_id: i64, label_id: i64) -> Result<UserLabel>;

    /// 解除用户标签关联（仅解除关联，不删除标签本体）
    async fn remove_label(&self, user_id: i64, label_id: i64) -> Result<()>;
}
```

### Trait 约束

| 约束 | 说明 |
| --- | --- |
| `'static` | 不包含非静态引用，可安全持有 |
| `Send` | 可跨线程传递所有权 |
| `Sync` | 可被多线程共享引用 |
| `#[async_trait]` | 支持 async fn 方法 |

### 方法说明

| 方法 | 参数 | 返回 | 说明 |
| --- | --- | --- | --- |
| `create_label` | `&LabelCreate` | `Result<LabelBase>` | 创建标签本体（名称需唯一） |
| `update_label` | `&LabelUpdate` | `Result<LabelBase>` | 更新标签本体（如修改备注、禁用/启用） |
| `get_label` | `label_id: i64` | `Result<Option<LabelBase>>` | 按 ID 获取标签本体 |
| `get_label_by_name` | `name: &str` | `Result<Option<LabelBase>>` | 按名称获取标签本体 |
| `get_all_labels` | — | `Result<Vec<LabelBase>>` | 获取全部标签本体（按创建时间排序） |
| `get_user_labels` | `user_id: i64` | `Result<Vec<UserLabel>>` | 查某用户的全部关联（按创建时间排序） |
| `get_users_by_label` | `label_id: i64` | `Result<Vec<UserLabel>>` | 查某标签下的全部用户关联 |
| `count_user_labels` | `user_id: i64` | `Result<u64>` | 统计关联数量（数量上限校验） |
| `add_label` | `user_id: i64, label_id: i64` | `Result<UserLabel>` | 为用户添加标签关联（含上限校验） |
| `remove_label` | `user_id: i64, label_id: i64` | `Result<()>` | 解除用户标签关联 |

> **注意：** 与 `UserDataBase` 不同，`LabelDataBase` **不包含** token/JWT 相关方法——标签数据访问不需要会话令牌逻辑。`add_label` 在委托持久化前会校验"单个用户关联上限 20 个"的业务规则（见 `docs/features/label.md`）。

---

## 默认实现：CommonLabelDataBase

```rust
pub struct CommonLabelDataBase {
    label_repo: Box<dyn LabelRepo>,
}
```

`CommonLabelDataBase` 是 `LabelDataBase` trait 的默认实现。它通过组合一个 `Box<dyn LabelRepo>` 将数据操作委托给具体的存储层。

### 构造方法

```rust
impl CommonLabelDataBase {
    pub fn new<T: LabelRepo>(label_repo: T) -> Self
}
```

接受任何实现了 `LabelRepo` trait 的类型，将其装箱后存储在内部。

### 实现委托关系

| LabelDataBase 方法 | 委托至 LabelRepo 方法 | 说明 |
| --- | --- | --- |
| `create_label` | `create_label` | async 传递 |
| `update_label` | `update_label` | async 传递 |
| `get_label` | `find_label_by_id` | async 传递 |
| `get_label_by_name` | `find_label_by_name` | async 传递 |
| `get_all_labels` | `find_all_labels` | async 传递 |
| `get_user_labels` | `find_user_labels` | async 传递 |
| `get_users_by_label` | `find_users_by_label` | async 传递 |
| `count_user_labels` | `count_user_labels` | async 传递 |
| `add_label` | `create_user_label` | 先校验数量上限，再构造 `UserLabelCreate` 委托 |
| `remove_label` | `delete_user_label_by_ids` | async 传递 |

---

## 架构关系

```
┌──────────────────────────────────────────────────────────┐
│                     App Layer                            │
│  (feel_api / 业务代码)                                    │
│      持有 Arc<dyn LabelDataBase>                         │
└─────────────────────┬────────────────────────────────────┘
                      │ 调用 trait 方法
                      ▼
┌──────────────────────────────────────────────────────────┐
│             LabelDataBase (trait)                        │
│           feel_storage::database::label                  │
│          标签数据访问抽象                                  │
└─────────────────────┬────────────────────────────────────┘
                      │ 默认实现委托
                      ▼
┌──────────────────────────────────────────────────────────┐
│          CommonLabelDataBase (struct)                    │
│             持有 Box<dyn LabelRepo>                      │
└─────────────────────┬────────────────────────────────────┘
                      │ 委托调用
                      ▼
┌──────────────────────────────────────────────────────────┐
│             LabelRepo (trait)                            │
│           feel_storage::repo::label                      │
│          持久化层抽象                                    │
└─────────────────────────┬────────────────────────────────┘
                          │ 具体实现 (SeaOrmLabelRepo)
                          ▼
┌──────────────────────────────────────────────────────────┐
│                 数据库 / ORM                              │
│           label 表            user_label 表               │
└──────────────────────────────────────────────────────────┘
```

### 依赖关系

- **crate 依赖**：`feel_storage` → `feel_entity`（数据模型）
- **trait 依赖**：`LabelDataBase` 依赖于 `LabelRepo`
- **类型依赖**：`LabelBase`、`UserLabel` 及请求结构体定义在 `feel_entity::label::models`

---

## 相关数据模型

以下类型定义于 `crates/feel_entity/src/label/models/mod.rs`。

### LabelBase

标签本体，作为创建/更新/查询的返回类型。

```rust
pub struct LabelBase {
    pub id: i64,                 // 数据库主键
    pub name: String,            // 标签名称（全局唯一）
    pub description: String,     // 公共描述
    pub remark: String,          // 标签备注（所有用户共享）
    pub influence: i64,          // 标签影响力（创建时确立）
    pub enabled: bool,           // 是否启用
    pub created_at: DateTime<Utc>,  // 创建时间
    pub updated_at: DateTime<Utc>,  // 更新时间
}
```

### UserLabel

用户标签关联，作为关联查询/添加的返回类型。

```rust
pub struct UserLabel {
    pub id: i64,                 // 数据库主键
    pub user_id: i64,            // 归属用户（关联 UserBase.id）
    pub label_id: i64,           // 关联标签（关联 LabelBase.id）
    pub enabled: bool,           // 是否启用
    pub created_at: DateTime<Utc>,  // 创建时间
    pub updated_at: DateTime<Utc>,  // 更新时间
}
```

### LabelCreate / LabelUpdate / UserLabelCreate

请求结构体，作为创建/更新/添加关联的参数（`id`、`enabled`、时间戳由存储层管理）。

```rust
pub struct LabelCreate {
    pub name: String,            // 标签名称（全局唯一）
    pub description: String,     // 公共描述
    pub remark: String,          // 标签备注
    pub influence: i64,          // 标签影响力
}

pub struct LabelUpdate {
    pub id: i64,                 // 待更新标签本体的主键
    pub name: String,
    pub description: String,
    pub remark: String,
    pub influence: i64,
    pub enabled: bool,           // 是否启用（用于禁用/启用）
}

pub struct UserLabelCreate {
    pub user_id: i64,            // 归属用户
    pub label_id: i64,           // 关联标签
}
```

---

## 使用示例

```rust
use std::sync::Arc;
use feel_storage::database::{LabelDataBase, CommonLabelDataBase};
use feel_storage::repo::SeaOrmLabelRepo;  // 具体实现

let repo = SeaOrmLabelRepo::new(db_connection);
let label_db: Arc<dyn LabelDataBase> = Arc::new(CommonLabelDataBase::new(repo));

// 创建标签本体
let label = label_db
    .create_label(&LabelCreate {
        name: "Rust 开发者".into(),
        description: "使用 Rust 进行开发的人".into(),
        remark: String::new(),
        influence: 50,
    })
    .await?;

// 为用户添加标签关联（内部校验数量上限）
label_db.add_label(1, label.id).await?;

// 查看用户的标签
let user_labels = label_db.get_user_labels(1).await?;
```

---

## 当前状态

| 方法 | 实现状态 | 说明 |
| --- | --- | --- |
| `create_label` | ✅ 已实现 | `crates/feel_storage/src/database/label.rs` |
| `update_label` | ✅ 已实现 | 同上 |
| `get_label` | ✅ 已实现 | 同上 |
| `get_label_by_name` | ✅ 已实现 | 同上 |
| `get_all_labels` | ✅ 已实现 | 同上 |
| `get_user_labels` | ✅ 已实现 | 同上 |
| `get_users_by_label` | ✅ 已实现 | 同上 |
| `count_user_labels` | ✅ 已实现 | 同上 |
| `add_label` | ✅ 已实现 | 同上（含数量上限校验） |
| `remove_label` | ✅ 已实现 | 同上 |

> `LabelDataBase` trait 定义与 `CommonLabelDataBase` 实现均位于 `crates/feel_storage/src/database/label.rs`，已随 `database` 模块导出。默认实现详见 [`CommonLabelDataBase.md`](CommonLabelDataBase.md)。

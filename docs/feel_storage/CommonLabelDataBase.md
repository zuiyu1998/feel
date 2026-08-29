# CommonLabelDataBase — 标签数据访问默认实现

## 概述

`CommonLabelDataBase` 是 `LabelDataBase` trait 的**默认实现**，位于 `crates/feel_storage/src/database/label.rs`。

它采用**纯委托模式**：全部数据操作转发给内部的 `Box<dyn LabelRepo>`，自身不做任何持久化逻辑。与 `CommonUserDataBase`（JWT 自实现 + 缓存）不同，`CommonLabelDataBase` 不需要会话令牌或缓存——唯一的附加逻辑是 `add_label` 中的"数量上限"业务校验。这使得业务层可以通过 `LabelDataBase` trait 统一编程，而底层存储实现可以灵活替换。

## 模块位置

```
crates/feel_storage/src/database/
├── mod.rs          # 重新导出 label / user 模块
├── user.rs         # UserDataBase trait + CommonUserDataBase 实现
└── label.rs        # LabelDataBase trait + CommonLabelDataBase 实现 ←
```

## 结构体定义

```rust
pub struct CommonLabelDataBase {
    label_repo: Box<dyn LabelRepo>,
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `label_repo` | `Box<dyn LabelRepo>` | 内部持有的持久化层委托对象 |

### Trait 约束

| 隐含约束 | 来源 | 说明 |
| --- | --- | --- |
| `Send` | `Box<dyn LabelRepo>` | 可在线程间安全转移 |
| `Sync` | `LabelRepo: Sync` | 可在线程间安全共享引用 |

> `CommonLabelDataBase` 本身未显式标注 trait 约束，但由于其成员 `Box<dyn LabelRepo>` 要求 `LabelRepo: 'static + Send + Sync`，这些约束会由编译器自动推导。

## 构造方法

```rust
impl CommonLabelDataBase {
    pub fn new<T: LabelRepo>(label_repo: T) -> Self;
}
```

- `label_repo` — 任何实现了 `LabelRepo` trait 的类型，装箱后存储

**使用示例：**

```rust
use feel_storage::repo::SeaOrmLabelRepo;
use feel_storage::database::CommonLabelDataBase;

let repo = SeaOrmLabelRepo::new(db_connection);
let label_db = CommonLabelDataBase::new(repo);
```

---

## 实现委托关系

`CommonLabelDataBase` 实现了 `LabelDataBase` trait，所有方法委托给 `self.label_repo`。

### create_label / update_label / get_label / get_label_by_name

```rust
async fn create_label(&self, create: &LabelCreate) -> Result<LabelBase> {
    self.label_repo.create_label(create).await
}

async fn update_label(&self, update: &LabelUpdate) -> Result<LabelBase> {
    self.label_repo.update_label(update).await
}

async fn get_label(&self, label_id: i64) -> Result<Option<LabelBase>> {
    self.label_repo.find_label_by_id(label_id).await
}

async fn get_label_by_name(&self, name: &str) -> Result<Option<LabelBase>> {
    self.label_repo.find_label_by_name(name).await
}
```

直接转发至 `LabelRepo` 对应方法，async 传递。标签本体的唯一约束（`name`）、只增不删等规则由存储层保证。

### get_user_labels / get_users_by_label / count_user_labels

```rust
async fn get_user_labels(&self, user_id: i64) -> Result<Vec<UserLabel>> {
    self.label_repo.find_user_labels(user_id).await
}

async fn get_users_by_label(&self, label_id: i64) -> Result<Vec<UserLabel>> {
    self.label_repo.find_users_by_label(label_id).await
}

async fn count_user_labels(&self, user_id: i64) -> Result<u64> {
    self.label_repo.count_user_labels(user_id).await
}
```

直接转发，async 传递。返回的 `UserLabel` 关联如需内嵌标签本体或用户信息，由 API 层二次查询组装。

### add_label

`add_label` 是唯一包含**业务校验**的方法：先统计用户现有关联数量，超限（`>= 20`）返回 `Error::Business`，否则构造 `UserLabelCreate` 委托存储层：

```rust
async fn add_label(&self, user_id: i64, label_id: i64) -> Result<UserLabel> {
    // 业务规则:单个用户的标签关联数量上限为 MAX_USER_LABELS 个
    let count = self.label_repo.count_user_labels(user_id).await?;
    if count >= MAX_USER_LABELS {
        return Err(crate::Error::Business(format!(
            "User {} already has {} labels, limit is {}",
            user_id, count, MAX_USER_LABELS
        )));
    }

    let create = UserLabelCreate { user_id, label_id };
    self.label_repo.create_user_label(&create).await
}
```

**关键细节：**

- **数量上限**：`MAX_USER_LABELS = 20`（`pub const`，与 `docs/features/label.md` 业务规则一致）
- **错误类型**：超限返回 `Error::Business(String)`，由 `feel_api` 映射为 `CODE_BUSINESS_ERROR` (10006)
- **唯一约束**：同一用户重复关联同一标签时，由存储层 `UNIQUE(user_id, label_id)` 约束兜底

### remove_label

```rust
async fn remove_label(&self, user_id: i64, label_id: i64) -> Result<()> {
    self.label_repo
        .delete_user_label_by_ids(user_id, label_id)
        .await
}
```

直接转发，async 传递。仅解除关联，不删除标签本体。

### 委托总览

| LabelDataBase 方法 | 委托目标 | 实现状态 |
| --- | --- | --- |
| `create_label` | `LabelRepo::create_label` | ✅ 完整 |
| `update_label` | `LabelRepo::update_label` | ✅ 完整 |
| `get_label` | `LabelRepo::find_label_by_id` | ✅ 完整 |
| `get_label_by_name` | `LabelRepo::find_label_by_name` | ✅ 完整 |
| `get_user_labels` | `LabelRepo::find_user_labels` | ✅ 完整 |
| `get_users_by_label` | `LabelRepo::find_users_by_label` | ✅ 完整 |
| `count_user_labels` | `LabelRepo::count_user_labels` | ✅ 完整 |
| `add_label` | `LabelRepo::count_user_labels` + `create_user_label`（含上限校验） | ✅ 完整 |
| `remove_label` | `LabelRepo::delete_user_label_by_ids` | ✅ 完整 |

---

## 架构关系

```
┌──────────────────────────────────────────────┐
│                  App Layer                    │
│ 持有 Arc<dyn LabelDataBase>                  │
│ 调用 LabelDataBase 的方法                     │
└─────────────────────┬────────────────────────┘
                      │
                      ▼
┌──────────────────────────────────────────────┐
│          LabelDataBase (trait)                │
│         feel_storage::database::label         │
│          标签数据访问抽象                      │
└─────────────────────┬────────────────────────┘
                      │ impl
                      ▼
┌──────────────────────────────────────────────┐
│     CommonLabelDataBase (struct) — ★ 本文件   │
│       持有 Box<dyn LabelRepo>                 │
│       全部方法 → 委托 LabelRepo               │
│       add_label 含数量上限校验                │
└─────────────────────┬────────────────────────┘
                      │ delegate
                      ▼
┌──────────────────────────────────────────────┐
│            LabelRepo (trait)                  │
│          feel_storage::repo::label            │
│          持久化层抽象                         │
└─────────────────────┬────────────────────────┘
                      │ impl
                      ▼
┌──────────────────────────────────────────────┐
│          SeaOrmLabelRepo (struct)             │
│          Sea-ORM 数据库实现                   │
└──────────────────────────────────────────────┘
```

### 依赖关系

| 依赖方向 | 说明 |
| --- | --- |
| 上层使用 | `feel_api` 通过 `Arc<dyn LabelDataBase>` 持有 `CommonLabelDataBase` |
| 内部委托 | `CommonLabelDataBase` 委托给 `dyn LabelRepo` |
| 底层实现 | `LabelRepo` 的默认实现为 `SeaOrmLabelRepo` |

---

## 与 LabelRepo 的对比

`CommonLabelDataBase` 和 `LabelRepo` 是两个不同职责层的 struct/trait：

| 维度 | CommonLabelDataBase | LabelRepo |
| --- | --- | --- |
| 所属模块 | `database::label` | `repo::label` |
| 角色 | `LabelDataBase` 的默认实现 | 持久化层抽象接口 |
| 定位 | 数据访问层（供业务层直接调用） | 存储实现层（供 database 层委托） |
| 业务校验 | ✅ 是（`add_label` 数量上限 20） | ❌ 否（不校验，由上层负责） |
| 使用方 | `feel_api` 等业务代码 | `CommonLabelDataBase` |
| 创建方式 | `CommonLabelDataBase::new(repo)` | 具体实现如 `SeaOrmLabelRepo::new(conn)` |

---

## 使用示例

### 在 AppState 中使用

```rust
use std::sync::Arc;
use feel_storage::database::{LabelDataBase, CommonLabelDataBase};
use feel_storage::repo::SeaOrmLabelRepo;
use sea_orm::DatabaseConnection;

let conn: DatabaseConnection = /* 获取数据库连接 */;
let repo = SeaOrmLabelRepo::new(conn);
let label_db: Arc<dyn LabelDataBase> = Arc::new(CommonLabelDataBase::new(repo));

// 创建标签本体
let label = label_db
    .create_label(&LabelCreate {
        name: "Rust 开发者".into(),
        description: "使用 Rust 进行开发的人".into(),
        remark: String::new(),
        influence: 50,
    })
    .await
    .unwrap();

// 为用户添加标签关联(内部校验数量上限 20)
label_db.add_label(1, label.id).await.unwrap();

// 查看用户的标签
let user_labels = label_db.get_user_labels(1).await.unwrap();

// 解除关联
label_db.remove_label(1, label.id).await.unwrap();
```

### 在切换存储实现时的灵活性

由于 `CommonLabelDataBase` 接受任何 `T: LabelRepo`，替换底层存储不需要修改业务代码：

```rust
// 假设有一个内存实现 MemoryLabelRepo
let repo = MemoryLabelRepo::new();
let label_db = CommonLabelDataBase::new(repo);
// 业务代码不变，只需替换 repo 即可
```

---

## 当前状态

| 方法 | 实现状态 | 说明 |
| --- | --- | --- |
| `create_label` | ✅ 完整 | async 委托至 `LabelRepo::create_label` |
| `update_label` | ✅ 完整 | async 委托至 `LabelRepo::update_label` |
| `get_label` | ✅ 完整 | async 委托至 `LabelRepo::find_label_by_id` |
| `get_label_by_name` | ✅ 完整 | async 委托至 `LabelRepo::find_label_by_name` |
| `get_user_labels` | ✅ 完整 | async 委托至 `LabelRepo::find_user_labels` |
| `get_users_by_label` | ✅ 完整 | async 委托至 `LabelRepo::find_users_by_label` |
| `count_user_labels` | ✅ 完整 | async 委托至 `LabelRepo::count_user_labels` |
| `add_label` | ✅ 完整 | 数量上限校验 + 委托 `create_user_label` |
| `remove_label` | ✅ 完整 | async 委托至 `LabelRepo::delete_user_label_by_ids` |

> `CommonLabelDataBase` 与 `LabelDataBase` trait 均实现于 `crates/feel_storage/src/database/label.rs`，已随 `database` 模块导出。

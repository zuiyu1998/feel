# LabelEntity — label 表 ORM 实体

## 概述

`LabelEntity` 是 `feel_sea_orm` crate 中定义的 **Sea-ORM 实体**，对应数据库中的 `label` 表（标签本体，公有资源），位于 `crates/feel_sea_orm/src/label/entities/label.rs`。

该实体通过 `DeriveEntityModel` 派生 ORM 模型，并提供 `LabelModel → LabelBase` 的类型转换，供 `feel_storage` 的 `SeaOrmLabelRepo` 使用。

## 模块位置

```
crates/feel_sea_orm/src/label/entities/
├── mod.rs        # 模块声明
├── prelude.rs    # 重新导出（LabelActiveModel / LabelColumn / LabelEntity / LabelModel）
├── label.rs      # label 表实体 ←
└── user_label.rs # user_label 表实体
```

### 重新导出路径

```rust
// 同模块 prelude
use feel_sea_orm::label::entities::prelude::*;

// 或顶级 prelude
use feel_sea_orm::prelude::*;

// 可用的别名：
//   LabelActiveModel  — ActiveModel
//   LabelColumn       — Column
//   LabelEntity       — Entity
//   LabelModel        — Model
```

## 数据模型（Model）

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "label")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    #[sea_orm(unique, index)]
    pub name: String,
    pub description: String,
    pub remark: String,
    pub influence: i64,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 字段说明

| 字段 | 类型 | Sea-ORM 属性 | 说明 |
|------|------|--------------|------|
| `id` | `i64` | `primary_key, auto_increment = false` | 主键，写入时 `NotSet` 交由数据库序列生成 |
| `name` | `String` | `unique, index` | 标签名（全局唯一标识） |
| `description` | `String` | — | 公共描述（默认空串） |
| `remark` | `String` | — | 标签备注（所有用户共享） |
| `influence` | `i64` | — | 影响力，创建时确立，之后不变 |
| `enabled` | `bool` | — | 是否启用（默认 true） |
| `created_at` | `DateTime<Utc>` | — | 创建时间（`TIMESTAMPTZ`） |
| `updated_at` | `DateTime<Utc>` | — | 最后更新时间（`TIMESTAMPTZ`） |

### DDL 对应（PostgreSQL，由 migration 创建）

```sql
CREATE TABLE label (
    id          BIGSERIAL    PRIMARY KEY,
    name        VARCHAR      NOT NULL UNIQUE,
    description VARCHAR      NOT NULL DEFAULT '',
    remark      VARCHAR      NOT NULL DEFAULT '',
    influence   BIGINT       NOT NULL,
    enabled     BOOLEAN      NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ  NOT NULL,
    updated_at  TIMESTAMPTZ  NOT NULL
);
```

## 关系定义（Relation）

```rust
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
```

当前**未定义 ORM 关系**。`user_label` 与 `label` 的关联通过 `user_label.label_id` 逻辑承载，ORM 层不建外键约束（见 [UserLabelEntity](UserLabelEntity.md)）。

## 类型转换：LabelModel → LabelBase

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
```

字段一一对应；`UserLabel` 关联信息的组装由 `feel_storage` 层完成（如 `find_user_labels` 二次查询 `find_label_by_id`）。

## 使用示例

```rust
use feel_sea_orm::prelude::*;
use sea_orm::*;

// 插入标签（id 由数据库序列生成）
let label = LabelActiveModel {
    id: NotSet,
    name: Set("Rust 开发者".into()),
    description: Set("Rust 后端开发者".into()),
    remark: Set("5 年 Rust 后端开发经验".into()),
    influence: Set(5),
    enabled: Set(true),
    created_at: Set(Utc::now()),
    updated_at: Set(Utc::now()),
}.insert(&db).await?;

// 按主键查询
let label = LabelEntity::find_by_id(1).one(&db).await?;

// 按名称查询
let label = LabelEntity::find()
    .filter(LabelColumn::Name.eq("Rust 开发者"))
    .one(&db)
    .await?;

// 转换为领域模型
let label_base: LabelBase = label.into();
```

## 依赖关系

| 依赖 | 用途 |
|------|------|
| `sea-orm` | ORM 框架（实体派生、查询） |
| `chrono` | 日期时间类型 `DateTime<Utc>` |
| `feel_entity` | 领域模型 `LabelBase` |

## 当前状态

| 项 | 状态 | 说明 |
|----|------|------|
| 字段定义 | ✅ | 8 个字段完整定义 |
| 索引 | ✅ | `name` 唯一索引 |
| Relation | 🚧 空 | 未定义 ORM 关系 |
| From 转换 | ✅ | LabelModel → LabelBase |
| 迁移建表 | ✅ | `m20260209_000000_add_labels` |

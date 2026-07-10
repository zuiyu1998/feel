# UserEntity — users 表 ORM 实体

## 概述

`UserEntity` 是 `feel_sea_orm` crate 中定义的 **Sea-ORM 实体**，对应数据库中的 `users` 表，位于 `crates/feel_sea_orm/src/user/entities/user.rs`。

该实体通过 Sea-ORM 的 `DeriveEntityModel` 派生宏自动生成 ORM 操作模型，并提供 `UserModel → UserBase` 的类型转换。

## 模块位置

```
crates/feel_sea_orm/src/user/entities/
├── mod.rs              # 模块声明
├── prelude.rs          # 重新导出（UserActiveModel / UserEntity / UserModel）
├── user.rs             # users 表实体 ←
└── user_credentials.rs # user_credentials 表实体
```

### 重新导出路径

```rust
// 通过 prelude 使用
use feel_sea_orm::user::entities::prelude::*;

// 或直接使用顶级 prelude
use feel_sea_orm::prelude::*;

// 可用的别名：
//   UserActiveModel  — ActiveModel
//   UserEntity       — Entity
//   UserModel        — Model
```

---

## 模块结构

```rust
// -- Model：数据模型（对应表行）--
// -- Relation：关系定义（当前为空）--
// -- ActiveModelBehavior：ActiveModel 行为 --
// -- From<Model> for UserBase：类型转换 --
```

---

## 数据模型（Model）

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,

    #[sea_orm(unique, index)]
    pub uid: String,

    pub name: String,
    pub avatar: String,
    pub slogan: String,
    pub is_enable: bool,
    pub is_delete: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 字段说明

| 字段         | 类型               | Sea-ORM 属性                          | 说明                       |
| ------------ | ------------------ | ------------------------------------- | -------------------------- |
| `id`         | `i64`              | `#[sea_orm(primary_key, auto_increment = false)]` | 主键，非自增（由 typedflake 生成） |
| `uid`        | `String`           | `#[sea_orm(unique, index)]`           | 唯一索引，用户全局唯一标识 |
| `name`       | `String`           | —                                     | 用户展示名                 |
| `avatar`     | `String`           | —                                     | 用户头像 URL               |
| `slogan`     | `String`           | —                                     | 用户签名/简介              |
| `is_enable`  | `bool`             | —                                     | 是否启用                   |
| `is_delete`  | `bool`             | —                                     | 软删除标记                 |
| `created_at` | `DateTime<Utc>`    | —                                     | 记录创建时间               |
| `updated_at` | `DateTime<Utc>`    | —                                     | 最后更新时间               |

### DDL 对应（PostgreSQL）

```sql
CREATE TABLE users (
    id         BIGINT       PRIMARY KEY,
    uid        VARCHAR      NOT NULL UNIQUE,
    name       VARCHAR      NOT NULL,
    avatar     VARCHAR      NOT NULL,
    slogan     VARCHAR      NOT NULL DEFAULT '',
    is_enable  BOOLEAN      NOT NULL DEFAULT false,
    is_delete  BOOLEAN      NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ  NOT NULL,
    updated_at TIMESTAMPTZ  NOT NULL
);

CREATE INDEX idx_users_uid ON users (uid);
```

### auto_increment = false 说明

`id` 字段设置了 `auto_increment = false`，表示主键不由数据库自增生成，而是由应用层通过 `typedflake` 分布式 ID 生成器在代码中指定，以保证分布式环境下的唯一性。

---

## 关系定义（Relation）

```rust
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
```

当前 `users` 表**未定义任何 Sea-ORM 关系**。这意味着：

- 无法通过 `UserEntity` 直接链式加载关联数据（如 `user_credentials`）
- 关联查询需手动通过 `user_uid` 字段进行

> **注意：** `user_credentials` 表中的 `user_uid` 字段在逻辑上关联 `users.uid`，但 ORM 层面暂未建立 `ForeignKey` 关系。

---

## ActiveModel 行为

```rust
impl ActiveModelBehavior for ActiveModel {}
```

`ActiveModelBehavior` 使用默认实现，没有自定义的钩子逻辑（如 `before_save`、`after_save` 等）。

---

## 类型转换：UserModel → UserBase

```rust
impl From<Model> for UserBase {
    fn from(value: Model) -> Self {
        UserBase {
            id: value.id,
            uid: value.uid,
            name: value.name,
            avatar: value.avatar,
            slogan: value.slogan,
            is_enable: value.is_enable,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
```

实现了 `From<UserModel> for UserBase`，允许通过 `.into()` 或 `From::from()` 将 ORM 模型转换为业务模型：

```rust
let user_model: UserModel = /* 从数据库查询 */;
let user_base: UserBase = user_model.into();
```

### 字段映射

| UserModel 字段  | →  | UserBase 字段  | 说明                         |
| --------------- | -- | -------------- | ---------------------------- |
| `id`            | →  | `id`           | 直接映射                     |
| `uid`           | →  | `uid`          | 直接映射                     |
| `name`          | →  | `name`         | 直接映射                     |
| `avatar`        | →  | `avatar`       | 直接映射                     |
| `slogan`        | →  | `slogan`       | 直接映射                     |
| `is_enable`  | →  | `enabled`      | 直接映射                     |
| `is_delete`     | →  | —              | 软删除标记（UserBase 无对应） |
| `created_at`    | →  | `created_at`   | 直接映射                     |
| `updated_at`    | →  | `updated_at`   | 直接映射                     |

---

## 使用示例

```rust
use feel_sea_orm::prelude::*;
use sea_orm::*;

// 插入新用户
let user = UserActiveModel {
    id: Set(12345),
    uid: Set("uid_abc123".into()),
    name: Set("Alice".into()),
    avatar: Set("https://example.com/avatar.png".into()),
    slogan: Set("Hello!".into()),
    is_enable: Set(true),
    created_at: Set(Utc::now()),
    updated_at: Set(Utc::now()),
}.insert(&db).await?;

// 按主键查询
let user = UserEntity::find_by_id(12345).one(&db).await?;

// 按 uid 查询
let user = UserEntity::find()
    .filter(UserColumn::Uid.eq("uid_abc123"))
    .one(&db)
    .await?;

// 转换为业务模型
let user_base: UserBase = user.into();
```

---

## 依赖关系

| 依赖            | 用途                        |
| --------------- | --------------------------- |
| `sea-orm`       | ORM 框架（实体派生、查询）  |
| `chrono`        | 日期时间类型 `DateTime<Utc>` |
| `feel_entity`   | 业务模型 `UserBase`         |
| `serde`         | 序列化支持                  |
| `serde_json`    | JSON 序列化                  |

---

## 当前状态

| 项                  | 状态 | 说明                           |
| ------------------- | ---- | ------------------------------ |
| 字段定义            | ✅    | 9 个字段完整定义               |
| 主键策略            | ✅    | 非自增，由 typedflake 生成     |
| 索引                | ✅    | uid 字段唯一索引               |
| Relation            | 🚧 空 | 未定义 ORM 关系                |
| ActiveModelBehavior | ✅    | 默认实现，无自定义钩子         |
| From 转换           | ✅    | UserModel → UserBase           |

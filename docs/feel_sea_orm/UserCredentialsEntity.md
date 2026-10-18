# UserCredentialsEntity — user_credentials 表 ORM 实体

## 概述

`UserCredentialsEntity` 是 `feel_sea_orm` crate 中定义的 **Sea-ORM 实体**，对应数据库中的 `user_credentials` 表（用户登录凭据，加密存储），位于 `crates/feel_sea_orm/src/user/entities/user_credentials.rs`。

该表通过 `user_uid` 逻辑关联 `users.uid`（ORM 层未建 `ForeignKey`，见 [UserEntity](UserEntity.md) 的说明）。当前**没有** `From<Model>` 业务转换——`feel_entity` 中未定义对应领域模型，`SeaOrmUserRepo` 在注册时直接写入凭据行。

## 模块位置

```
crates/feel_sea_orm/src/user/entities/
├── mod.rs              # 模块声明
├── prelude.rs          # 重新导出（UserCredentialsActiveModel / Column / Entity / Model）
├── user.rs             # users 表实体
└── user_credentials.rs # user_credentials 表实体 ←
```

### 重新导出路径

```rust
use feel_sea_orm::user::entities::prelude::*;  // 或 feel_sea_orm::prelude::*

// 可用的别名：
//   UserCredentialsActiveModel  — ActiveModel
//   UserCredentialsColumn       — Column
//   UserCredentialsEntity       — Entity
//   UserCredentialsModel        — Model
```

## 数据模型（Model）

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_credentials")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    #[sea_orm(index)]
    pub user_uid: String,
    #[sea_orm(index)]
    pub credential_type: String,
    pub credential_name: String,
    #[sea_orm(select, insert, update)]
    pub encrypted_data: String,
    pub encryption_key: String,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 字段说明

| 字段 | 类型 | Sea-ORM 属性 | 说明 |
|------|------|--------------|------|
| `id` | `i64` | `primary_key, auto_increment = false` | 主键，写入时 `NotSet` 交由数据库序列生成 |
| `user_uid` | `String` | `index` | 用户业务键（逻辑关联 `users.uid`） |
| `credential_type` | `String` | `index` | 凭据类型，如 `password` / `email` / `phone` |
| `credential_name` | `String` | — | 凭据名/标识 |
| `encrypted_data` | `String` | `select, insert, update` | 加密后的凭据数据（`select` 表示查询时也包含该列） |
| `encryption_key` | `String` | — | 加密密钥标识（如加盐） |
| `is_enabled` | `bool` | — | 凭据是否启用（默认 true） |
| `created_at` | `DateTime<Utc>` | — | 创建时间（`TIMESTAMPTZ`） |
| `updated_at` | `DateTime<Utc>` | — | 最后更新时间（`TIMESTAMPTZ`） |

`#[sea_orm(select, insert, update)]` 表示 `encrypted_data` 默认参与查询/插入/更新，避免被误遗漏。

### DDL 对应（PostgreSQL，由 migration 创建）

```sql
CREATE TABLE user_credentials (
    id              BIGSERIAL    PRIMARY KEY,
    user_uid        VARCHAR      NOT NULL,
    credential_type VARCHAR      NOT NULL,
    credential_name VARCHAR      NOT NULL,
    encrypted_data  VARCHAR      NOT NULL,
    encryption_key  VARCHAR      NOT NULL,
    is_enabled      BOOLEAN      NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ  NOT NULL,
    updated_at      TIMESTAMPTZ  NOT NULL
);

-- 同一用户的同一凭据名唯一
CREATE UNIQUE INDEX idx_user_credentials_user_uid_credential_name
    ON user_credentials (user_uid, credential_name);
```

> 实体上的 `#[sea_orm(index)]` 属性（`user_uid`、`credential_type`）为 SeaORM 实体元信息，实际 DDL 以迁移为准（迁移中仅建上述组合唯一索引）。

## 关系定义（Relation）

```rust
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
```

**未定义 ORM 关系 / 外键约束**：`user_uid` 与 `users.uid` 为逻辑外键，关联查询需手动按 `user_uid` 过滤。

## 类型转换

当前**未实现** `From<Model>` 转换（`feel_entity` 无对应领域模型）。注册流程（`SeaOrmUserRepo::register`）中直接构造 `UserCredentialsActiveModel` 写入：

```rust
let mut credential = UserCredentialsActiveModel::default();
credential.encryption_key = Set(salt);
credential.encrypted_data = Set(password);   // 已哈希的凭据
credential.credential_type = Set(register.credential_type.to_string());
credential.credential_name = Set(register.credential_name.to_string());
credential.user_uid = Set(user.uid.clone());
credential.created_at = Set(now);
credential.updated_at = Set(now);
credential.insert(&self.conn).await?;
```

## 使用示例

```rust
use feel_sea_orm::prelude::*;
use sea_orm::*;

// 按用户查询凭据
let creds = UserCredentialsEntity::find()
    .filter(UserCredentialsColumn::UserUid.eq("uid_abc123"))
    .all(&db)
    .await?;

// 按凭据名定位（登录校验用）
let cred = UserCredentialsEntity::find()
    .filter(UserCredentialsColumn::UserUid.eq("uid_abc123"))
    .filter(UserCredentialsColumn::CredentialName.eq("password"))
    .one(&db)
    .await?;
```

## 依赖关系

| 依赖 | 用途 |
|------|------|
| `sea-orm` | ORM 框架（实体派生、查询） |
| `chrono` | 日期时间类型 `DateTime<Utc>` |

## 当前状态

| 项 | 状态 | 说明 |
|----|------|------|
| 字段定义 | ✅ | 9 个字段完整定义 |
| 唯一索引 | ✅ | `(user_uid, credential_name)` 联合唯一 |
| Relation / 外键 | 🚧 逻辑关联 | 无 ORM 关系、无 FK 约束 |
| From 转换 | — | 无对应领域模型，直接由 Repo 写入 |
| 迁移建表 | ✅ | `m20260208_000000_add_user_credentials` |

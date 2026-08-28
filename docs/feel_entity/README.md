# feel_entity — 领域实体定义

## 概述

`feel_entity` 是 feel 平台的**领域实体 crate**，定义核心业务数据结构（DTO / 领域模型），位于 `crates/feel_entity/`。

所有实体均为纯数据结构（plain struct），不含任何业务逻辑、持久化逻辑或 HTTP 序列化逻辑。这使得实体层可以同时被 `feel_storage`（数据层）和 `feel_api`（展现层）依赖，而不会引入循环依赖。

## 模块位置

```
crates/feel_entity/src/
├── lib.rs               # 模块声明 + prelude
├── user/
│   ├── mod.rs            # models 子模块重新导出
│   └── models/
│       └── mod.rs        # 用户相关实体（UserBase 等）
└── label/
    ├── mod.rs            # models 子模块重新导出
    └── models/
        └── mod.rs        # 标签实体（LabelBase、UserLabel）
```

## 依赖关系

| 依赖 | 版本 | 用途 |
|------|------|------|
| `feel_core` | workspace | 通过 `feel_core::chrono` 获取时间类型 |
| `serde` | 1.0 (derive) | `Serialize` / `Deserialize` 派生 |

`feel_core` 本身只是 `chrono` 和 `tokio` 的重新导出（re-export），所以 `feel_entity` 的间接依赖链为：

```
feel_entity
  └── feel_core
        └── chrono (DateTime<Utc> 等时间类型)
```

## 模块：`user`

所有用户相关实体定义在 `feel_entity::user` 模块下，通过 `feel_entity::prelude` 也可访问。

### 模块结构

```
feel_entity::user
  ├── UserBase         — 用户基础信息
  ├── UserRegister     — 用户注册请求
  ├── UserLogin        — 用户登录请求
  ├── LoginResult      — 登录成功结果
  ├── UserUpdate       — 用户更新请求（占位）
  └── UserCredential   — 用户凭据
```

### UserBase

用户基础信息，作为注册/登录/查询的返回类型。

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserBase {
    pub id: i64,
    pub uid: String,
    pub name: String,
    pub avatar: String,
    pub slogan: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `i64` | 数据库主键 |
| `uid` | `String` | 全局唯一用户标识（UUID 风格字符串），分布式友好 |
| `name` | `String` | 用户展示名 |
| `avatar` | `String` | 用户头像 URL |
| `slogan` | `String` | 用户签名/简介 |
| `enabled` | `bool` | 是否启用 |
| `created_at` | `DateTime<Utc>` | 记录创建时间 |
| `updated_at` | `DateTime<Utc>` | 记录更新时间 |

**JSON 示例：**

```json
{
    "id": 1,
    "uid": "uid_abc123",
    "name": "张三",
    "avatar": "https://example.com/avatar.png",
    "slogan": "Hello!",
    "enabled": true,
    "created_at": "2026-07-11T03:00:00Z",
    "updated_at": "2026-07-11T03:00:00Z"
}
```

### UserRegister

用户注册请求体，同时包含用户信息和凭据信息。

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserRegister {
    pub name: String,
    pub avatar: String,
    pub credential_type: String,
    pub credential_name: String,
    pub data: String,
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | `String` | 用户展示名（必填） |
| `avatar` | `String` | 用户头像 URL（必填） |
| `credential_type` | `String` | 凭据类型（如 `"password"`、`"email"`、`"phone"`） |
| `credential_name` | `String` | 凭据标识（如邮箱地址、手机号） |
| `data` | `String` | 凭据数据（将被加密存储，如密码原文） |

**JSON 示例：**

```json
{
    "name": "张三",
    "avatar": "https://example.com/avatar.png",
    "credential_type": "password",
    "credential_name": "alice@example.com",
    "data": "my_secret_password"
}
```

### UserLogin

用户登录请求体，用于凭据验证。

```rust
#[derive(Debug, Clone)]
pub struct UserLogin {
    pub credential_name: String,
    pub data: String,
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `credential_name` | `String` | 凭据标识（如邮箱地址、手机号） |
| `data` | `String` | 凭据数据（如密码原文） |

> **注意：** `UserLogin` 没有派生 `Serialize`/`Deserialize`，因为它目前仅在服务端内部传递，不直接参与 JSON 序列化。API 层使用对应的 DTO（`feel_api::model::user::LoginRequest`）进行序列化。

**JSON 示例（对应 API DTO）：**

```json
{
    "credential_name": "alice@example.com",
    "data": "my_password"
}
```

### LoginResult

登录成功后的返回结果，包含认证令牌和用户数据。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResult {
    pub token: String,
    pub user_base: UserBase,
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `token` | `String` | JWT 认证令牌 |
| `user_base` | `UserBase` | 登录用户完整数据 |

**JSON 示例：**

```json
{
    "token": "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ1...",
    "user_base": {
        "id": 1,
        "uid": "uid_abc123",
        "name": "张三",
        "avatar": "https://example.com/avatar.png",
        "slogan": "",
        "enabled": true,
        "created_at": "2026-07-11T03:00:00Z",
        "updated_at": "2026-07-11T03:00:00Z"
    }
}
```

### UserUpdate

用户更新请求体（当前为占位类型）。

```rust
pub struct UserUpdate {}
```

> **当前状态：** 空结构体，尚无字段。待更新功能实现时补充。

### UserCredential

用户凭据实体，用于认证和密码管理。

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserCredential {
    pub id: i64,
    pub user_uid: String,
    pub credential_type: String,
    pub credential_name: String,
    pub encrypted_data: String,
    pub encryption_key: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `i64` | 数据库主键 |
| `user_uid` | `String` | 用户唯一标识（关联 `UserBase.uid`） |
| `credential_type` | `String` | 凭据类型（如 `"password"`、`"email"`、`"phone"`） |
| `credential_name` | `String` | 凭据标识（如邮箱地址、手机号） |
| `encrypted_data` | `String` | 加密后的凭据数据 |
| `encryption_key` | `String` | 加密密钥标识 |
| `enabled` | `bool` | 凭据是否启用 |
| `created_at` | `DateTime<Utc>` | 记录创建时间 |
| `updated_at` | `DateTime<Utc>` | 记录更新时间 |

**JSON 示例：**

```json
{
    "id": 1,
    "user_uid": "uid_abc123",
    "credential_type": "password",
    "credential_name": "alice@example.com",
    "encrypted_data": "$argon2id$v=19$m=65536,t=3,p=4$...",
    "encryption_key": "salt_value_here",
    "enabled": true,
    "created_at": "2026-07-11T03:00:00Z",
    "updated_at": "2026-07-11T03:00:00Z"
}
```

## 模块：`label`

标签相关实体定义在 `feel_entity::label` 模块下，通过 `feel_entity::prelude` 也可访问。领域模型与特性设计文档 `docs/features/label.md` 保持一致。

### 模块结构

```
feel_entity::label
  ├── LabelBase        — 标签本体（公有，多个用户共享）
  ├── UserLabel        — 用户标签关联（用户 × 标签，多对多）
  ├── LabelCreate      — 创建标签请求（`LabelRepo::create_label` 参数）
  ├── LabelUpdate      — 更新标签请求（`LabelRepo::update_label` 参数）
  └── UserLabelCreate  — 添加关联请求（`LabelRepo::create_user_label` 参数）
```

### LabelBase — 标签本体（公有）

标签本体，是多个用户所公有的标签实体，用于刻画用户特征；名称全局唯一，备注对所有用户共享。

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LabelBase {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub remark: String,
    pub influence: i64,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `i64` | 数据库主键 |
| `name` | `String` | 标签名称（全局唯一标识） |
| `description` | `String` | 标签的公共描述，帮助所有人理解该标签的含义 |
| `remark` | `String` | 标签备注，对标签含义的补充说明，所有用户共享 |
| `influence` | `i64` | 标签影响力，在创建标签时确立，之后不再变更 |
| `enabled` | `bool` | 是否启用 |
| `created_at` | `DateTime<Utc>` | 记录创建时间 |
| `updated_at` | `DateTime<Utc>` | 记录更新时间 |

**JSON 示例：**

```json
{
    "id": 1,
    "name": "Rust 开发者",
    "description": "使用 Rust 进行开发的人",
    "remark": "5 年 Rust 后端开发经验",
    "influence": 100,
    "enabled": true,
    "created_at": "2026-07-11T03:00:00Z",
    "updated_at": "2026-07-11T03:00:00Z"
}
```

### UserLabel — 用户标签关联

用户与标签的**多对多**关联记录，仅承载"用户 × 标签"关系；备注等个性化内容位于 `LabelBase` 上，不在此处。

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserLabel {
    pub id: i64,
    pub user_id: i64,
    pub label_id: i64,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `i64` | 数据库主键 |
| `user_id` | `i64` | 归属用户标识，关联 `UserBase.id` |
| `label_id` | `i64` | 关联的标签本体，关联 `LabelBase.id` |
| `enabled` | `bool` | 是否启用 |
| `created_at` | `DateTime<Utc>` | 记录创建时间 |
| `updated_at` | `DateTime<Utc>` | 记录更新时间 |

**JSON 示例：**

```json
{
    "id": 10,
    "user_id": 1,
    "label_id": 1,
    "enabled": true,
    "created_at": "2026-07-11T03:00:00Z",
    "updated_at": "2026-07-11T03:00:00Z"
}
```

### LabelCreate

创建标签本体的请求结构体，作为 `LabelRepo::create_label` 的参数。`id`、`enabled`（默认 `true`）与时间戳由存储层管理，调用方无需提供。

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LabelCreate {
    pub name: String,
    pub description: String,
    pub remark: String,
    pub influence: i64,
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | `String` | 标签名称（全局唯一标识） |
| `description` | `String` | 公共描述 |
| `remark` | `String` | 标签备注（所有用户共享） |
| `influence` | `i64` | 标签影响力，创建时确立 |

**JSON 示例：**

```json
{
    "name": "Rust 开发者",
    "description": "使用 Rust 进行开发的人",
    "remark": "",
    "influence": 50
}
```

### LabelUpdate

更新标签本体的请求结构体，作为 `LabelRepo::update_label` 的参数。携带主键 `id` 与全部可写字段（含 `enabled`，用于禁用/启用）。

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LabelUpdate {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub remark: String,
    pub influence: i64,
    pub enabled: bool,
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `i64` | 待更新标签本体的主键 |
| `name` | `String` | 标签名称 |
| `description` | `String` | 公共描述 |
| `remark` | `String` | 标签备注 |
| `influence` | `i64` | 标签影响力 |
| `enabled` | `bool` | 是否启用 |

**JSON 示例：**

```json
{
    "id": 1,
    "name": "Rust 开发者",
    "description": "使用 Rust 进行开发的人",
    "remark": "5 年 Rust 后端开发经验",
    "influence": 100,
    "enabled": true
}
```

### UserLabelCreate

创建用户标签关联的请求结构体，作为 `LabelRepo::create_user_label` 的参数。`id`、`enabled`（默认 `true`）与时间戳由存储层管理。

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserLabelCreate {
    pub user_id: i64,
    pub label_id: i64,
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `user_id` | `i64` | 归属用户，关联 `UserBase.id` |
| `label_id` | `i64` | 关联的标签本体，关联 `LabelBase.id` |

**JSON 示例：**

```json
{
    "user_id": 1,
    "label_id": 1
}
```

## 重新导出与 Prelude

### 导出路径

| 完整路径 | 类型 |
|---------|------|
| `feel_entity::user::UserBase` | struct |
| `feel_entity::user::UserRegister` | struct |
| `feel_entity::user::UserLogin` | struct |
| `feel_entity::user::LoginResult` | struct |
| `feel_entity::user::UserUpdate` | struct |
| `feel_entity::user::UserCredential` | struct |
| `feel_entity::label::LabelBase` | struct |
| `feel_entity::label::UserLabel` | struct |
| `feel_entity::label::LabelCreate` | struct |
| `feel_entity::label::LabelUpdate` | struct |
| `feel_entity::label::UserLabelCreate` | struct |

### Prelude

```rust
// src/lib.rs
pub mod prelude {
    pub use crate::label::*;
    pub use crate::user::*;
}
```

使用方式：

```rust
use feel_entity::prelude::*;   // 导入所有实体（user + label）
// 等价于：
use feel_entity::user::{UserBase, UserRegister, UserLogin, LoginResult, UserUpdate, UserCredential};
use feel_entity::label::{LabelBase, LabelCreate, LabelUpdate, UserLabel, UserLabelCreate};
```

## 类型关系图

```
UserRegister ──────────────────────────────────────────┐
  (注册时同时创建用户 + 凭据)                             │
                                                         │
┌─────────────────────┐      ┌─────────────────────────┐ │
│     UserBase         │      │    UserCredential        │ │
│─────────────────────│      │─────────────────────────│ │
│ id: i64              │      │ id: i64                  │ │
│ uid: String (唯一键) │◄────►│ user_uid: String (FK)   │ │
│ name: String         │      │ credential_type: String  │ │
│ avatar: String       │      │ credential_name: String  │ │
│ slogan: String       │      │ encrypted_data: String   │ │
│ enabled: bool        │      │ encryption_key: String   │ │
│ created_at: DateTime │      │ enabled: bool            │ │
│ updated_at: DateTime │      │ created_at: DateTime     │ │
└─────────────────────┘      │ updated_at: DateTime      │
        ▲                     └─────────────────────────┘
        │
┌───────┴──────────────┐
│    LoginResult        │
│──────────────────────│
│ token: String         │
│ user_base: UserBase   │
└──────────────────────┘

UserLogin ──────────────► 验证 → LoginResult
  (credential_name + data)

UserUpdate (空占位)
```

### 标签关系

标签与用户为**多对多（M:N）** 关系，通过 `UserLabel` 关联表承载：

```
LabelBase (1) ──────── (N) UserLabel (N) ──────── (1) UserBase
  id                            user_id                    id
                                label_id

一个用户 ──拥有──► 多个标签（UserLabel）
一个标签 ──对应──► 多个用户（UserLabel）
```

- **正向（用户 → 标签）**：一个用户可以关联多个标签，这些关联构成其自我画像
- **反向（标签 → 用户）**：一个标签可以被多个用户关联，按标签即可找到使用它的用户群
- 标签本体是公有的，不属于任何单个用户
- 备注挂在标签本体上：所有用户看到同一份标签备注

## 与 ORM 实体的关系

`feel_entity` 中的类型是**纯领域模型**，与 `feel_sea_orm` 中的 ORM 实体（Sea-ORM `DeriveEntityModel`）分开定义：

| feel_entity 领域模型 | feel_sea_orm ORM 实体 | 关系 |
|---------------------|----------------------|------|
| `UserBase` | `feel_sea_orm::user::entities::user::Model` | `Model` 实现了 `From<Model> for UserBase` |
| `UserCredential` | `feel_sea_orm::user::entities::user_credentials::Model` | `Model` 实现了 `From<Model> for UserCredential` |
| `UserRegister` | — | 组合了 UserBase 和 UserCredential 的注册字段 |
| `UserLogin` | — | 纯业务请求 |
| `LoginResult` | — | 业务返回结果 |
| `UserUpdate` | — | 占位 |
| `LabelBase` | `feel_sea_orm::label::entities::label::Model` | `Model` 实现了 `From<Model> for LabelBase` |
| `UserLabel` | `feel_sea_orm::label::entities::user_label::Model` | `Model` 实现了 `From<Model> for UserLabel` |
| `LabelCreate` | — | 创建标签请求（`LabelRepo::create_label` 参数） |
| `LabelUpdate` | — | 更新标签请求（`LabelRepo::update_label` 参数） |
| `UserLabelCreate` | — | 添加关联请求（`LabelRepo::create_user_label` 参数） |

领域层与 ORM 层的分离使得：
- 数据库表结构变化不影响业务层（通过 `From` 转换隔离）
- 业务类型可以组合多个 ORM 实体的数据
- API 层可以依赖领域模型而不引入 ORM 依赖

## 使用示例

```rust
use feel_entity::prelude::*;

// 注册用户
let register = UserRegister {
    name: "Alice".into(),
    avatar: "https://example.com/avatar.png".into(),
    credential_type: "password".into(),
    credential_name: "alice@example.com".into(),
    data: "secure_password".into(),
};

// 登录
let login = UserLogin {
    credential_name: "alice@example.com".into(),
    data: "secure_password".into(),
};

// 登录结果
let result = LoginResult {
    token: "eyJhbGci...".into(),
    user_base: UserBase {
        id: 1,
        uid: "uid_abc123".into(),
        name: "Alice".into(),
        avatar: "https://example.com/avatar.png".into(),
        slogan: String::new(),
        enabled: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    },
};

// 标签本体
let label = LabelBase {
    id: 1,
    name: "Rust 开发者".into(),
    description: "使用 Rust 进行开发的人".into(),
    remark: "5 年 Rust 后端开发经验".into(),
    influence: 100,
    enabled: true,
    created_at: Utc::now(),
    updated_at: Utc::now(),
};

// 用户标签关联
let user_label = UserLabel {
    id: 10,
    user_id: 1,
    label_id: 1,
    enabled: true,
    created_at: Utc::now(),
    updated_at: Utc::now(),
};

// 创建/更新请求结构体(作为 LabelRepo 方法参数)
let label_create = LabelCreate {
    name: "Rust 开发者".into(),
    description: "使用 Rust 进行开发的人".into(),
    remark: String::new(),
    influence: 50,
};

let label_update = LabelUpdate {
    id: 1,
    name: "Rust 开发者".into(),
    description: "使用 Rust 进行开发的人".into(),
    remark: "5 年 Rust 后端开发经验".into(),
    influence: 100,
    enabled: true,
};

let user_label_create = UserLabelCreate {
    user_id: 1,
    label_id: 1,
};
```

## 设计说明

1. **纯数据结构** — 所有实体均为 struct + pub fields，无方法、无 trait、无业务逻辑
2. **序列化控制** — 需要序列化的类型派生 `Serialize` / `Deserialize`，内部传递的类型（如 `UserLogin`）不派生以保持灵活
3. **字段级文档** — 每个字段都有 `///` doc 注释，可直接生成 API 文档
4. **类型安全** — 使用 `DateTime<Utc>` 而非原始字符串，确保时间操作的类型安全
5. **零依赖原则** — 只依赖 `feel_core`（chrono 包装）和 `serde`，不引入 ORM、HTTP 或存储层依赖

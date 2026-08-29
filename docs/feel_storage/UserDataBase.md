# UserDataBase — 用户数据访问接口

## 概述

`UserDataBase` 是 `feel_storage` crate 中定义的**用户数据访问层 trait**，位于 `crates/feel_storage/src/database/user.rs`。

它抽象了对用户系统（注册、登录、注销、更新、注销）的数据操作，使得业务层无需关心底层存储实现。默认实现为 `CommonUserDataBase`，内部委托给 `UserRepo` trait 完成实际持久化。

## 模块位置

```
crates/feel_storage/src/database/
├── mod.rs          # 重新导出 user 模块
└── user.rs         # UserDataBase trait + CommonUserDataBase 实现
```

## Trait 定义

```rust
#[async_trait]
pub trait UserDataBase: 'static + Send + Sync {
    /// 用户系统注册用户
    async fn register(&self, register: &UserRegister) -> Result<UserBase>;

    /// 用户系统注销用户
    fn unregister(&self, user_id: i64) -> Result<UserBase>;

    /// 用户登录系统
    async fn login(&self, login: &UserLogin) -> Result<LoginResult>;

    /// 用户登出系统
    fn logout(&self, user_id: u32) -> Result<()>;

    /// 用户系统更改个人信息
    fn update(&self, update: &UserUpdate) -> Result<UserBase>;

    // -- JWT token management --

    /// 为用户 uid 生成 JWT token，嵌入 7 天有效期
    fn generate_token(&self, uid: &str) -> Result<String>;

    /// 解析并验证 JWT token，返回其中的 claims
    fn parse_token(&self, token: &str) -> Result<TokenClaims>;

    // -- User query --

    /// 根据用户 UID 获取用户基础信息
    async fn get_user(&self, uid: &str) -> Result<UserBase>;

    /// 根据用户 ID 获取用户基础信息
    async fn get_user_by_id(&self, user_id: i64) -> Result<Option<UserBase>>;
}
```

### Trait 约束

| 约束      | 说明                                   |
| --------- | -------------------------------------- |
| `'static` | 不包含非静态引用，可安全持有           |
| `Send`    | 可跨线程传递所有权                     |
| `Sync`    | 可被多线程共享引用                     |
| `#[async_trait]` | 支持 async fn 方法                |

### 方法说明

| 方法             | 参数                    | 返回                     | 说明                           |
| ---------------- | ----------------------- | ------------------------ | ------------------------------ |
| `register`       | `&UserRegister`         | `Result<UserBase>`       | 注册新用户（**async**）        |
| `unregister`     | `user_id: i64`          | `Result<UserBase>`       | 注销指定用户                   |
| `login`          | `&UserLogin`            | `Result<LoginResult>`    | 用户登录，返回 token+用户数据  |
| `logout`         | `user_id: u32`          | `Result<()>`             | 用户登出                       |
| `update`         | `&UserUpdate`           | `Result<UserBase>`       | 更新用户个人信息               |
| `generate_token` | `uid: &str`             | `Result<String>`         | 为用户生成 JWT token           |
| `parse_token`    | `token: &str`           | `Result<TokenClaims>`    | 解析并验证 JWT token           |
| `get_user`       | `uid: &str`             | `Result<UserBase>`       | 根据 UID 获取用户信息（**async**） |
| `get_user_by_id` | `user_id: i64`          | `Result<Option<UserBase>>` | 根据 ID 获取用户信息（**async**） |

> **注意：** `generate_token` 和 `parse_token` 由 `CommonUserDataBase` 直接实现（使用 JWT HMAC-SHA256），不委托给 `UserRepo`。`get_user` 委托给 `UserRepo::find_by_uid`，`get_user_by_id` 委托给 `UserRepo::find_by_id`。


---

## 默认实现：CommonUserDataBase

```rust
pub struct CommonUserDataBase {
    user_repo: Box<dyn UserRepo>,
}
```

`CommonUserDataBase` 是 `UserDataBase` trait 的默认实现。它通过组合一个 `Box<dyn UserRepo>` 将数据操作委托给具体的存储层。

### 构造方法

```rust
impl CommonUserDataBase {
    pub fn new<T: UserRepo>(user_repo: T) -> Self
}
```

接受任何实现了 `UserRepo` trait 的类型，将其装箱后存储在内部。

### 实现委托关系

| UserDataBase 方法 | 委托至 UserRepo 方法 | 说明              |
| ----------------- | -------------------- | ----------------- |
| `register`        | `register`           | async 传递        |
| `unregister`      | `unregister`         | 直接委托          |
| `login`           | `login`              | 直接委托          |
| `update`          | `update`             | 直接委托          |
| `logout`          | —                    | 无对应，待实现    |
| `generate_token`  | —                    | 内部 JWT 签名实现 |
| `parse_token`     | —                    | 内部 JWT 验证实现 |
| `get_user`        | `find_by_uid`        | async 传递        |
| `get_user_by_id`  | `find_by_id`         | async 传递        |

---

## 架构关系

```
┌──────────────────────────────────────────────────────────┐
│                     App Layer                            │
│  (feel_api / 业务代码)                                    │
│      持有 Arc<dyn UserDataBase>                          │
└─────────────────────┬────────────────────────────────────┘
                      │ 调用 trait 方法
                      ▼
┌──────────────────────────────────────────────────────────┐
│              UserDataBase (trait)                        │
│           feel_storage::database::user                    │
│          用户数据访问抽象                                  │
└─────────────────────┬────────────────────────────────────┘
                      │ 默认实现委托
                      ▼
┌──────────────────────────────────────────────────────────┐
│           CommonUserDataBase (struct)                    │
│              持有 Box<dyn UserRepo>                      │
└─────────────────────┬────────────────────────────────────┘
                      │ 委托调用
                      ▼
┌──────────────────────────────────────────────────────────┐
│              UserRepo (trait)                            │
│           feel_storage::repo::user                        │
│          持久化层抽象                                    │
└─────────────────────────┬────────────────────────────────┘
                          │ 具体实现 (如 SeaOrmUserRepo)
                          ▼
┌──────────────────────────────────────────────────────────┐
│                 数据库 / ORM                               │
└──────────────────────────────────────────────────────────┘
```

### 依赖关系

- **crate 依赖**：`feel_storage` → `feel_entity`（数据模型）
- **trait 依赖**：`UserDataBase` 依赖于 `UserRepo`
- **类型依赖**：`UserRegister`、`UserLogin`、`UserUpdate`、`UserBase` 定义在 `feel_entity::user::models`

---

## 相关数据模型

以下类型定义于 `crates/feel_entity/src/user/models/mod.rs`。

### UserBase

用户基础信息，作为注册/更新/查询的返回类型。

```rust
pub struct UserBase {
    pub id: i64,           // 数据库主键
    pub uid: String,       // 唯一用户标识
    pub name: String,      // 用户展示名
    pub avatar: String,    // 用户头像 URL
    pub slogan: String,    // 用户签名/简介
    pub enabled: bool,     // 是否启用
    pub created_at: DateTime<Utc>,  // 创建时间
    pub updated_at: DateTime<Utc>,  // 更新时间
}
```

### UserRegister

用户注册请求体，同时包含用户信息和凭据信息。

```rust
pub struct UserRegister {
    pub name: String,              // 用户展示名（必填）
    pub avatar: String,            // 用户头像 URL（必填）
    pub credential_type: String,   // 凭据类型（如 "password"/"email"/"phone"）
    pub credential_name: String,   // 凭据标识（如邮箱/手机号）
    pub data: String,              // 凭据数据（将被加密存储）
}
```

### UserLogin / LoginResult / UserUpdate

```rust
pub struct UserLogin {
    pub credential_name: String,  // 凭据标识（如邮箱/手机号）
    pub data: String,             // 凭据数据（如密码）
}

pub struct LoginResult {
    pub token: String,       // 认证令牌
    pub user_base: UserBase, // 登录用户完整数据
}

pub struct UserUpdate {}     // 更新请求体（暂未定义字段）
```

### TokenClaims

JWT token 的 claims 结构体，由 `parse_token` 返回。

```rust
pub struct TokenClaims {
    pub sub: String,  // 用户 uid
    pub iat: usize,   // 签发时间（UNIX 时间戳）
    pub exp: usize,   // 过期时间（UNIX 时间戳）
}
```

`generate_token` 使用 HS256（HMAC-SHA256）算法签发 token，有效期为 7 天。`parse_token` 验证签名和过期时间，返回 token 中的 claims。

### UserCredential

用户凭据实体，用于认证。

```rust
pub struct UserCredential {
    pub id: i64,                  // 数据库主键
    pub user_uid: String,         // 用户唯一标识
    pub credential_type: String,  // 凭据类型
    pub credential_name: String,  // 凭据标识
    pub encrypted_data: String,   // 加密后的凭据数据
    pub encryption_key: String,   // 加密密钥标识
    pub enabled: bool,            // 是否启用
    pub created_at: DateTime<Utc>, // 创建时间
    pub updated_at: DateTime<Utc>, // 更新时间
}
```

---

## 使用示例

```rust
use std::sync::Arc;
use feel_storage::database::{UserDataBase, CommonUserDataBase};
use feel_storage::repo::SeaOrmUserRepo;  // 假设的具体实现

let repo = SeaOrmUserRepo::new(db_connection);
let user_db: Arc<dyn UserDataBase> = Arc::new(CommonUserDataBase::new(repo));

// 注册用户
let register = UserRegister {
    name: "Alice".into(),
    avatar: "https://example.com/avatar.png".into(),
    credential_type: "password".into(),
    credential_name: "alice@example.com".into(),
    data: "secure_password".into(),
};
let user = user_db.register(&register).await.unwrap();
```

---

## 当前状态

| 方法             | 实现状态 | 说明                                       |
| ---------------- | -------- | ------------------------------------------ |
| register         | ✅ 完整   | async 实现，委托 UserRepo.register         |
| unregister       | ✅ 完整   | 委托 UserRepo.unregister，软删除           |
| login            | ✅ 完整   | async 委托 + 生成 token + 缓存 user_base   |
| logout           | 🚧 占位   | `todo!()`                                  |
| update           | 🚧 占位   | `todo!()`                                  |
| **generate_token** | ✅ 完整 | JWT HS256 签名，7 天有效期                 |
| **parse_token**    | ✅ 完整 | 验证签名 + 过期，返回 TokenClaims          |
| **get_user**       | ✅ 完整 | async 委托 UserRepo::find_by_uid           |
| **get_user_by_id** | ✅ 完整 | async 委托 UserRepo::find_by_id             |

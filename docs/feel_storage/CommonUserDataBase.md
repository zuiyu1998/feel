# CommonUserDataBase — 用户数据访问默认实现

## 概述

`CommonUserDataBase` 是 `UserDataBase` trait 的**默认实现**，位于 `crates/feel_storage/src/database/user.rs`。

它采用委托与自实现混合模式：数据操作（注册、登录、更新等）转发给内部的 `Box<dyn UserRepo>`，而 JWT token 的生成和验证则由自身直接实现（使用 `jwt_secret` 密钥签验）。这使得业务层可以通过 `UserDataBase` trait 统一编程，而底层存储实现可以灵活替换。

## 模块位置

```
crates/feel_storage/src/database/
├── mod.rs          # 重新导出 user 模块
└── user.rs         # UserDataBase trait + CommonUserDataBase 实现
```

## 结构体定义

```rust
pub struct CommonUserDataBase {
    user_repo: Box<dyn UserRepo>,
    user_cache: Box<dyn UserCache>,
    jwt_secret: String,
}
```

| 字段          | 类型                      | 说明                               |
| ------------- | ------------------------- | ---------------------------------- |
| `user_repo`   | `Box<dyn UserRepo>`       | 内部持有的持久化层委托对象         |
| `user_cache`  | `Box<dyn UserCache>`      | 缓存层，用于 Redis 缓存用户数据   |
| `jwt_secret`  | `String`                  | JWT 签名密钥，用于签发和验证 token |

### Trait 约束

| 隐含约束 | 来源                 | 说明                         |
| -------- | -------------------- | ---------------------------- |
| `Send`   | `Box<dyn UserRepo>`  | 可在线程间安全转移           |
| `Sync`   | `UserRepo: Sync`     | 可在线程间安全共享引用       |

> `CommonUserDataBase` 本身未显式标注 trait 约束，但由于其成员 `Box<dyn UserRepo>` 要求 `UserRepo: 'static + Send + Sync`，这些约束会由编译器自动推导。

## 构造方法

```rust
impl CommonUserDataBase {
    pub fn new<T: UserRepo>(user_repo: T, user_cache: Box<dyn UserCache>, jwt_secret: &str) -> Self;
}
```

- `user_repo` — 任何实现了 `UserRepo` trait 的类型，装箱后存储
- `user_cache` — 缓存层实例，用于 Redis 缓存
- `jwt_secret` — JWT 签名密钥字符串

**使用示例：**

```rust
use feel_storage::repo::SeaOrmUserRepo;
use feel_storage::cache::UserCache;
use feel_storage::database::CommonUserDataBase;

let repo = SeaOrmUserRepo::new(db_connection);
let cache = /* 初始化 UserCache 实现 */;
let user_db = CommonUserDataBase::new(repo, cache, "your-jwt-secret");
```

---

## 实现委托关系

`CommonUserDataBase` 实现了 `UserDataBase` trait，数据操作方法委托给 `self.user_repo`，token 管理方法由自身实现。

### register

```rust
async fn register(&self, register: &UserRegister) -> Result<UserBase> {
    self.user_repo.register(register).await
}
```

直接转发至 `UserRepo::register`，async 传递。该方法是完整的，委托 `SeaOrmUserRepo::register` 完成事务内的用户创建 + 凭据存储 + 密码加盐哈希。

### unregister

```rust
async fn unregister(&self, user_id: i64) -> Result<UserBase> {
    self.user_repo.unregister(user_id).await
}
```

直接转发至 `UserRepo::unregister`，async 传递。该方法是完整的，委托 `SeaOrmUserRepo::unregister` 完成软删除（将 `is_delete` 置为 `true`）。

### login

```rust
async fn login(&self, login: &UserLogin) -> Result<LoginResult> {
    // 1. Authenticate via UserRepo — returns user data on success
    let user_base = self.user_repo.login(login).await?;

    // 2. Generate JWT token with user uid as subject, 7-day expiry
    let token = UserDataBase::generate_token(self, &user_base.uid)?;

    // 3. Cache the authenticated user's data for fast subsequent access
    self.user_cache.set_user_base(&user_base).await?;

    Ok(LoginResult { token, user_base })
}
```

`login` 方法包含**三步流程**：
1. **认证** — 委托 `UserRepo::login` 验证凭据，返回 `UserBase`
2. **签 token** — 调用 `generate_token` 为用户 uid 签发 JWT（7 天有效）
3. **缓存** — 将认证用户的 `user_base` 写入 Redis 缓存

### logout / update

```rust
fn logout(&self, _user_id: u32) -> Result<()>        { todo!() }
fn update(&self, _update: &UserUpdate) -> Result<UserBase> { todo!() }
```

### generate_token

```rust
fn generate_token(&self, uid: &str) -> Result<String> {
    let now = Utc::now();
    let claims = TokenClaims {
        sub: uid.to_string(),
        iat: now.timestamp() as usize,
        exp: (now + Duration::hours(24 * 7)).timestamp() as usize,
    };

    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
    )
    .map_err(|e| Error::Authentication(format!("Failed to generate token: {}", e)))
}
```

使用 HS256（HMAC-SHA256）算法签发 JWT，Claims 中包含用户 uid（`sub`）、签发时间（`iat`）和 7 天后的过期时间（`exp`）。

### parse_token

```rust
fn parse_token(&self, token: &str) -> Result<TokenClaims> {
    jsonwebtoken::decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| Error::Authentication(format!("Failed to parse token: {}", e)))
}
```

解析并验证 JWT：验证 HMAC 签名是否匹配、token 是否过期。成功时返回 `TokenClaims`（内含 `sub`、`iat`、`exp`），失败返回 `Error::Authentication`。

`generate_token` 和 `parse_token` 都不委托给 `UserRepo`，由 `CommonUserDataBase` 内部使用 `jwt_secret` 直接实现。

### get_user

```rust
async fn get_user(&self, uid: &str) -> Result<UserBase> {
    // 1. Try cache first (keyed by numeric id — requires uid→id mapping, skip for now)
    // 2. Fallback to repo lookup
    self.user_repo
        .find_by_uid(uid)
        .await?
        .ok_or_else(|| Error::Authentication(format!("User {} not found", uid)))
}
```

委托 `UserRepo::find_by_uid` 按 uid 查找用户，将 `None` 转换为 `Error::Authentication`。暂未集成缓存——因缓存键为数值型 `user_id`，而查询键为字符串 `uid`，后续可通过缓存 uid→id 映射来优化。

### get_user_by_id

```rust
async fn get_user_by_id(&self, user_id: i64) -> Result<Option<UserBase>> {
    self.user_repo.find_by_id(user_id).await
}
```

委托 `UserRepo::find_by_id` 按主键查找用户，未找到时返回 `None`（不转换为错误）。供标签接口"查看标签下的用户"按 `user_id` 组装用户信息。

### 委托总览

| UserDataBase 方法 | 委托目标                  | 实现状态 |
| ----------------- | ------------------------- | -------- |
| `register`        | `UserRepo::register`      | ✅ 完整  |
| `unregister`      | `UserRepo::unregister`    | ✅ 完整  |
| `login`           | `UserRepo::login` + 自签 token + 缓存 | ✅ 完整 |
| `logout`          | —（UserRepo 无此方法）    | 🚧 占位  |
| `update`          | `UserRepo::update`        | 🚧 占位  |
| `generate_token`  | —（JWT HMAC-SHA256 自实现）| ✅ 完整 |
| `parse_token`     | —（JWT 验证自实现）       | ✅ 完整 |
| `get_user`        | `UserRepo::find_by_uid`   | ✅ 完整 |
| `get_user_by_id`  | `UserRepo::find_by_id`    | ✅ 完整 |

---

## 架构关系

```
┌──────────────────────────────────────────────┐
│                  App Layer                    │
│ 持有 Arc<dyn UserDataBase>                   │
│ 调用 UserDataBase 的方法                      │
└─────────────────────┬────────────────────────┘
                      │
                      ▼
┌──────────────────────────────────────────────┐
│          UserDataBase (trait)                 │
│         feel_storage::database::user          │
│          用户数据访问抽象                      │
│     + generate_token / parse_token            │
└─────────────────────┬────────────────────────┘
                      │ impl
                      ▼
┌──────────────────────────────────────────────┐
│     CommonUserDataBase (struct) — ★ 本文件    │
│   持有 Box<dyn UserRepo> + JWT secret        │
│   数据操作 → 委托 UserRepo                    │
│   token管理 → 内部 JWT 签验                   │
└────────────┬────────────────────┬─────────────┘
             │ delegate           │ self impl
             ▼                    ▼
┌─────────────────────┐  ┌─────────────────────┐
│   UserRepo (trait)   │  │  JWT (jsonwebtoken) │
│   持久化层抽象       │  │  HS256 签验         │
└──────────┬──────────┘  └─────────────────────┘
           │ impl
           ▼
┌─────────────────────┐
│  SeaOrmUserRepo      │
│  Sea-ORM 数据库实现  │
└─────────────────────┘
```

### 依赖关系

| 依赖方向       | 说明                                     |
| -------------- | ---------------------------------------- |
| 上层使用       | `feel_api` 通过 `Arc<dyn UserDataBase>` 持有 `CommonUserDataBase` |
| 内部委托       | `CommonUserDataBase` 委托给 `dyn UserRepo` |
| 底层实现       | `UserRepo` 的默认实现为 `SeaOrmUserRepo`  |

---

## 与 UserRepo 的对比

`CommonUserDataBase` 和 `UserRepo` 是两个不同职责层的 trait/struct：

| 维度             | CommonUserDataBase                          | UserRepo                        |
| ---------------- | ------------------------------------------- | ------------------------------- |
| 所属模块         | `database::user`                            | `repo::user`                    |
| 角色             | `UserDataBase` 的默认实现                   | 持久化层抽象接口                |
| 定位             | 数据访问层（供业务层直接调用）              | 存储实现层（供 database 层委托）|
| 是否包含 token   | ✅ 是（内部 JWT 签发与验证）                | ❌ 否（token 不属于持久化职责） |
| 使用方           | `feel_api::AppState` 等业务代码             | `CommonUserDataBase`            |
| 创建方式         | `CommonUserDataBase::new(repo, cache, secret)` | 具体实现如 `SeaOrmUserRepo::new(conn)` |

---

## 使用示例

### 在 AppState 中使用

```rust
use std::sync::Arc;
use feel_storage::database::{UserDataBase, CommonUserDataBase};
use feel_storage::repo::SeaOrmUserRepo;
use feel_storage::cache::UserCache; // 实际缓存实现
use sea_orm::DatabaseConnection;

let conn: DatabaseConnection = /* 获取数据库连接 */;
let repo = SeaOrmUserRepo::new(conn);
let cache = /* 初始化 UserCache 实现 */;
let jwt_secret = "your-256-bit-secret";
let user_db: Arc<dyn UserDataBase> = Arc::new(CommonUserDataBase::new(repo, cache, jwt_secret));

// 注册用户
let register = UserRegister {
    name: "Alice".into(),
    avatar: "https://example.com/avatar.png".into(),
    credential_type: "password".into(),
    credential_name: "alice@example.com".into(),
    data: "secure_password".into(),
};
let user = user_db.register(&register).await.unwrap();

// 登录 — 自动签发 JWT token
let login = UserLogin {
    credential_name: "alice@example.com".into(),
    data: "secure_password".into(),
};
let result = user_db.login(&login).await.unwrap();
println!("Token: {}", result.token);

// 验证 token
let claims = user_db.parse_token(&result.token).unwrap();
println!("User uid: {}", claims.sub);

// 获取用户信息
let user_info = user_db.get_user(&claims.sub).await.unwrap();
println!("User name: {}", user_info.name);
```

### 在切换存储实现时的灵活性

由于 `CommonUserDataBase` 接受任何 `T: UserRepo`，替换底层存储不需要修改业务代码：

```rust
// 假设有一个内存实现 MemoryUserRepo
let repo = MemoryUserRepo::new();
let cache = /* 内存或 Redis 缓存实现 */;
let user_db = CommonUserDataBase::new(repo, cache, "test-secret");
// 业务代码不变，只需替换 repo 和 cache 即可
```

---

## 当前状态

| 方法             | 实现状态 | 说明                                                           |
| ---------------- | -------- | -------------------------------------------------------------- |
| register         | ✅ 完整   | async 委托至 `UserRepo::register`，事务内创建用户 + 凭据      |
| unregister       | ✅ 完整   | async 委托至 `UserRepo::unregister`，软删除用户                |
| login            | ✅ 完整   | async 委托认证 + 签发 JWT token + 缓存 user_base 至 Redis     |
| logout           | 🚧 占位   | `todo!()`，登出属于会话层逻辑，`UserRepo` 无对应方法           |
| update           | 🚧 占位   | `todo!()`，待实现后委托至 `UserRepo::update`                   |
| **generate_token** | ✅ 完整 | JWT HS256 签名，7 天有效期，使用 `jwt_secret` 密钥             |
| **parse_token**    | ✅ 完整 | 验证 HMAC 签名 + 过期时间，返回 `TokenClaims`                  |
| **get_user**       | ✅ 完整 | async 委托至 `UserRepo::find_by_uid`，None→Authentication 错误 |
| **get_user_by_id** | ✅ 完整 | async 委托至 `UserRepo::find_by_id`，None 原样返回           |

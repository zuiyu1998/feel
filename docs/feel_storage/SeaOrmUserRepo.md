# SeaOrmUserRepo — 用户持久化 Sea-ORM 实现

## 概述

`SeaOrmUserRepo` 是 `UserRepo` trait 的 **Sea-ORM 数据库实现**，位于 `crates/feel_storage/src/repo/user.rs`。

它通过 Sea-ORM 操作 `users` 和 `user_credentials` 两张数据库表，完成用户的持久化存储。当前 `register` 和 `unregister` 方法有完整实现，其余方法为 `todo!()` 占位。

## 模块位置

```
crates/feel_storage/src/repo/
├── mod.rs         # UserRepo trait 定义
└── user.rs        # SeaOrmUserRepo 实现 ←
```

## 结构体定义

```rust
pub struct SeaOrmUserRepo {
    /// Sea-ORM 数据库连接
    pub conn: DatabaseConnection,
}
```

### 构造方法

```rust
impl SeaOrmUserRepo {
    pub fn new(conn: DatabaseConnection) -> Self
}
```

接收一个 `sea_orm::DatabaseConnection`，直接持有以备后续数据库操作。

---

## 依赖项

| 依赖                      | 用途                               |
| ------------------------- | ---------------------------------- |
| `sea_orm`                 | ORM 框架（`DatabaseConnection`、`ActiveModelTrait`、`EntityTrait`、`QueryFilter`、`ColumnTrait`、`TransactionTrait`） |
| `chrono::Local`           | 生成当前时间戳                     |
| `typedflake`              | 分布式唯一 ID 生成器（`UserId`）   |
| `argon2`                  | 密码加盐哈希（通过 `get_passwod_hash`） |
| `feel_entity::prelude::*` | 业务模型（`UserBase`、`UserRegister` 等） |
| `feel_sea_orm::user::entities::prelude::*` | ORM 实体（`UserModel`、`UserCredentialsModel`、`UserActiveModel` 等） |

---

## 操作的表

### users 表（ORM 实体：`UserModel` / `UserActiveModel`）

| 字段         | 类型               | 约束/说明                      |
| ------------ | ------------------ | ------------------------------ |
| `id`         | `i64`              | 主键，非自增（由 typedflake 生成） |
| `uid`        | `String`           | 唯一索引，用户唯一标识         |
| `name`       | `String`           | 用户展示名                     |
| `avatar`     | `String`           | 用户头像 URL                   |
| `slogan`     | `String`           | 用户签名/简介（默认空串）      |
| `enabled`    | `bool`             | 是否启用（默认 false）         |
| `created_at` | `DateTime<Utc>`    | 创建时间                       |
| `updated_at` | `DateTime<Utc>`    | 更新时间                       |

### user_credentials 表（ORM 实体：`UserCredentialsModel` / `UserCredentialsActiveModel`）

| 字段              | 类型               | 约束/说明                      |
| ----------------- | ------------------ | ------------------------------ |
| `id`              | `i64`              | 主键，非自增                   |
| `user_uid`        | `String`           | 索引，关联用户                  |
| `credential_type` | `String`           | 索引，凭据类型（如 password）  |
| `credential_name` | `String`           | 凭据标识（如邮箱/手机号）      |
| `encrypted_data`  | `String`           | 加密后的凭据数据（Argon2 hash）|
| `encryption_key`  | `String`           | 加密密钥标识（Argon2 salt）    |
| `enabled`         | `bool`             | 是否启用（默认 false）         |
| `created_at`      | `DateTime<Utc>`    | 创建时间                       |
| `updated_at`      | `DateTime<Utc>`    | 更新时间                       |

### 模型转换

`UserModel` 实现了 `From<Model> for UserBase`，可直接转换为业务模型：

```rust
impl From<Model> for UserBase {
    fn from(value: Model) -> Self {
        UserBase {
            id: value.id,
            uid: value.uid,
            name: value.name,
            avatar: value.avatar,
            slogan: value.slogan,
            enabled: value.enabled,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
```

---

## register 方法详解

`register` 是唯一完整实现的方法，使用**数据库事务**保证用户基础信息和凭据信息的原子性写入。

### 完整流程图

```
┌──────────────────────────────────────────────────────────────┐
│  register(&self, register: &UserRegister)                    │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  1. conn.begin() ───────────────── 开启事务                   │
│                     │                                        │
│  2. 构造 UserActiveModel                                     │
│     ├── name      ← register.name                            │
│     ├── avatar    ← register.avatar                          │
│     ├── uid       ← UserId::generate()  (typedflake)         │
│     ├── created_at ← now().to_utc()                          │
│     └── updated_at ← now().to_utc()                          │
│                     │                                        │
│  3. user_active_model.insert(&begin) ─── 写入 users 表       │
│                     │                                        │
│  4. get_passwod_hash(&register.data)                         │
│     ├── 使用 OsRng 生成随机盐 (SaltString)                    │
│     └── 使用 Argon2 对密码+盐进行哈希                         │
│     → 返回 (salt, password_hash)                             │
│                     │                                        │
│  5. 构造 UserCredentialsActiveModel                          │
│     ├── encryption_key    ← salt                             │
│     ├── encrypted_data    ← password_hash                    │
│     ├── credential_type   ← register.credential_type         │
│     └── credential_name   ← register.credential_name         │
│                     │                                        │
│  6. user_credentials_active_model.insert(&begin) ── 写入表   │
│                     │                                        │
│  7. begin.commit() ──────────────── 提交事务                  │
│                     │                                        │
│  8. Ok(user.into()) ─────────────── UserModel → UserBase     │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### 关键代码

```rust
async fn register(&self, register: &UserRegister) -> Result<UserBase> {
    let begin = self.conn.begin().await?;    // 1. 开启事务

    let now = Local::now();

    // 2. 构造用户模型
    let mut user_active_model: UserActiveModel = Default::default();
    user_active_model.name = Set(register.name.clone());
    user_active_model.avatar = Set(register.avatar.clone());
    user_active_model.uid = Set(UserId::generate().to_string());
    user_active_model.updated_at = Set(now.to_utc());
    user_active_model.created_at = Set(now.to_utc());
    let user: UserModel = user_active_model.insert(&begin).await?;  // 3. 写入 users

    // 4. 密码加盐哈希
    let (salt, password) = get_passwod_hash(&register.data);

    // 5. 构造凭据模型
    let mut user_credentials_active_model: UserCredentialsActiveModel = Default::default();
    user_credentials_active_model.encryption_key = Set(salt);
    user_credentials_active_model.encrypted_data = Set(password);
    user_credentials_active_model.credential_type = Set(register.credential_type.to_string());
    user_credentials_active_model.credential_name = Set(register.credential_name.to_string());

    let _user_credentials: UserCredentialsModel =
        user_credentials_active_model.insert(&begin).await?;  // 6. 写入凭据

    begin.commit().await?;   // 7. 提交事务

    Ok(user.into())          // 8. 返回 UserBase
}
```

### 密码加密：get_passwod_hash

```rust
pub fn get_passwod_hash(password: &str) -> (String, String) {
    let salt = SaltString::generate(&mut OsRng);  // 随机盐
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Hash password failed.")
        .to_string();
    (salt.as_str().to_string(), password_hash)
}
```

- **算法**：Argon2（默认参数）
- **盐**：`OsRng` 密码学安全随机数生成器
- **返回值**：`(salt, password_hash)` — salt 存入 `encryption_key`，hash 存入 `encrypted_data`

---

## unregister 方法详解

`unregister` 使用**软删除**方式，仅将 `users` 表的 `is_delete` 字段置为 `true`，保留数据完整性。

### 完整流程

```
┌──────────────────────────────────────────────────────────────┐
│  unregister(&self, user_id: i64)                             │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  1. UserEntity::find_by_id(user_id)                          │
│     └── one(&self.conn) 查询用户                              │
│                     │                                        │
│  2. 未找到 → 返回 RecordNotFound 错误                        │
│                     │                                        │
│  3. user.into_active_model()                                 │
│     ├── is_delete = Set(true)                                │
│     └── updated_at = Set(now)                                │
│                     │                                        │
│  4. user_active.update(&self.conn) ─── 更新 users 表          │
│                     │                                        │
│  5. Ok(updated_user.into()) ─────── UserModel → UserBase     │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### 关键代码

```rust
async fn unregister(&self, user_id: i64) -> Result<UserBase> {
    let now = Local::now();

    // 1. 查找用户
    let user = UserEntity::find_by_id(user_id)
        .one(&self.conn)
        .await?
        .ok_or_else(|| {
            sea_orm::DbErr::RecordNotFound(format!("User {} not found", user_id))
        })?;

    // 2. 转为 ActiveModel，设置软删除标记
    let mut user_active = user.into_active_model();
    user_active.is_delete = Set(true);
    user_active.updated_at = Set(now.to_utc());

    // 3. 执行更新
    let updated_user: UserModel = user_active.update(&self.conn).await?;

    Ok(updated_user.into())   // 4. 返回 UserBase
}
```

### 注意事项

- **未找到处理**：如果 `user_id` 对应的用户不存在，返回 `sea_orm::DbErr::RecordNotFound`
- **软删除**：仅将 `is_delete` 置为 `true`，不物理删除数据，凭据信息保留在 `user_credentials` 表中
- **类型匹配**：`user_id: i64` 直接匹配数据库主键类型，无需转换
- **无需事务**：单条记录更新，直接使用连接而非事务

---

## 其他方法（待实现）

```rust
fn login(&self, _login: &UserLogin) -> Result<String> {
    todo!()
}

fn update(&self, _update: &UserUpdate) -> Result<UserBase> {
    todo!()
}
```

以上两个方法目前均为 `todo!()` 占位，等待后续实现。

---

## 架构关系

```
┌────────────────────────────────────────────┐
│         UserDataBase (trait)                │
│      feel_storage::database::user           │
│    register / unregister / login / update   │
└──────────────────┬─────────────────────────┘
                   │ 委托调用
                   ▼
┌────────────────────────────────────────────┐
│          UserRepo (trait)                   │
│        feel_storage::repo::user             │
└──────────────────┬─────────────────────────┘
                   │ 实现
                   ▼
┌────────────────────────────────────────────┐
│        SeaOrmUserRepo (struct)              │
│       持有 DatabaseConnection               │
│                                            │
│  ┌──────────────────┐    ┌──────────────┐  │
│  │    users 表       │    │user_credentials│ │
│  │   (UserModel)     │    │ (UserCredModel)│ │
│  └────────┬─────────┘    └──────┬───────┘  │
│           │                     │          │
│           └─────────┬───────────┘          │
│                  事务(Transaction)          │
└────────────────────────────────────────────┘
```

---

## 使用示例

```rust
use sea_orm::Database;
use feel_storage::repo::{UserRepo, SeaOrmUserRepo};

// 建立数据库连接
let conn = Database::connect("sqlite://data.db?mode=rwc").await?;
let repo = SeaOrmUserRepo::new(conn);

// 注册新用户
let register = UserRegister {
    name: "Alice".into(),
    avatar: "https://example.com/avatar.png".into(),
    credential_type: "password".into(),
    credential_name: "alice@example.com".into(),
    data: "my_secure_password".into(),
};

let user = repo.register(&register).await?;
println!("注册成功: {} (uid: {})", user.name, user.uid);
```

---

## 当前状态

| 方法       | 实现状态 | 说明                                            |
| ---------- | -------- | ----------------------------------------------- |
| `register` | ✅ 完整   | 事务内写入 users + user_credentials，Argon2 加密 |
| `unregister` | ✅ 完整   | 软删除：仅将 is_delete 置为 true，保留数据完整性 |
| `login`    | 🚧 占位   | `todo!()`                                       |
| `update`   | 🚧 占位   | `todo!()`                                       |

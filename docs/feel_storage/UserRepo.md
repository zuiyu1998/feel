# UserRepo — 用户持久化层接口

## 概述

`UserRepo` 是 `feel_storage` crate 中定义的**用户数据持久化层 trait**，位于 `crates/feel_storage/src/repo/mod.rs`。

它抽象了对用户数据的数据库操作（注册、登录、注销、更新），使得上层 `UserDataBase` 无需关心具体存储实现。默认的 Sea-ORM 实现为 `SeaOrmUserRepo`，位于 `crates/feel_storage/src/repo/user.rs`。

## 模块位置

```
crates/feel_storage/src/repo/
├── mod.rs         # UserRepo trait 定义 + 模块重新导出
└── user.rs        # SeaOrmUserRepo 实现
```

## Trait 定义

```rust
pub trait UserRepo: 'static + Send + Sync {
    /// 用户系统注册用户
    async fn register(&self, register: &UserRegister) -> Result<UserBase>;

    /// 用户系统注销用户
    async fn unregister(&self, user_id: i64) -> Result<UserBase>;

    /// 用户登录系统
    async fn login(&self, login: &UserLogin) -> Result<LoginResult>;

    /// 用户系统更改个人信息
    fn update(&self, update: &UserUpdate) -> Result<UserBase>;

    /// 根据凭据名称查找用户
    async fn find_by_credential_name(&self, credential_name: &str) -> Result<Option<UserBase>>;
}
```

### Trait 约束

| 约束      | 说明                                   |
| --------- | -------------------------------------- |
| `'static` | 不包含非静态引用，可安全持有           |
| `Send`    | 可跨线程传递所有权                     |
| `Sync`    | 可被多线程共享引用                     |

### 方法说明

| 方法         | 参数                    | 返回                  | 说明                       |
| ------------ | ----------------------- | --------------------- | -------------------------- |
| `register`   | `&UserRegister`         | `Result<UserBase>`    | 注册新用户（**async**）    |
| `unregister` | `user_id: i64`          | `Result<UserBase>`    | 注销指定用户               |
| `login`      | `&UserLogin`            | `Result<LoginResult>` | 用户登录，返回 token+用户  |
| `update`     | `&UserUpdate`           | `Result<UserBase>`    | 更新用户个人信息           |
| `find_by_credential_name` | `credential_name: &str` | `Result<Option<UserBase>>` | 凭据名查用户（**async**） |

> **注意：** `UserRepo` 不包含 `logout` 方法——登出属于会话层逻辑，不在持久化层处理。

### LoginResult

`login` 方法返回 `LoginResult` 结构体，同时携带认证令牌和用户数据：

```rust
pub struct LoginResult {
    pub token: String,       // 认证令牌
    pub user_base: UserBase, // 登录用户的完整数据
}
```

调用方一次性获得令牌和用户信息，无需再次查询数据库。`CommonUserDataBase` 在获取 `LoginResult` 后会将其中的 `user_base` 写入 Redis 缓存。

---

## 默认实现：SeaOrmUserRepo

```rust
pub struct SeaOrmUserRepo {
    /// Sea-ORM 数据库连接
    pub conn: DatabaseConnection,
}
```

`SeaOrmUserRepo` 是 `UserRepo` trait 的基于 [Sea-ORM](https://www.sea-ql.org/SeaORM/) 的实现。

### 构造方法

```rust
impl SeaOrmUserRepo {
    pub fn new(conn: DatabaseConnection) -> Self
}
```

接收一个 Sea-ORM 的 `DatabaseConnection` 实例。

### register 完整实现

`register` 是当前唯一有完整实现的方法，包含以下步骤：

```
┌────────────────────────────────────────────────────┐
│ register(&self, register: &UserRegister)           │
├────────────────────────────────────────────────────┤
│  1. begin() 开启数据库事务                          │
│  2. 生成 UserActiveModel 并填充字段：               │
│     • name / avatar / uid ← UserId::generate()     │
│     • created_at / updated_at ← now()              │
│  3. insert 写入 users 表                            │
│  4. get_passwod_hash() 对密码加盐哈希               │
│  5. 生成 UserCredentialsActiveModel 并填充：        │
│     • encryption_key ← salt                         │
│     • encrypted_data ← password hash                │
│     • credential_type / credential_name             │
│  6. insert 写入 user_credentials 表                 │
│  7. commit() 提交事务                                │
│  8. 返回 UserModel → UserBase 转换结果              │
└────────────────────────────────────────────────────┘
```

**关键细节：**

- **用户 ID 生成**：使用 `typedflake` 库生成分布式唯一 ID (`UserId::generate()`)
- **密码加密**：`get_passwod_hash()` 函数对密码加盐并哈希，返回 `(salt, password_hash)`
- **事务保证**：用户基础信息和凭据在一个数据库事务中完成，确保原子性

---

## 架构关系

```
┌──────────────────────────────────────────────┐
│           UserDataBase (trait)                │
│        feel_storage::database::user           │
│        用户数据访问抽象                        │
└──────────────────┬───────────────────────────┘
                   │ 委托调用
                   ▼
┌──────────────────────────────────────────────┐
│            UserRepo (trait)                   │
│          feel_storage::repo::user             │
│          持久化层抽象                         │
└──────────────────┬───────────────────────────┘
                   │ 实现
                   ▼
┌──────────────────────────────────────────────┐
│         SeaOrmUserRepo (struct)               │
│          Sea-ORM 数据库操作实现               │
├──────────────────────────────────────────────┤
│  users 表           user_credentials 表       │
│  ┌──────────────┐   ┌──────────────────┐     │
│  │ id           │   │ id               │     │
│  │ uid          │   │ user_uid         │     │
│  │ name         │   │ credential_type  │     │
│  │ avatar       │   │ credential_name  │     │
│  │ slogan       │   │ encrypted_data   │     │
│  │ enabled      │   │ encryption_key   │     │
│  │ created_at   │   │ enabled          │     │
│  │ updated_at   │   │ created_at       │     │
│  └──────────────┘   │ updated_at       │     │
│                      └──────────────────┘     │
└──────────────────────────────────────────────┘
```

### 依赖关系

- **数据库 ORM**：Sea-ORM（`sea_orm` crate）
- **实体模型**：`feel_sea_orm::user::entities`（定义 `users` 和 `user_credentials` 表的 ORM 实体）
- **上层数据模型**：`feel_entity::user::models`（`UserBase`、`UserRegister` 等）
- **工具函数**：`crate::utils::get_passwod_hash`（密码加盐哈希）
- **ID 生成**：`typedflake`（分布式唯一 ID 生成器）

---

## 错误处理

`UserRepo` 中的方法均返回 `crate::Result<T>`，其定义为：

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
}
```

`register` 方法中可能返回的错误：
- `sea_orm::DbErr` — 数据库操作失败（插入 users 或 user_credentials 表）
- 通过 `?` 操作符自动向上传播

---

## 使用示例

```rust
use sea_orm::DatabaseConnection;
use feel_storage::repo::{UserRepo, SeaOrmUserRepo};

let conn: DatabaseConnection = /* 获取数据库连接 */;
let repo = SeaOrmUserRepo::new(conn);

// 注册用户
let register = UserRegister {
    name: "Bob".into(),
    avatar: "https://example.com/bob.png".into(),
    credential_type: "password".into(),
    credential_name: "bob@example.com".into(),
    data: "secure_password".into(),
};
let user = repo.register(&register).await.unwrap();
```

---

## 与 UserDataBase 的对比

| 维度       | UserDataBase                    | UserRepo                        |
| ---------- | ------------------------------- | ------------------------------- |
| 所属模块   | `database::user`                | `repo::user`                    |
| 职责       | 数据访问接口（业务层使用）      | 持久化实现接口（存储层）        |
| 方法       | register / unregister / login / logout / update | register / unregister / login / update |
| 无 logout  | 有 logout                        | 无 logout（会话层职责）         |
| 实现方式   | 委托 UserRepo                   | 直接操作 Sea-ORM                |

---

## 当前状态

| 方法       | 实现状态 | 说明                                           |
| ---------- | -------- | ---------------------------------------------- |
| register   | ✅ 完整   | 事务内创建用户 + 凭据，密码加盐哈希存储        |
| `unregister` | ✅ 完整   | 软删除：仅将 is_delete 置为 true，保留数据完整性 |
| login      | 🚧 占位   | `todo!()`                                      |
| update     | 🚧 占位   | `todo!()`                                      |
| `find_by_credential_name` | ✅ 完整   | 凭据名 → user_uid → 用户数据查找               |

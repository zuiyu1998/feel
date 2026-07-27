# CommonUserDataBase — 用户数据访问默认实现

## 概述

`CommonUserDataBase` 是 `UserDataBase` trait 的**默认实现**，位于 `crates/feel_storage/src/database/user.rs`。

它采用委托模式（Delegation Pattern），将所有的数据操作转发给内部的 `Box<dyn UserRepo>`，自身不包含任何业务逻辑或持久化细节。这使得业务层可以通过 `UserDataBase` trait 统一编程，而底层存储实现可以灵活替换。

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
    user_cache: Option<Box<dyn UserCache>>,
}
```

| 字段          | 类型                      | 说明                           |
| ------------- | ------------------------- | ------------------------------ |
| `user_repo`   | `Box<dyn UserRepo>`       | 内部持有的持久化层委托对象     |
| `user_cache`  | `Option<Box<dyn UserCache>>` | 可选的缓存层，用于 Redis 缓存 |

### Trait 约束

| 隐含约束 | 来源                 | 说明                         |
| -------- | -------------------- | ---------------------------- |
| `Send`   | `Box<dyn UserRepo>`  | 可在线程间安全转移           |
| `Sync`   | `UserRepo: Sync`     | 可在线程间安全共享引用       |

> `CommonUserDataBase` 本身未显式标注 trait 约束，但由于其成员 `Box<dyn UserRepo>` 要求 `UserRepo: 'static + Send + Sync`，这些约束会由编译器自动推导。

## 构造方法

```rust
impl CommonUserDataBase {
    pub fn new<T: UserRepo>(user_repo: T) -> Self;
    pub fn with_cache<T: UserRepo>(user_repo: T, user_cache: Box<dyn UserCache>) -> Self;
}
```

- `new` — 接受任何实现了 `UserRepo` trait 的类型，将其装箱后存储，不含缓存
- `with_cache` — 接受 `UserRepo` 和 `UserCache`，在委托的基础上支持 Redis 缓存

**使用示例：**

```rust
use feel_storage::repo::SeaOrmUserRepo;
use feel_storage::database::CommonUserDataBase;

let repo = SeaOrmUserRepo::new(db_connection);
let user_db = CommonUserDataBase::new(repo);
```

---

## 实现委托关系

`CommonUserDataBase` 实现了 `UserDataBase` trait，每个方法直接委托给 `self.user_repo` 的对应方法。

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
    let login_result = self.user_repo.login(login).await?;

    // Cache the authenticated user's data for fast subsequent access
    if let Some(cache) = &self.user_cache {
        cache.set_user_base(&login_result.user_base).await?;
    }

    Ok(login_result)
}
```

`login` 方法包含**两步流程**：
1. **认证** — 委托 `UserRepo::login` 验证凭据，返回 `LoginResult`（含 token 和用户数据）
2. **缓存** — 若配置了 `UserCache`，将认证用户的 `user_base` 写入 Redis 缓存

### logout / update

```rust
fn logout(&self, _user_id: u32) -> Result<()>        { todo!() }
fn update(&self, _update: &UserUpdate) -> Result<UserBase> { todo!() }

### 委托总览

| UserDataBase 方法 | 委托目标                  | 实现状态 |
| ----------------- | ------------------------- | -------- |
| `register`        | `UserRepo::register`      | ✅ 完整  |
| `unregister`      | `UserRepo::unregister`    | ✅ 完整  |
| `login`           | `UserRepo::login`         | ✅ 完整，含缓存 |
| `logout`          | —（UserRepo 无此方法）    | 🚧 占位  |
| `update`          | `UserRepo::update`        | 🚧 占位  |

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
└─────────────────────┬────────────────────────┘
                      │ impl
                      ▼
┌──────────────────────────────────────────────┐
│     CommonUserDataBase (struct) — ★ 本文件    │
│          持有 Box<dyn UserRepo>              │
│        所有方法委托给 UserRepo                │
└─────────────────────┬────────────────────────┘
                      │ delegate
                      ▼
┌──────────────────────────────────────────────┐
│           UserRepo (trait)                    │
│         feel_storage::repo::user              │
│          持久化层抽象                         │
└─────────────────────┬────────────────────────┘
                      │ impl
                      ▼
┌──────────────────────────────────────────────┐
│         SeaOrmUserRepo (struct)               │
│         Sea-ORM 数据库操作实现                │
└──────────────────────────────────────────────┘
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

| 维度             | CommonUserDataBase                  | UserRepo                        |
| ---------------- | ----------------------------------- | ------------------------------- |
| 所属模块         | `database::user`                    | `repo::user`                    |
| 角色             | `UserDataBase` 的默认实现           | 持久化层抽象接口                |
| 定位             | 数据访问层（供业务层直接调用）      | 存储实现层（供 database 层委托）|
| 是否包含 `logout` | ✅ 是（自己持有，不委托）           | ❌ 否（会话层不属于持久化职责） |
| 使用方           | `feel_api::AppState` 等业务代码     | `CommonUserDataBase`            |
| 创建方式         | `CommonUserDataBase::new(repo)`     | 具体实现如 `SeaOrmUserRepo::new(conn)` |

---

## 使用示例

### 在 AppState 中使用

```rust
use std::sync::Arc;
use feel_storage::database::{UserDataBase, CommonUserDataBase};
use feel_storage::repo::SeaOrmUserRepo;
use sea_orm::DatabaseConnection;

let conn: DatabaseConnection = /* 获取数据库连接 */;
let repo = SeaOrmUserRepo::new(conn);
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

### 在切换存储实现时的灵活性

由于 `CommonUserDataBase` 接受任何 `T: UserRepo`，替换底层存储不需要修改业务代码：

```rust
// 假设有一个内存实现 MemoryUserRepo
let repo = MemoryUserRepo::new();
let user_db = CommonUserDataBase::new(repo);
// 业务代码不变，只需替换 repo 即可
```

---

## 当前状态

| 方法       | 实现状态 | 说明                                                       |
| ---------- | -------- | ---------------------------------------------------------- |
| register   | ✅ 完整   | async 委托至 `UserRepo::register`，事务内创建用户 + 凭据  |
| unregister | ✅ 完整   | async 委托至 `UserRepo::unregister`，软删除用户            |
| login      | ✅ 完整   | async 委托认证 + 缓存 user_base 至 Redis               |
| logout     | 🚧 占位   | `todo!()`，登出属于会话层逻辑，`UserRepo` 无对应方法       |
| update     | 🚧 占位   | `todo!()`，待实现后委托至 `UserRepo::update`               |

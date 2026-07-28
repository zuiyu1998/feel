# UserCache — 用户缓存接口

## 概述

`UserCache` 是 `feel_storage` crate 中定义的**用户缓存层 trait**，位于 `crates/feel_storage/src/cache/user.rs`。

它抽象了对用户基础信息的 Redis 缓存操作（设置与获取），使业务层可以快速读取用户数据而无需每次都查询数据库。默认实现为 `CommonUserCache`，基于 Redis Hash 结构存储。

## 模块位置

```
crates/feel_storage/src/cache/
├── mod.rs         # 模块声明
└── user.rs        # UserCache trait + CommonUserCache 实现
```

## Trait 定义

```rust
#[async_trait]
pub trait UserCache: 'static + Send + Sync {
    /// 设置用户基础信息到缓存
    async fn set_user_base(&self, user_base: &UserBase) -> Result<()>;

    /// 从缓存获取用户基础信息
    async fn get_user_base(&self, user_id: &str) -> Result<Option<UserBase>>;
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

| 方法             | 参数                    | 返回                       | 说明                       |
| ---------------- | ----------------------- | -------------------------- | -------------------------- |
| `set_user_base`  | `&UserBase`             | `Result<()>`               | 将用户基础信息写入缓存     |
| `get_user_base`  | `user_id: &str`          | `Result<Option<UserBase>>` | 从缓存读取用户基础信息     |

---

## 默认实现：CommonUserCache

```rust
pub struct CommonUserCache {
    client: Client,  // redis 客户端
}
```

`CommonUserCache` 是 `UserCache` trait 的基于 Redis 的默认实现。

### 构造方法

```rust
impl CommonUserCache {
    pub fn new(client: Client) -> Self
}
```

接收一个 `redis::Client` 实例。

### 内部辅助方法

```rust
impl CommonUserCache {
    async fn get_connection(&self) -> RedisResult<redis::aio::MultiplexedConnection>
}
```

获取一个 Redis 异步多路复用连接（基于 Tokio）。

### 实现细节

#### set_user_base 流程

```
┌───────────────────────────────────────────────────┐
│ set_user_base(&self, user_base: &UserBase)        │
├───────────────────────────────────────────────────┤
│  1. 获取 Redis 异步连接                            │
│  2. serde_json::to_string(user_base) 序列化为 JSON │
│  3. HSET users <uid> <json> 写入 Redis Hash    │
│  4. 返回 Ok(())                                    │
└───────────────────────────────────────────────────┘
```

#### get_user_base 流程

```
┌───────────────────────────────────────────────────┐
│ get_user_base(&self, user_id: &str)                │
├───────────────────────────────────────────────────┤
│  1. 获取 Redis 异步连接                            │
│  2. HGET users <user_id> 读取 Redis Hash           │
│  3. 匹配结果：                                     │
│     • Some(json) → serde_json::from_str 反序列化   │
│     • None → 返回 Ok(None)                         │
│  4. 返回 Ok(Some(UserBase)) 或 Ok(None)            │
└───────────────────────────────────────────────────┘
```

### Redis 存储结构

```
Redis Key: "users" (Hash 类型)

┌──────────────┬──────────────────────────────────────┐
│ Field (uid)        │ Value (JSON 序列化的 UserBase)    │
├──────────────┼──────────────────────────────────────┤
│ "usr_abc123" │ {"id":1,"uid":"usr_abc123","name":"Alice",  │
│              │  "avatar":"...","slogan":"...", ...}  │
├──────────────┼──────────────────────────────────────┤
│ "usr_def456" │ {"id":2,"uid":"usr_def456","name":"Bob",    │
│              │  "avatar":"...","slogan":"...", ...}  │
└──────────────┴──────────────────────────────────────┘
```

---

## 架构关系

```
┌──────────────────────────────────────────────┐
│                App Layer                       │
│     持有 Arc<dyn UserCache>                    │
│     先查缓存 → 未命中再查 DB → 写入缓存        │
└──────────────────┬───────────────────────────┘
                   │ 调用 trait 方法
                   ▼
┌──────────────────────────────────────────────┐
│          UserCache (trait)                    │
│       feel_storage::cache::user               │
│        用户缓存抽象                            │
└──────────────────┬───────────────────────────┘
                   │ 默认实现
                   ▼
┌──────────────────────────────────────────────┐
│        CommonUserCache (struct)               │
│            持有 redis::Client                  │
│        基于 Redis Hash 存储                    │
└──────────────────┬───────────────────────────┘
                   │ HSET / HGET
                   ▼
┌──────────────────────────────────────────────┐
│              Redis                             │
│     Key: "users" (Hash)                       │
│     Field: uid → JSON(UserBase)           │
└──────────────────────────────────────────────┘
```

### 依赖关系

| 依赖            | 用途                     |
| --------------- | ------------------------ |
| `redis` crate   | Redis 客户端 + 异步连接  |
| `serde_json`    | UserBase 序列化/反序列化 |
| `feel_entity`   | UserBase 数据模型        |

---

## 错误处理

`UserCache` 方法返回 `crate::Result<T>`，其定义为：

```rust
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),     // Redis 连接/命令错误

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),      // JSON 序列化/反序列化错误

    #[error("Database error: {0}")]
    Db(#[from] sea_orm::DbErr),           // 数据库错误（缓存层不使用）
}
```

**缓存层可能出现的错误：**

| 错误类型              | 场景                               |
| --------------------- | ---------------------------------- |
| `Redis`               | Redis 连接失败、命令执行失败       |
| `Json`                | UserBase 序列化或反序列化失败      |

---

## 使用示例

```rust
use std::sync::Arc;
use redis::Client;
use feel_storage::cache::{UserCache, CommonUserCache};

let redis_client = Client::open("redis://127.0.0.1:6379/")?;
let cache: Arc<dyn UserCache> = Arc::new(CommonUserCache::new(redis_client));

// 写入缓存
let user = UserBase { /* ... */ };
cache.set_user_base(&user).await.unwrap();

// 读取缓存
if let Some(cached_user) = cache.get_user_base("usr_abc123").await.unwrap() {
    println!("从缓存命中用户: {}", cached_user.name);
} else {
    println!("缓存未命中，需要从数据库查询");
}
```

### 典型缓存模式（Cache-Aside）

```rust
async fn get_user(user_id: &str, cache: &dyn UserCache, repo: &dyn UserRepo) -> UserBase {
    // 1. 先查缓存
    if let Some(user) = cache.get_user_base(user_id).await.unwrap() {
        return user;
    }
    // 2. 缓存未命中，查数据库
    let user = /* repo 查询 */;
    // 3. 写入缓存
    cache.set_user_base(&user).await.unwrap();
    user
}
```

---

## 当前状态

| 方法             | 实现状态 | 说明                                    |
| ---------------- | -------- | --------------------------------------- |
| `set_user_base`  | ✅ 完整   | JSON 序列化后通过 HSET 写入 Redis Hash  |
| `get_user_base`  | ✅ 完整   | 通过 HGET 读取并反序列化为 UserBase     |

两个方法均已完整实现，可直接使用。

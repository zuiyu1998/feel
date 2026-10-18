# feel_storage::Error — 统一错误类型

## 概述

`feel_storage` 定义平台统一的错误类型 `Error`（`crates/feel_storage/src/error.rs`），贯穿 Repo、DataBase、缓存各层；`feel_api` 层通过 `from_storage_error` 将其映射为统一的 HTTP JSON 错误响应（`ApiResponse`）。

## 类型定义

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Database error: {0}")]
    Db(#[from] sea_orm::DbErr),
    #[error("Business error: {0}")]
    Business(String),
    #[error("Authentication error: {0}")]
    Authentication(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
```

## 变体说明

| 变体 | 来源 | 典型场景 |
|------|------|----------|
| `Redis` | `redis::RedisError` 自动 `#[from]` | `UserCache` 缓存读写（get/set/del）失败 |
| `Json` | `serde_json::Error` 自动 `#[from]` | 缓存值或请求体的 JSON 序列化/反序列化失败 |
| `Db` | `sea_orm::DbErr` 自动 `#[from]` | 任何 Repo 的数据库操作；`DbErr::RecordNotFound` 单独映射为 404 |
| `Business` | 手工构造 `Error::Business(String)` | 业务规则校验：标签数量超限（20）、无权限操作他人关联、标签不存在等 |
| `Authentication` | 手工构造 `Error::Authentication(String)` | 登录凭据校验失败、JWT token 解析/过期 |

## API 错误码映射（feel_api）

`cmd/feel_api/src/model/response.rs` 中的 `from_storage_error` 按变体自动映射状态码：

| `feel_storage::Error` | 状态码 | 常量（值） |
|-----------------------|--------|------------|
| `Redis(_)` | 10001 | `CODE_REDIS_ERROR` |
| `Json(_)` | 10002 | `CODE_JSON_ERROR` |
| `Db(DbErr::RecordNotFound)` | 10004 | `CODE_NOT_FOUND` |
| `Db(_)`（其他） | 10003 | `CODE_DB_ERROR` |
| `Authentication(_)` | 10005 | `CODE_AUTH_ERROR` |
| `Business(_)` | 10006 | `CODE_BUSINESS_ERROR` |

`message` 字段直接取错误的 display 文本（`e.to_string()`）；`data` 置为 `null`：

```json
{
    "code": 10006,
    "message": "Business error: User 1 cannot manage labels of user 2",
    "data": null
}
```

（状态码常量完整定义见 [model/response.md](../feel_api/model/response.md)）

## 使用示例

```rust
// handler 中从 StorageError 转换为 HTTP 错误响应
let user = match state.user_database.get_user(&body.0.user_uid).await {
    Ok(u) => u,
    Err(e) => return from_storage_error(e),
};

// Repo/DataBase 内部：数据库错误由 `?` 自动转换为 Error::Db
let model = UserEntity::find().one(&self.conn).await?;  // DbErr → Error::Db

// 业务校验错误手工构造
return Err(Error::Business(format!(
    "User {} cannot manage labels of user {}",
    me.id, body.0.user_id
)));
```

## 错误产生位置

| 层 | 常见错误 | 说明 |
|----|----------|------|
| Repo（`SeaOrmUserRepo` / `SeaOrmLabelRepo`） | `Db` | SQL 执行失败、`RecordNotFound` |
| DataBase（`CommonUserDataBase` / `CommonLabelDataBase`） | `Db`、`Business` | 组合 Repo + 缓存；业务校验（标签数量上限） |
| Cache（`UserCache`） | `Redis`、`Json` | 缓存读写与序列化 |
| Handler（`feel_api`） | `Business`、`Authentication` | 权限校验、登录验证失败 |

## 当前状态

| 项 | 状态 | 说明 |
|----|------|------|
| 错误枚举 | ✅ | 5 个变体，含 `#[from]` 自动转换 |
| API 映射 | ✅ | `from_storage_error` 统一映射状态码 |
| 各层使用 | ✅ | Repo / DataBase / Cache / Handler 全覆盖 |

## 相关文档

- [model/response.md](../feel_api/model/response.md) — `ApiResponse<T>` 与状态码常量
- 各层文档的「错误处理」小节：[UserRepo](UserRepo.md)、[UserCache](UserCache.md)、[LabelRepo](LabelRepo.md)

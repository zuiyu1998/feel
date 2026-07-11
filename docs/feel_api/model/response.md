# ApiResponse — 通用 JSON 响应封装

## 概述

`ApiResponse<T>` 是一个泛型响应包装结构体，提供统一的 API 响应格式。配合 `ok()` / `err()` / `from_storage_error()` 工厂函数，所有接口返回一致的 JSON 结构。

定义在 `cmd/feel_api/src/model/response.rs`。

## 预定义状态码

| 常量 | 值 | 说明 |
|------|-----|------|
| [`CODE_OK`] | `0` | 业务处理成功 |
| [`CODE_SERVER_ERROR`] | `10000` | 服务器内部错误 |
| [`CODE_REDIS_ERROR`] | `10001` | Redis 操作异常 |
| [`CODE_JSON_ERROR`] | `10002` | JSON 序列化/反序列化错误 |
| [`CODE_DB_ERROR`] | `10003` | 数据库操作异常 |
| [`CODE_NOT_FOUND`] | `10004` | 请求的资源不存在 |

```rust
pub const CODE_OK: i32 = 0;
pub const CODE_SERVER_ERROR: i32 = 10000;
pub const CODE_REDIS_ERROR: i32 = 10001;
pub const CODE_JSON_ERROR: i32 = 10002;
pub const CODE_DB_ERROR: i32 = 10003;
pub const CODE_NOT_FOUND: i32 = 10004;
```

业务模块可在此基础上定义自己的错误码（如 `1001`、`1002` 等），建议 `CODE_SERVER_ERROR` 保留给未知的服务器内部异常。

## 类型定义

```rust
pub struct ApiResponse<T: Serialize> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}
```

| 字段      | 类型         | 说明                                    |
|-----------|--------------|-----------------------------------------|
| `code`    | `i32`        | 业务状态码（`CODE_OK` = 0 表示成功）   |
| `message` | `String`     | 提示消息                                |
| `data`    | `Option<T>`  | 响应数据。成功时为 `Some`，错误时为 `None` 即 JSON `null` |

## 工厂函数

### `ok` — 成功响应

```rust
pub fn ok<T: Serialize>(data: T) -> Json<ApiResponse<T>>
```

返回 `code = CODE_OK`、`message = "ok"` 的成功响应，`data` 自动包装为 `Some(data)`。

**示例：**

```rust
#[handler]
async fn register(body: Json<RegisterRequest>) -> Json<ApiResponse<RegisterResponse>> {
    // ...
    ok(RegisterResponse { /* ... */ })
}
```

**输出 JSON：**

```json
{
    "code": 0,
    "message": "ok",
    "data": { /* RegisterResponse 字段 */ }
}
```

### `err` — 错误响应

```rust
pub fn err<T: Serialize>(code: i32, message: impl Into<String>) -> Json<ApiResponse<T>>
```

返回自定义状态码和消息的错误响应，`data` 置为 `None`（JSON `null`）。
泛型 `T` 由调用处的返回类型自动推断。

**示例：**

```rust
#[handler]
async fn register(body: Json<RegisterRequest>) -> Json<ApiResponse<RegisterResponse>> {
    if body.name.is_empty() {
        return err(1001, "用户名不能为空");
    }
    // ...
    ok(RegisterResponse { /* ... */ })
}
```

**输出 JSON：**

```json
{
    "code": 1001,
    "message": "用户名不能为空",
    "data": null
}
```

### `from_storage_error` — 转换存储层错误

```rust
pub fn from_storage_error<T: Serialize>(e: StorageError) -> Json<ApiResponse<T>>
```

将 `feel_storage::error::Error` 转换为统一的 API 错误响应，无需手动匹配错误类型。

#### 错误映射规则

| `StorageError` 变体 | 状态码 | 适用场景 |
|---------------------|--------|----------|
| `Redis(_)` | `CODE_REDIS_ERROR` (10001) | Redis 操作失败 |
| `Json(_)` | `CODE_JSON_ERROR` (10002) | JSON 序列化/反序列化失败 |
| `Db(DbErr::RecordNotFound(_))` | `CODE_NOT_FOUND` (10004) | 请求的资源不存在 |
| `Db(_)` | `CODE_DB_ERROR` (10003) | 其他数据库操作异常 |

消息内容直接使用错误的 `Display` 文本，便于调试。

#### 使用方式

**方式一：与 `map_err` 配合（推荐）**

在 handler 中链式调用，将 `Result<_, StorageError>` 转换为 `Json<ApiResponse<T>>`：

```rust
use feel_api::model::response::from_storage_error;

#[handler]
async fn get_user(
    state: Data<&AppState>,
    Path(user_id): Path<i64>,
) -> Json<ApiResponse<RegisterResponse>> {
    let user_base = state
        .user_database
        .unregister(user_id)
        .await
        .map_err(from_storage_error)?;

    ok(RegisterResponse {
        id: user_base.id,
        uid: user_base.uid,
        // ...
    })
}
```

**方式二：显式转换**

```rust
use feel_api::model::response::from_storage_error;

let result = state.user_database.unregister(id).await;
match result {
    Ok(user) => ok(user.into()),
    Err(e) => from_storage_error(e),
}
```

## 使用方式

```rust
use feel_api::model::response::{ok, err, from_storage_error, ApiResponse, CODE_OK, CODE_SERVER_ERROR};
use poem::web::Json;

// 成功（带数据）
fn list_users() -> Json<ApiResponse<Vec<UserBase>>> {
    let users = vec![/* ... */];
    ok(users)
}

// 错误（无数据）
fn delete_user(id: i64) -> Json<ApiResponse<()>> {
    if id <= 0 {
        return err(400, "无效的用户 ID");
    }
    // ...
    ok(())
}

// 存储层错误转换
async fn find_user(id: i64) -> Json<ApiResponse<UserBase>> {
    let user = repo.find(id).await.map_err(from_storage_error)?;
    ok(user)
}
```

## 与 Poem 的集成

`ApiResponse<T>` 通过 `Json<ApiResponse<T>>` 自动序列化，可直接作为 handler 返回值使用：

```rust
#[handler]
fn get_user() -> Json<ApiResponse<UserBase>> {
    // ...
}
```

## 设计说明

- 泛型参数 `T` 绑定 `Serialize`，确保所有 data 均可序列化为 JSON
- `data: Option<T>` 使错误响应时无需构造无意义的数据占位，JSON 输出 `null`
- `ok()` 与 `err()` 覆盖成功/失败两种场景，保持响应格式统一
- `from_storage_error()` 桥接 `feel_storage` 的错误类型，handler 中可通过 `map_err` 链式转换
- 预定义状态码 `CODE_OK` / `CODE_SERVER_ERROR` 等，业务模块可在此基础上扩展（如 `1001`、`1002` 等业务码）
- 前端可按 `code` 字段判断业务状态，而非依赖 HTTP 状态码

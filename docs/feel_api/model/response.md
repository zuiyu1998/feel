# ApiResponse — 通用 JSON 响应封装

## 概述

`ApiResponse<T>` 是一个泛型响应包装结构体，提供统一的 API 响应格式。配合 `ok()` / `err()` 工厂函数，所有接口返回一致的 JSON 结构。

定义在 `cmd/feel_api/src/model/response.rs`。

## 类型定义

```rust
pub struct ApiResponse<T: Serialize> {
    pub code: i32,
    pub message: String,
    pub data: T,
}
```

| 字段      | 类型     | 说明                          |
|-----------|----------|-------------------------------|
| `code`    | `i32`    | 业务状态码（0 表示成功）      |
| `message` | `String` | 提示消息                      |
| `data`    | `T`      | 响应数据（泛型，实现了 Serialize） |

## 工厂函数

### `ok` — 成功响应

```rust
pub fn ok<T: Serialize>(data: T) -> Json<ApiResponse<T>>
```

返回 `code = 0`、`message = "ok"` 的成功响应。

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
pub fn err<T: Serialize>(code: i32, message: impl Into<String>, data: T) -> Json<ApiResponse<T>>
```

返回自定义状态码和消息的错误响应。

**示例：**

```rust
#[handler]
async fn register(body: Json<RegisterRequest>) -> Json<ApiResponse<()>> {
    if body.name.is_empty() {
        return err(1001, "用户名不能为空", ());
    }
    // ...
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

## 使用方式

```rust
use feel_api::model::response::{ok, err, ApiResponse};
use poem::web::Json;

// 成功
fn list_users() -> Json<ApiResponse<Vec<UserBase>>> {
    let users = vec![/* ... */];
    ok(users)
}

// 错误
fn delete_user(id: i64) -> Json<ApiResponse<()>> {
    if id <= 0 {
        return err(400, "无效的用户 ID", ());
    }
    // ...
    ok(())
}
```

## 与 Poem 的集成

`ApiResponse<T>` 实现了 `poem::IntoResponse`（或通过 `Json<ApiResponse<T>>` 自动序列化），可直接作为 handler 返回值使用：

```rust
#[handler]
fn get_user() -> Json<ApiResponse<UserBase>> {
    // ...
}
```

## 设计说明

- 泛型参数 `T` 绑定 `Serialize`，确保所有 data 均可序列化为 JSON
- `ok()` 与 `err()` 覆盖成功/失败两种场景，保持响应格式统一
- 前端可按 `code` 字段判断业务状态，而非依赖 HTTP 状态码

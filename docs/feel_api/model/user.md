# User DTO — 用户 API 请求/响应模型

## 概述

用户相关的 API 请求/响应 DTO 定义在 `cmd/feel_api/src/model/user.rs` 中，通过 `serde` 实现 JSON 序列化与反序列化。

## 类型定义

### RegisterRequest

注册用户请求体。

```rust
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub avatar: String,
    pub credential_type: String,
    pub credential_name: String,
    pub data: String,
}
```

| 字段              | 类型     | 说明                        |
|-------------------|----------|-----------------------------|
| `name`            | `String` | 用户昵称                    |
| `avatar`          | `String` | 头像 URL                    |
| `credential_type` | `String` | 凭证类型（如 `"password"`） |
| `credential_name` | `String` | 凭证名称（如 `"default"`）  |
| `data`            | `String` | 明文密码                    |

**与领域模型的对应关系：**

与 `feel_entity::user::models::UserRegister` 字段一致，当前为直接映射。后续可在此添加校验注解（如 `#[validate]`）或字段调整，而不影响领域层。

**JSON 示例：**

```json
{
    "name": "张三",
    "avatar": "https://example.com/avatar.png",
    "credential_type": "password",
    "credential_name": "default",
    "data": "my_secret_password"
}
```

### RegisterResponse

注册用户响应体。

```rust
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub id: i64,
    pub uid: String,
    pub name: String,
    pub avatar: String,
    pub slogan: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}
```

| 字段         | 类型     | 说明                        |
|--------------|----------|-----------------------------|
| `id`         | `i64`    | 自增主键                    |
| `uid`        | `String` | 全局唯一标识符（业务键）    |
| `name`       | `String` | 用户昵称                    |
| `avatar`     | `String` | 头像 URL                    |
| `slogan`     | `String` | 用户签名                    |
| `enabled`    | `bool`   | 是否启用                    |
| `created_at` | `String` | 创建时间（ISO 8601 字符串） |
| `updated_at` | `String` | 更新时间（ISO 8601 字符串） |

**与领域模型的差异：**

相比 `feel_entity::user::models::UserBase`（使用 `DateTime<Utc>`），`RegisterResponse` 的时间字段使用 `String`，将时区格式化的职责交给序列化层，避免 HTTP 响应中暴露 chrono 类型细节。

**JSON 示例：**

```json
{
    "id": 1,
    "uid": "uid_abc123",
    "name": "张三",
    "avatar": "https://example.com/avatar.png",
    "slogan": "",
    "enabled": true,
    "created_at": "2026-07-11T03:00:00Z",
    "updated_at": "2026-07-11T03:00:00Z"
}
```

## 后续扩展

当前 `user.rs` 末尾留有 TODO 标记，后续可按需添加：

- `LoginRequest` / `LoginResponse`
- `UnregisterRequest` / `UnregisterResponse`
- `LogoutRequest` / `LogoutResponse`
- 通用的分页/错误响应包装类型

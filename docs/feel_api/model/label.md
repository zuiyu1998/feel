# Label DTO — 标签 API 请求/响应模型

## 概述

标签相关的 API 请求/响应 DTO 定义在 `cmd/feel_api/src/model/label.rs` 中，通过 `serde` 实现 JSON 序列化与反序列化。各接口的完整调用链与示例见 [feel_api/label.md](../label.md)。

与 `feel_entity::label` 中的领域模型不同，此模块中的类型专注于 HTTP 序列化与字段裁剪；响应 DTO 提供 `From<LabelBase>` / `From<UserLabel>` 转换。

## 请求 DTO

### ListUserLabelsRequest

查看用户标签请求体（`GET /api/v1/labels/list`）。

```rust
#[derive(Debug, Deserialize)]
pub struct ListUserLabelsRequest {
    pub user_id: i64,       // 查看哪个用户的标签
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `user_id` | `i64` | 目标用户主键（`UserBase.id`） |

### ListLabelUsersRequest

查看标签下的用户请求体（`GET /api/v1/labels/users`）。

```rust
#[derive(Debug, Deserialize)]
pub struct ListLabelUsersRequest {
    pub label_id: i64,      // 查看哪个标签下的用户
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `label_id` | `i64` | 标签主键（`LabelBase.id`） |

### AddLabelRequest

添加标签关联请求体（`POST /api/v1/labels/add`）。

```rust
#[derive(Debug, Deserialize)]
pub struct AddLabelRequest {
    pub user_uid: String,   // 归属用户（业务键，关联 UserBase.uid）
    pub label_id: i64,      // 关联的标签本体
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `user_uid` | `String` | 用户业务键（非主键），handler 内部解析为 `UserBase.id` |
| `label_id` | `i64` | 标签主键 |

### CreateLabelRequest

创建标签本体请求体（`POST /api/v1/labels/create`，需登录）。

```rust
#[derive(Debug, Deserialize)]
pub struct CreateLabelRequest {
    pub name: String,
    pub description: String,
    pub remark: String,
    pub influence: i64,
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | `String` | 标签名（全局唯一，由存储层保证） |
| `description` | `String` | 公共描述 |
| `remark` | `String` | 标签备注（所有用户共享） |
| `influence` | `i64` | 影响力，创建时确立，之后不变 |

### UpdateLabelRequest

修改标签备注请求体（`PUT /api/v1/labels/update`，需登录）。

```rust
#[derive(Debug, Deserialize)]
pub struct UpdateLabelRequest {
    pub label_id: i64,
    pub remark: String,
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `label_id` | `i64` | 要修改的标签主键 |
| `remark` | `String` | 新的备注内容 |

### RemoveLabelRequest

解除标签关联请求体（`DELETE /api/v1/labels/remove`，需登录，仅能操作自己的关联）。

```rust
#[derive(Debug, Deserialize)]
pub struct RemoveLabelRequest {
    pub user_id: i64,
    pub label_id: i64,
}
```

## 响应 DTO

### LabelResponse

标签本体响应（`From<LabelBase>` 转换，`created_at` / `updated_at` 序列化为 RFC3339 字符串）。

```rust
#[derive(Debug, Serialize)]
pub struct LabelResponse {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub remark: String,
    pub influence: i64,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}
```

### UserLabelResponse

用户标签关联响应（`From<UserLabel>` 转换，添加标签后返回）。

```rust
#[derive(Debug, Serialize)]
pub struct UserLabelResponse {
    pub id: i64,
    pub user_id: i64,
    pub label_id: i64,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}
```

### UserLabelItem

查看用户标签列表项（`GET /api/v1/labels/list` 的 `data` 元素）。

平铺返回标签本体（`LabelBase`）的全部字段——不再内嵌摘要结构，列表元素即标签自身数据，JSON 与 `LabelResponse` 同构：

```rust
#[derive(Debug, Serialize)]
pub struct UserLabelItem {
    pub id: i64,          // 标签 ID（LabelBase.id）
    pub name: String,
    pub description: String,
    pub remark: String,
    pub influence: i64,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<LabelBase> for UserLabelItem { /* 字段一一映射 */ }
```

### UserBrief / LabelUserItem

查看标签下的用户列表项（`GET /api/v1/labels/users` 的 `data` 元素）。

```rust
#[derive(Debug, Serialize)]
pub struct UserBrief {
    pub user_id: i64,
    pub name: String,
    pub avatar: String,
}

#[derive(Debug, Serialize)]
pub struct LabelUserItem {
    pub user: UserBrief,
}
```

> 用户信息缺失时（如已注销用户），`UserBrief` 以空字符串 `name` / `avatar` 占位，`user_id` 取关联中的原始值。

## 与领域模型的对应关系

| DTO | 领域模型 | 转换方式 |
|-----|----------|----------|
| `LabelResponse` | `feel_entity::label::LabelBase` | `From<LabelBase>` |
| `UserLabelResponse` | `feel_entity::label::UserLabel` | `From<UserLabel>` |
| `UserLabelItem` | `feel_entity::label::LabelBase` | `From<LabelBase>` |
| `CreateLabelRequest` | `feel_entity::label::LabelCreate` | handler 内字段组装 |

时间字段在 DTO 层统一序列化为 RFC3339 字符串（`DateTime<Utc>::to_rfc3339()`）。
